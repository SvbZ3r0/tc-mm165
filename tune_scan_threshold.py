#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
import shutil
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BASE_SOURCE = ROOT / "versions" / "Battleships_iter9_scan_gate_p030_d1581732d025.rs"
WORK_SOURCE = ROOT / "Battleships.rs"
BENCH = ROOT / "benchmark.sh"
OUT_DIR = ROOT / "benchmarks" / "tuning_iter10_scan_threshold"

THRESHOLDS = [0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50]


def run(cmd: list[str]) -> str:
    completed = subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)
    return completed.stdout


def patch_source(threshold: float) -> None:
    src = BASE_SOURCE.read_text()
    src, n = re.subn(r"if p <= [0-9.]+ \{", f"if p <= {threshold:.2f} {{", src, count=1)
    if n != 1:
        raise RuntimeError("failed to patch scan threshold")
    WORK_SOURCE.write_text(src)


def latest_run_dir(label: str) -> Path:
    matches = sorted((ROOT / "benchmarks").glob(f"*_{label}"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not matches:
        raise RuntimeError(f"no run dir for {label}")
    return matches[0]


def score(run_dir: Path) -> float:
    return float((run_dir / "summary.tsv").read_text().splitlines()[1].split("\t")[3])


def run_one(threshold: float, seed_spec: str, prefix: str) -> dict[str, str]:
    label = f"{prefix}_p{threshold:.2f}".replace(".", "p")
    patch_source(threshold)
    start = time.time()
    run([str(BENCH), "run", label, str(WORK_SOURCE), seed_spec])
    elapsed = time.time() - start
    run_dir = latest_run_dir(label)
    return {
        "threshold": f"{threshold:.2f}",
        "seed_spec": seed_spec,
        "score": f"{score(run_dir):.6f}",
        "seconds": f"{elapsed:.3f}",
        "run_dir": str(run_dir.relative_to(ROOT)),
    }


def write_rows(path: Path, rows: list[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["threshold", "seed_spec", "score", "seconds", "run_dir"], delimiter="\t")
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    stage1 = []
    for t in THRESHOLDS:
        row = run_one(t, "1,1000", "scanthr_s1")
        stage1.append(row)
        write_rows(OUT_DIR / "stage1_1_1000.tsv", sorted(stage1, key=lambda r: float(r["score"])))
        print("stage1", row)

    top = sorted(stage1, key=lambda r: float(r["score"]))[:3]
    stage2 = []
    for row in top:
        t = float(row["threshold"])
        for seed_spec, suffix in [("1001,3000", "offset1"), ("3001,5000", "offset2")]:
            out = run_one(t, seed_spec, f"scanthr_s2_{suffix}")
            stage2.append(out)
            write_rows(OUT_DIR / "stage2_offsets.tsv", stage2)
            print("stage2", out)

    # Leave active file on best stage1 candidate for now; caller can choose after offset validation.
    best = min(stage1, key=lambda r: float(r["score"]))
    patch_source(float(best["threshold"]))


if __name__ == "__main__":
    main()
