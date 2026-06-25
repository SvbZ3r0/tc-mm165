#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BASE_SOURCE = ROOT / "versions" / "Battleships_iter10_scan_threshold_p050_d2fb414695bd.rs"
WORK_SOURCE = ROOT / "Battleships.rs"
BENCH = ROOT / "benchmark.sh"
OUT_DIR = ROOT / "benchmarks" / "tuning_iter11_scan_threshold_high"
THRESHOLDS = [0.50, 0.55, 0.60, 0.70, 0.80, 1.00]


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)


def patch_source(threshold: float) -> None:
    src = BASE_SOURCE.read_text()
    src, n = re.subn(r"if p <= [0-9.]+ \{", f"if p <= {threshold:.2f} {{", src, count=1)
    if n != 1:
        raise RuntimeError("failed to patch threshold")
    WORK_SOURCE.write_text(src)


def latest(label: str) -> Path:
    matches = sorted((ROOT / "benchmarks").glob(f"*_{label}"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not matches:
        raise RuntimeError(label)
    return matches[0]


def score(run_dir: Path) -> float:
    return float((run_dir / "summary.tsv").read_text().splitlines()[1].split("\t")[3])


def run_one(threshold: float, seed_spec: str, prefix: str) -> dict[str, str]:
    label = f"{prefix}_p{threshold:.2f}".replace(".", "p")
    patch_source(threshold)
    start = time.time()
    run([str(BENCH), "run", label, str(WORK_SOURCE), seed_spec])
    elapsed = time.time() - start
    d = latest(label)
    return {
        "threshold": f"{threshold:.2f}",
        "seed_spec": seed_spec,
        "score": f"{score(d):.6f}",
        "seconds": f"{elapsed:.3f}",
        "run_dir": str(d.relative_to(ROOT)),
    }


def write(path: Path, rows: list[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["threshold", "seed_spec", "score", "seconds", "run_dir"], delimiter="\t")
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    s1=[]
    for t in THRESHOLDS:
        row=run_one(t,"1,1000","scanthr_hi_s1")
        s1.append(row)
        write(OUT_DIR/"stage1_1_1000.tsv", sorted(s1,key=lambda r:float(r["score"])))
        print("stage1", row)
    top=sorted(s1,key=lambda r:float(r["score"]))[:3]
    s2=[]
    for r in top:
        t=float(r["threshold"])
        for seed,suffix in [("1001,3000","offset1"),("3001,5000","offset2")]:
            row=run_one(t,seed,f"scanthr_hi_s2_{suffix}")
            s2.append(row)
            write(OUT_DIR/"stage2_offsets.tsv", s2)
            print("stage2", row)
    best=min(s1,key=lambda r:float(r["score"]))
    patch_source(float(best["threshold"]))

if __name__ == "__main__":
    main()
