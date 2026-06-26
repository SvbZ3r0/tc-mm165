#!/usr/bin/env python3
"""Compare Topcoder benchmark logs seed-by-seed and rank losses.

This is intentionally separate from the solver. It reads benchmark run logs with
lines like:

    Seed = 3001, Score = 78.70, RunTime = 6 ms

and writes a TSV that can be kept as long-term diagnostics. The final columns
are reserved for root-cause labels after traced/manual inspection.
"""

from __future__ import annotations

import argparse
import re
from pathlib import Path
from statistics import mean

LINE_RE = re.compile(
    r"^Seed = (?P<seed>\d+), Score = (?P<score>-?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?), RunTime = (?P<runtime>\d+) ms$"
)


def parse_log(path: Path) -> dict[int, tuple[float, int]]:
    rows: dict[int, tuple[float, int]] = {}
    for line in path.read_text().splitlines():
        match = LINE_RE.match(line.strip())
        if not match:
            continue
        rows[int(match.group("seed"))] = (
            float(match.group("score")),
            int(match.group("runtime")),
        )
    if not rows:
        raise SystemExit(f"no seed rows found in {path}")
    return rows


def severity(delta: float) -> str:
    if delta >= 100:
        return "huge_loss"
    if delta >= 50:
        return "large_loss"
    if delta >= 20:
        return "medium_loss"
    if delta > 0:
        return "small_loss"
    if delta == 0:
        return "same"
    if delta <= -100:
        return "huge_win"
    if delta <= -50:
        return "large_win"
    if delta <= -20:
        return "medium_win"
    return "small_win"


def possible_scan_cost(score: float) -> str:
    # Integer scores generally mean no scan cost was charged. Fractional score is
    # a cheap signal that scans happened, not proof that scans caused the loss.
    return "yes" if abs(score - round(score)) > 1e-9 else "no"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--current", required=True, type=Path, help="current solver run log")
    parser.add_argument("--baseline", required=True, type=Path, help="baseline run log")
    parser.add_argument("--current-label", default="current")
    parser.add_argument("--baseline-label", default="baseline")
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--top", type=int, default=200, help="number of worst-loss seeds to write; 0 writes all")
    args = parser.parse_args()

    current = parse_log(args.current)
    baseline = parse_log(args.baseline)
    seeds = sorted(set(current) & set(baseline))
    if not seeds:
        raise SystemExit("no overlapping seeds")

    rows = []
    for seed in seeds:
        current_score, current_ms = current[seed]
        baseline_score, baseline_ms = baseline[seed]
        delta = current_score - baseline_score
        ratio = current_score / baseline_score if baseline_score != 0 else float("inf")
        rows.append((seed, current_score, baseline_score, delta, ratio, current_ms, baseline_ms))

    losses = [row for row in rows if row[3] > 0]
    wins = [row for row in rows if row[3] < 0]
    rows.sort(key=lambda row: row[3], reverse=True)
    selected = rows if args.top == 0 else rows[: args.top]

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w") as f:
        f.write(
            "seed\tcurrent_label\tbaseline_label\tcurrent_score\tbaseline_score\tdelta_current_minus_baseline\tratio\tcurrent_ms\tbaseline_ms\tseverity\tcurrent_has_scan_cost\tclassification\tnotes\n"
        )
        for seed, current_score, baseline_score, delta, ratio, current_ms, baseline_ms in selected:
            f.write(
                f"{seed}\t{args.current_label}\t{args.baseline_label}\t"
                f"{current_score:.12f}\t{baseline_score:.12f}\t{delta:.12f}\t{ratio:.9f}\t"
                f"{current_ms}\t{baseline_ms}\t{severity(delta)}\t{possible_scan_cost(current_score)}\t\t\n"
            )

    summary = args.out.with_suffix(args.out.suffix + ".summary")
    with summary.open("w") as f:
        f.write(f"current_label\t{args.current_label}\n")
        f.write(f"baseline_label\t{args.baseline_label}\n")
        f.write(f"current_log\t{args.current}\n")
        f.write(f"baseline_log\t{args.baseline}\n")
        f.write(f"overlap_seeds\t{len(seeds)}\n")
        f.write(f"loss_count\t{len(losses)}\n")
        f.write(f"win_count\t{len(wins)}\n")
        f.write(f"same_count\t{len(seeds) - len(losses) - len(wins)}\n")
        f.write(f"mean_delta\t{mean(row[3] for row in rows):.12f}\n")
        f.write(f"total_delta\t{sum(row[3] for row in rows):.12f}\n")
        if losses:
            f.write(f"mean_loss_delta\t{mean(row[3] for row in losses):.12f}\n")
            f.write(f"max_loss_seed\t{rows[0][0]}\n")
            f.write(f"max_loss_delta\t{rows[0][3]:.12f}\n")

    print(f"wrote {args.out}")
    print(f"wrote {summary}")


if __name__ == "__main__":
    main()
