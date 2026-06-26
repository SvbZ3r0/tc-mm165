# Loss Diagnostics Workflow

The purpose of this workflow is to identify high-value seeds before changing solver policy. Do not tune against a bucket until the loss mechanism is understood.

## Tool

Use:

```bash
python3 tools/loss_diagnostics.py \
  --current <current run log> \
  --baseline <baseline run log> \
  --current-label iter17 \
  --baseline-label iter16 \
  --out benchmarks/loss_diagnostics_iter17_vs_iter16_1_1000.tsv \
  --top 200
```

The script parses benchmark logs with lines like:

```text
Seed = 588, Score = 400.956630086491, RunTime = 65 ms
```

It writes two files:

- `<out>`: ranked per-seed loss table, worst `current - baseline` first.
- `<out>.summary`: overlap, win/loss counts, total delta, and max-loss seed.

The TSV includes blank `classification` and `notes` columns. These should be filled only after traced/manual inspection.

## Initial Diagnostic Tables

Generated so far:

- `benchmarks/loss_diagnostics_iter17_vs_iter16_1_1000.tsv`
- `benchmarks/loss_diagnostics_iter17_vs_random_1_1000.tsv`
- `benchmarks/loss_diagnostics_iter17_vs_checkers_1_1000.tsv`

For iter17 vs iter16 on `1..1000`:

```text
overlap_seeds  1000
loss_count     152
win_count      317
same_count     531
total_delta    -7797
max_loss_seed  588
max_loss_delta +148
```

Top iter17 losses vs iter16:

```text
seed  iter17_score     iter16_score     delta
588   400.956630086491 252.956630086491 +148
853   260.508359305730 144.508359305730 +116
256   283.553769829569 170.553769829569 +113
835   396.472928827552 289.472928827552 +107
188   308.863480118052 202.863480118052 +106
865   260.188125214032 159.188125214032 +101
952   394.294240157425 293.294240157425 +101
967   398.074965113542 307.074965113542 +91
300   298.838728641047 223.838728641047 +75
272   205.252010814079 140.252010814079 +65
22    164.224669218394 99.224669218394  +65
```

## Classification Taxonomy

Use one of these labels in the `classification` column:

- `bad_kill_inference`: solver likely marked the wrong cells dead after `KILL`, usually in touching-ship or ambiguous-hit cases.
- `chase_overcommit`: solver spent too many shots extending or revisiting a contaminated hit cluster.
- `scan_cost_waste`: scans added cost or misdirected hunt without eliminating enough future misses.
- `hunt_misses`: no obvious KILL/chase/scan issue; loss mainly came from worse hunt ordering.
- `unknown`: insufficient trace evidence.

Multiple labels may be comma-separated if needed, but prefer one primary cause.

## Required Evidence Per Label

`bad_kill_inference`:

- Loss follows a `KILL` event.
- Active hits remain nearby or later shots indicate the killed placement choice was probably wrong.
- A compared version with different KILL inference does much better on the same seed.

`chase_overcommit`:

- Many consecutive `CHASE` shots around one active cluster.
- The cluster contains hits from touching ships or ambiguous geometry.
- The compared version escapes to hunt or another cluster earlier.

`scan_cost_waste`:

- Current score has scan-cost fractional components and loses despite similar or fewer misses later.
- Scan count/cost is high relative to the final delta.
- Shot choices after scans are pulled into low-payoff regions.

`hunt_misses`:

- No active cluster or KILL ambiguity explains the delta.
- Difference is mostly in miss ordering before first/next hit.

## Next Tooling Gap

Current benchmark logs do not include command traces, shot mode, scan events, or KILL inference decisions. The loss TSV ranks seeds, but classification still needs traced replay.

Recommended next step: add an instrumented solver build that writes compact diagnostic events to stderr or a sidecar file for selected seeds only:

```text
DIAG turn=12 mode=SCAN region=0,0,9,19 count=18 cost=6.52
DIAG turn=28 mode=HUNT shot=7,13 score=...
DIAG turn=31 mode=CHASE cluster=2 cluster_len=3 shot=8,13 support=...
DIAG turn=32 result=KILL inferred_len=4 candidates=2 committed=3
```

Then replay only the top loss seeds from `benchmarks/loss_diagnostics_iter17_vs_iter16_1_1000.tsv`, starting with:

```text
588, 853, 256, 835, 188, 865, 952, 967, 300, 272, 22
```
