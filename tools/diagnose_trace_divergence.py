#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from collections import Counter, defaultdict
from pathlib import Path

KV_RE = re.compile(r"([A-Za-z_]+)=([^\s]+)")


def parse_trace(path: Path) -> list[dict[str, str]]:
    decisions: list[dict[str, str]] = []
    for line in path.read_text(errors="ignore").splitlines():
        if "mode=" not in line or "turn=" not in line or "cell=" not in line:
            continue
        d = dict(KV_RE.findall(line))
        d["raw"] = line
        if "turn" in d:
            decisions.append(d)
    decisions.sort(key=lambda x: int(x.get("turn", "0")))
    return decisions


def score_rows(path: Path) -> dict[int, tuple[float, float, float]]:
    rows = {}
    for line in path.read_text().splitlines()[1:]:
        if not line.strip():
            continue
        seed, vs, bs, delta = line.split("\t")[:4]
        rows[int(seed)] = (float(vs), float(bs), float(delta))
    return rows


def turn_bucket(turn: int | None) -> str:
    if turn is None:
        return "no_divergence"
    if turn <= 30:
        return "early_<=30"
    if turn <= 80:
        return "mid_31_80"
    return "late_>80"


def remaining_bucket(value: str | None) -> str:
    if value is None:
        return "remaining_unknown"
    try:
        n = int(value.rstrip(","))
    except ValueError:
        return "remaining_unknown"
    if n <= 20:
        return "remaining_<=20"
    if n <= 40:
        return "remaining_21_40"
    if n <= 80:
        return "remaining_41_80"
    return "remaining_>80"


def first_divergence(a: list[dict[str, str]], b: list[dict[str, str]]) -> tuple[int | None, dict[str, str] | None, dict[str, str] | None]:
    for da, db in zip(a, b):
        if da.get("cell") != db.get("cell") or da.get("mode") != db.get("mode"):
            return int(da.get("turn", db.get("turn", "0"))), da, db
    if len(a) != len(b):
        d = a[min(len(a), len(b))] if len(a) > len(b) else b[min(len(a), len(b))]
        return int(d.get("turn", "0")), (a[min(len(a), len(b))] if len(a) > len(b) else None), (b[min(len(a), len(b))] if len(b) > len(a) else None)
    return None, None, None


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--deltas", required=True, type=Path)
    ap.add_argument("--base-dir", required=True, type=Path)
    ap.add_argument("--variant-dir", required=True, type=Path)
    ap.add_argument("--loss-seeds", required=True, type=Path)
    ap.add_argument("--win-seeds", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args()

    deltas = score_rows(args.deltas)
    loss_seeds = [int(x) for x in args.loss_seeds.read_text().split()]
    win_seeds = [int(x) for x in args.win_seeds.read_text().split()]
    groups = [("top_loss", s) for s in loss_seeds] + [("top_win", s) for s in win_seeds]

    rows = []
    counters: dict[str, Counter[str]] = defaultdict(Counter)
    for group, seed in groups:
        base_trace = parse_trace(args.base_dir / f"seed_{seed}.trace")
        var_trace = parse_trace(args.variant_dir / f"seed_{seed}.trace")
        turn, base_dec, var_dec = first_divergence(base_trace, var_trace)
        vs, bs, delta = deltas[seed]
        mode = (var_dec or {}).get("mode", "none")
        base_mode = (base_dec or {}).get("mode", "none")
        # Trace generator versions used different names over time; keep whichever is present.
        remaining = (var_dec or {}).get("remaining_cells") or (var_dec or {}).get("remaining") or (var_dec or {}).get("rem")
        active = (var_dec or {}).get("active") or (var_dec or {}).get("cluster") or "unknown"
        tb = turn_bucket(turn)
        rb = remaining_bucket(remaining)
        counters[group][tb] += 1
        counters[group][f"mode:{mode}"] += 1
        counters[group][f"base_mode:{base_mode}"] += 1
        counters[group][rb] += 1
        rows.append({
            "group": group,
            "seed": seed,
            "delta": delta,
            "variant_score": vs,
            "baseline_score": bs,
            "divergence_turn": turn if turn is not None else "",
            "turn_bucket": tb,
            "variant_mode": mode,
            "baseline_mode": base_mode,
            "remaining": remaining or "",
            "remaining_bucket": rb,
            "active": active,
            "variant_cell": (var_dec or {}).get("cell", ""),
            "baseline_cell": (base_dec or {}).get("cell", ""),
            "variant_raw": (var_dec or {}).get("raw", ""),
            "baseline_raw": (base_dec or {}).get("raw", ""),
        })

    args.out.parent.mkdir(parents=True, exist_ok=True)
    header = ["group","seed","delta","variant_score","baseline_score","divergence_turn","turn_bucket","variant_mode","baseline_mode","remaining","remaining_bucket","active","variant_cell","baseline_cell","variant_raw","baseline_raw"]
    with args.out.open("w") as f:
        f.write("\t".join(header) + "\n")
        for row in rows:
            f.write("\t".join(str(row[h]).replace("\t", " ") for h in header) + "\n")

    summary = args.out.with_suffix(args.out.suffix + ".summary")
    with summary.open("w") as f:
        for group in ["top_loss", "top_win"]:
            f.write(f"[{group}]\n")
            for key, count in counters[group].most_common():
                f.write(f"{key}\t{count}\n")
            f.write("\n")
    print(args.out)
    print(summary)


if __name__ == "__main__":
    main()
