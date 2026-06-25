#!/usr/bin/env python3
from __future__ import annotations

import csv
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BASE_SOURCE = ROOT / "versions" / "Battleships_iter10_quadrant_density_scans_rejected_0eb7e8447508.rs"
WORK_SOURCE = ROOT / "Battleships.rs"
BENCH = ROOT / "benchmark.sh"
OUT_DIR = ROOT / "benchmarks" / "tuning_iter12_scan_policy"
POLICIES = [0, 2, 3, 4]


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)


def latest(label: str) -> Path:
    matches = sorted((ROOT / "benchmarks").glob(f"*_{label}"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not matches:
        raise RuntimeError(label)
    return matches[0]


def score(run_dir: Path) -> float:
    return float((run_dir / "summary.tsv").read_text().splitlines()[1].split("\t")[3])


def schedule_source(policy: int) -> str:
    return """const OPENING_SCAN_POLICY: usize = {policy};

fn opening_scan_schedule(n: usize, _p: f64) -> Vec<Scan> {{
    let mut scans = Vec::new();
    let mid = n / 2;
    match OPENING_SCAN_POLICY {{
        0 => {{}}
        2 => {{
            scans.push(Scan {{ r1: 0, c1: 0, r2: mid - 1, c2: n - 1, count: 0, kind: ScanKind::TopHalf }});
            scans.push(Scan {{ r1: 0, c1: 0, r2: n - 1, c2: mid - 1, count: 0, kind: ScanKind::LeftHalf }});
        }}
        3 => {{
            scans.push(Scan {{ r1: 0, c1: 0, r2: mid - 1, c2: n - 1, count: 0, kind: ScanKind::TopHalf }});
            scans.push(Scan {{ r1: 0, c1: 0, r2: n - 1, c2: mid - 1, count: 0, kind: ScanKind::LeftHalf }});
            scans.push(Scan {{ r1: 0, c1: 0, r2: mid - 1, c2: mid - 1, count: 0, kind: ScanKind::TopLeft }});
        }}
        4 => {{
            scans.push(Scan {{ r1: 0, c1: 0, r2: mid - 1, c2: mid - 1, count: 0, kind: ScanKind::TopLeft }});
            scans.push(Scan {{ r1: 0, c1: mid, r2: mid - 1, c2: n - 1, count: 0, kind: ScanKind::Other }});
            scans.push(Scan {{ r1: mid, c1: 0, r2: n - 1, c2: mid - 1, count: 0, kind: ScanKind::Other }});
            scans.push(Scan {{ r1: mid, c1: mid, r2: n - 1, c2: n - 1, count: 0, kind: ScanKind::Other }});
        }}
        _ => unreachable!(),
    }}
    scans
}}
""".format(policy=policy)


def patch_source(policy: int) -> None:
    src = BASE_SOURCE.read_text()
    fn_start = src.index("fn opening_scan_schedule(")
    fn_end = src.index("\nfn apply_zero_scan", fn_start)
    src = src[:fn_start] + schedule_source(policy) + src[fn_end:]
    WORK_SOURCE.write_text(src)


def run_one(policy: int, seed_spec: str, prefix: str) -> dict[str, str]:
    label = f"{prefix}_pol{policy}"
    patch_source(policy)
    start = time.time()
    run([str(BENCH), "run", label, str(WORK_SOURCE), seed_spec])
    elapsed = time.time() - start
    d = latest(label)
    return {"policy": str(policy), "seed_spec": seed_spec, "score": f"{score(d):.6f}", "seconds": f"{elapsed:.3f}", "run_dir": str(d.relative_to(ROOT))}


def write(path: Path, rows: list[dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["policy", "seed_spec", "score", "seconds", "run_dir"], delimiter="\t")
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    s1=[]
    for pol in POLICIES:
        row=run_one(pol,"1,1000","scanpol_s1")
        s1.append(row)
        write(OUT_DIR/"stage1_1_1000.tsv", sorted(s1,key=lambda r:float(r["score"])))
        print("stage1", row)
    top=sorted(s1,key=lambda r:float(r["score"]))[:3]
    s2=[]
    for r in top:
        pol=int(r["policy"])
        for seed,suffix in [("1001,3000","offset1"),("3001,5000","offset2")]:
            row=run_one(pol,seed,f"scanpol_s2_{suffix}")
            s2.append(row)
            write(OUT_DIR/"stage2_offsets.tsv", s2)
            print("stage2", row)
    best=min(s1,key=lambda r:float(r["score"]))
    patch_source(int(best["policy"]))

if __name__ == "__main__":
    main()
