#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCORE_RE = re.compile(r"Seed\s*=\s*(\d+),\s*Score\s*=\s*([-+]?\d+(?:\.\d+)?)")
KV_RE = re.compile(r"([A-Za-z_]+)=([^\s]+)")

BASELINE_LOGS = [
    Path("benchmarks/20260629T103704Z_iter18_diag_baseline/run_1_1000.log"),
    Path("benchmarks/20260629T103704Z_iter18_diag_baseline/run_1001_3000.log"),
    Path("benchmarks/20260629T103704Z_iter18_diag_baseline/run_3001_5000.log"),
    Path("benchmarks/20260629T162532Z_iter18_diag_baseline_5001_10000/run_5001_10000.log"),
]
VARIANT_LOGS = [
    Path("benchmarks/20260629T153333Z_iter23_beam_topk5_rerank_f005/run_1_1000.log"),
    Path("benchmarks/20260629T153716Z_iter23_beam_topk5_rerank_f005/run_1001_3000.log"),
    Path("benchmarks/20260629T154517Z_iter23_beam_topk5_rerank_f005/run_3001_5000.log"),
    Path("benchmarks/20260629T155247Z_iter23_beam_topk5_rerank_f005/run_5001_10000.log"),
]


def parse_scores(paths: list[Path]) -> dict[int, float]:
    scores = {}
    for path in paths:
        for line in (ROOT / path).read_text(errors="ignore").splitlines():
            m = SCORE_RE.search(line)
            if m:
                scores[int(m.group(1))] = float(m.group(2))
    return scores


def run_one(seed: int, trace: Path, force: tuple[int, int, int] | None = None) -> float:
    env = os.environ.copy()
    env["BS_DIAG"] = str(trace)
    if force is not None:
        env["BS_FORCE_TURN"] = str(force[0])
        env["BS_FORCE_R"] = str(force[1])
        env["BS_FORCE_C"] = str(force[2])
    label = f"cf_{seed}_{trace.stem}"[:80]
    cmd = ["./benchmark.sh", "run", label, "tmp_trace/Battleships_iter23_branch.rs", str(seed)]
    subprocess.run(cmd, cwd=ROOT, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, check=True)
    # Find newest matching summary via runs.tsv is unnecessary; parse the trace run log from benchmark output dirs by label prefix.
    candidates = sorted((ROOT / "benchmarks").glob(f"*_{label}/summary.tsv"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not candidates:
        # fallback: scan recent summaries for exact label
        candidates = sorted((ROOT / "benchmarks").glob("*/summary.tsv"), key=lambda p: p.stat().st_mtime, reverse=True)[:20]
    for summary in candidates:
        for line in summary.read_text().splitlines()[1:]:
            parts = line.split("\t")
            if len(parts) >= 4 and parts[0] == label:
                return float(parts[3])
    raise RuntimeError(f"score not found for {label}")


def first_changed_branch(trace: Path):
    pending = None
    for line in trace.read_text(errors="ignore").splitlines():
        if line.startswith("BRANCH"):
            pending = dict(KV_RE.findall(line))
        elif line.startswith("DECISION") and pending is not None:
            decision = dict(KV_RE.findall(line))
            heat_cell = pending.get("heat_cell", "")
            chosen_cell = pending.get("chosen_cell", "")
            if chosen_cell and heat_cell and chosen_cell != heat_cell:
                return pending, decision
            pending = None
    return None, None


def parse_cell(cell: str) -> tuple[int, int]:
    a, b = cell.split(",")
    return int(a), int(b)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--per-side", type=int, default=10)
    ap.add_argument("--out", type=Path, default=Path("diagnostics/iter23_counterfactual_branch_labels.tsv"))
    args = ap.parse_args()

    base = parse_scores(BASELINE_LOGS)
    var = parse_scores(VARIANT_LOGS)
    rows = []
    for seed in sorted(set(base) & set(var)):
        rows.append((var[seed] - base[seed], seed, var[seed], base[seed]))
    rows.sort(reverse=True)
    selected = [("top_loss", seed, delta) for delta, seed, _, _ in rows[: args.per_side]]
    selected += [("top_win", seed, delta) for delta, seed, _, _ in rows[-args.per_side:]]

    args.out.parent.mkdir(parents=True, exist_ok=True)
    trace_root = ROOT / "diagnostics" / "counterfactual_iter23"
    trace_root.mkdir(parents=True, exist_ok=True)
    header = [
        "group", "seed", "iter23_delta_vs_iter18", "branch_turn", "heat_cell", "policy_cell", "beam_cell",
        "score_force_heat", "score_force_policy", "beam_better", "score_delta_policy_minus_heat",
        "heat_rank_of_beam", "beam_rank_of_heat", "chosen_heat_rank", "chosen_beam_rank", "beam_states",
        "heat_best", "heat_second", "beam_best", "beam_second", "heat_cell_beam", "beam_cell_heat",
        "chosen_heat", "chosen_beam", "beam_entropy", "n", "remaining_cells",
    ]
    with (ROOT / args.out).open("w") as f:
        f.write("\t".join(header) + "\n")
        for group, seed, delta in selected:
            natural_trace = trace_root / f"seed_{seed}_natural.trace"
            run_one(seed, natural_trace, None)
            branch, decision = first_changed_branch(natural_trace)
            if branch is None:
                continue
            turn = int(decision["branch_turn"])
            heat_r, heat_c = parse_cell(branch["heat_cell"])
            policy_r, policy_c = parse_cell(branch["chosen_cell"])
            heat_trace = trace_root / f"seed_{seed}_force_heat.trace"
            policy_trace = trace_root / f"seed_{seed}_force_policy.trace"
            score_heat = run_one(seed, heat_trace, (turn, heat_r, heat_c))
            score_policy = run_one(seed, policy_trace, (turn, policy_r, policy_c))
            row = {
                "group": group,
                "seed": seed,
                "iter23_delta_vs_iter18": f"{delta:+.12f}",
                "branch_turn": turn,
                "heat_cell": branch.get("heat_cell", ""),
                "policy_cell": branch.get("chosen_cell", ""),
                "beam_cell": branch.get("beam_cell", ""),
                "score_force_heat": f"{score_heat:.12f}",
                "score_force_policy": f"{score_policy:.12f}",
                "beam_better": str(score_policy < score_heat),
                "score_delta_policy_minus_heat": f"{score_policy - score_heat:+.12f}",
            }
            for key in header[11:]:
                row[key] = branch.get(key, "")
            f.write("\t".join(str(row.get(h, "")) for h in header) + "\n")
            f.flush()
    print(ROOT / args.out)


if __name__ == "__main__":
    main()
