#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from pathlib import Path

LINE_RE = re.compile(r"Seed\s*=\s*(\d+),\s*Score\s*=\s*([-+]?\d+(?:\.\d+)?)")


def parse_log(path: Path) -> dict[int, float]:
    scores: dict[int, float] = {}
    for line in path.read_text(errors="ignore").splitlines():
        m = LINE_RE.search(line)
        if m:
            scores[int(m.group(1))] = float(m.group(2))
    return scores


def turn_bucket(turn: int | None) -> str:
    if turn is None:
        return "no_divergence"
    if turn <= 30:
        return "early_<=30"
    if turn <= 80:
        return "mid_31_80"
    return "late_>80"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--baseline", nargs="+", required=True, type=Path)
    ap.add_argument("--variant", nargs="+", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--top", type=int, default=80)
    args = ap.parse_args()

    base: dict[int, float] = {}
    var: dict[int, float] = {}
    for p in args.baseline:
        base.update(parse_log(p))
    for p in args.variant:
        var.update(parse_log(p))

    seeds = sorted(set(base) & set(var))
    rows = []
    for seed in seeds:
        delta = var[seed] - base[seed]
        rows.append((delta, seed, var[seed], base[seed]))
    rows.sort(reverse=True)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w") as f:
        f.write("seed\tvariant_score\tbaseline_score\tdelta_variant_minus_baseline\n")
        for delta, seed, vs, bs in rows:
            f.write(f"{seed}\t{vs:.12f}\t{bs:.12f}\t{delta:+.12f}\n")

    wins = sum(1 for d, *_ in rows if d < 0)
    losses = sum(1 for d, *_ in rows if d > 0)
    same = len(rows) - wins - losses
    total = sum(d for d, *_ in rows)
    top_loss = sum(d for d, *_ in rows[: args.top])
    top_win = sum(d for d, *_ in rows[-args.top:])
    summary = args.out.with_suffix(args.out.suffix + ".summary")
    summary.write_text(
        f"seeds\t{len(rows)}\n"
        f"total_delta\t{total:+.12f}\n"
        f"variant_better\t{wins}\n"
        f"variant_worse\t{losses}\n"
        f"same\t{same}\n"
        f"top_{args.top}_loss_delta\t{top_loss:+.12f}\n"
        f"top_{args.top}_win_delta\t{top_win:+.12f}\n"
    )
    print(summary)
    print(args.out)


if __name__ == "__main__":
    main()
