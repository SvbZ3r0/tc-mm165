#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import itertools
import re
import shutil
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BASE_SOURCE = ROOT / "versions" / "Battleships_iter7_blended_heatmap_a50_d5a44e0a1504.rs"
WORK_SOURCE = ROOT / "Battleships.rs"
BENCH = ROOT / "benchmark.sh"
RESULTS_DIR = ROOT / "benchmarks" / "tuning_iter8_constants"

ALPHAS = [0.40, 0.50, 0.60]
SINGLE_HIT_WEIGHTS = [0.15, 0.25, 0.35, 0.50]
CENTER_BIASES = [0.0, 0.0005, 0.0010]


def run(cmd: list[str]) -> str:
    completed = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=True,
    )
    return completed.stdout


def patch_source(alpha: float, single_hit: float, center_bias: float) -> None:
    src = BASE_SOURCE.read_text()
    src = re.sub(
        r"const HEATMAP_ALPHA: f64 = [0-9.]+;",
        f"const HEATMAP_ALPHA: f64 = {alpha:.4f};",
        src,
        count=1,
    )
    src = re.sub(
        r"let placement_bonus = if active_hits\.len\(\) == 1 \{ placement_score \* [0-9.]+ \} else \{ placement_score \};",
        f"let placement_bonus = if active_hits.len() == 1 {{ placement_score * {single_hit:.4f} }} else {{ placement_score }};",
        src,
    )
    src = re.sub(
        r"[0-9.]+ \* \(n as f64 - dr - dc\)",
        f"{center_bias:.4f} * (n as f64 - dr - dc)",
        src,
        count=1,
    )
    WORK_SOURCE.write_text(src)


def latest_run_dir(label: str) -> Path:
    matches = sorted((ROOT / "benchmarks").glob(f"*_{label}"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not matches:
        raise RuntimeError(f"no run dir for {label}")
    return matches[0]


def read_score(run_dir: Path) -> float:
    lines = (run_dir / "summary.tsv").read_text().splitlines()
    return float(lines[1].split("\t")[3])


def run_config(alpha: float, single_hit: float, center_bias: float, seed_spec: str, prefix: str) -> dict[str, str]:
    label = f"{prefix}_a{alpha:.2f}_sh{single_hit:.2f}_cb{center_bias:.4f}".replace(".", "p")
    patch_source(alpha, single_hit, center_bias)
    start = time.time()
    run([str(BENCH), "run", label, str(WORK_SOURCE), seed_spec])
    seconds = time.time() - start
    run_dir = latest_run_dir(label)
    score = read_score(run_dir)
    return {
        "alpha": f"{alpha:.4f}",
        "single_hit_weight": f"{single_hit:.4f}",
        "center_bias": f"{center_bias:.4f}",
        "seed_spec": seed_spec,
        "score": f"{score:.6f}",
        "seconds": f"{seconds:.3f}",
        "run_dir": str(run_dir.relative_to(ROOT)),
    }


def write_rows(path: Path, rows: list[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fields = ["alpha", "single_hit_weight", "center_bias", "seed_spec", "score", "seconds", "run_dir"]
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields, delimiter="\t")
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage1-seeds", default="1,300")
    parser.add_argument("--stage2-seeds", default="1,1000")
    parser.add_argument("--top", type=int, default=5)
    parser.add_argument("--stage", choices=["stage1", "stage2", "all"], default="all")
    args = parser.parse_args()

    if not BASE_SOURCE.exists():
        raise SystemExit(f"missing baseline source: {BASE_SOURCE}")

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    stage1_path = RESULTS_DIR / "stage1_1_300.tsv"
    stage2_path = RESULTS_DIR / "stage2_1_1000.tsv"

    stage1_rows: list[dict[str, str]] = []
    if args.stage in ("stage1", "all"):
        for alpha, single_hit, center_bias in itertools.product(ALPHAS, SINGLE_HIT_WEIGHTS, CENTER_BIASES):
            row = run_config(alpha, single_hit, center_bias, args.stage1_seeds, "tune_s1")
            stage1_rows.append(row)
            write_rows(stage1_path, sorted(stage1_rows, key=lambda r: float(r["score"])))
            print("stage1", row)
    else:
        with stage1_path.open() as f:
            stage1_rows = list(csv.DictReader(f, delimiter="\t"))

    top_rows = sorted(stage1_rows, key=lambda r: float(r["score"]))[: args.top]

    stage2_rows: list[dict[str, str]] = []
    if args.stage in ("stage2", "all"):
        for row in top_rows:
            stage2 = run_config(
                float(row["alpha"]),
                float(row["single_hit_weight"]),
                float(row["center_bias"]),
                args.stage2_seeds,
                "tune_s2",
            )
            stage2_rows.append(stage2)
            write_rows(stage2_path, sorted(stage2_rows, key=lambda r: float(r["score"])))
            print("stage2", stage2)

    # Leave the working solver on the best stage2 config if available, otherwise restore baseline.
    if stage2_rows:
        best = min(stage2_rows, key=lambda r: float(r["score"]))
        patch_source(float(best["alpha"]), float(best["single_hit_weight"]), float(best["center_bias"]))
    else:
        shutil.copyfile(BASE_SOURCE, WORK_SOURCE)


if __name__ == "__main__":
    main()
