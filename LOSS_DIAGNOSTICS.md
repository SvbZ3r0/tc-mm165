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


## First Trace Replay Results

Generated an instrumented copy of active iter18 with `tools/make_trace_solver.py`. It writes sidecar traces through `BS_DIAG` and does not change `Battleships.rs`.

Trace artifacts:

- `tmp_trace/Battleships_trace.rs`
- `diagnostics/traces_iter18_20260626/seed_*.trace`
- `diagnostics/trace_summary_iter18_top_losses.tsv`

Initial seeds traced: `588, 853, 256, 835, 188`.

Summary:

```text
seed  score              scans  hunt_turns  chase_options  ambiguous_kills  max_miss_streak  preliminary classification
588   400.956630086491   2      320         72             2                188              hunt_misses
853   260.508359305730   2      168         78             2                110              hunt_misses
256   283.553769829569   2      187         92             3                112              hunt_misses
835   396.472928827552   2      211         179            5                113              bad_kill_inference_or_touching_ship_ambiguity,chase_overcommit,hunt_misses
188   308.863480118052   2      183         122            2                118              hunt_misses
```

Early read: these top-loss seeds are not primarily scan-cost waste. They all took exactly two scans. The dominant common symptom is very long miss streaks after the opening information, which points toward hunt ordering / posterior quality rather than fixed scan cost. Seed `835` is the main mixed case with large active clusters and multiple ambiguous KILL events.

Next diagnostic step: trace the same seeds with the previous comparator version (`iter16`) and compare decision divergence near the first large score split. That is more useful than another policy change.


## Iter13 Comparator Check

Because iter13 reportedly scored better on TC's own stack, the same top-loss seeds were traced against iter13 as well.

Artifacts:

- `tmp_trace/Battleships_iter13_trace.rs`
- `diagnostics/traces_iter13_20260626/seed_*.trace`
- `diagnostics/trace_compare_iter18_vs_iter13_top_losses.tsv`

Key comparison across the 11 top-loss seeds:

```text
seed  iter18_score       iter13_score       delta18-13  pattern
588   400.956630086491   252.956630086491   +148.000    iter18 chases leftover after KILL; iter13 hunts
853   260.508359305730   144.508359305730   +116.000    iter18 chases leftover after KILL; iter13 hunts
256   283.553769829569   285.728969192127   -2.175      hunt divergence, iter18 slightly better
835   396.472928827552   388.926647925649   +7.546      scan policy divergence, near neutral
188   308.863480118052   202.863480118052   +106.000    iter18 chases leftover after KILL; iter13 hunts
865   260.188125214032   159.188125214032   +101.000    iter18 chases leftover after KILL; iter13 hunts
952   394.294240157425   396.390673475730   -2.096      scan policy divergence, iter18 slightly better
967   398.074965113542   307.074965113542   +91.000     iter18 chases leftover after KILL; iter13 hunts
300   298.838728641047   223.838728641047   +75.000     iter18 chases leftover after KILL; iter13 hunts
272   205.252010814079   112.173341981923   +93.079     scan policy divergence, iter13 four scans
22    164.224669218394   68.340976230820    +95.884     hunt divergence
```

Inference:

- The largest repeated mechanism is not general scan waste. It is leftover-cluster preservation after KILL. Iter18's same-length ambiguity improvement is globally positive, but on these seeds it leaves an active cluster that iter13/iter16 do not keep, and iter18 then chases that leftover cluster into long losses.
- A secondary mechanism is scan policy divergence on low-P seeds where iter13 uses four quadrant scans and iter18 uses two scans. This appears mixed: iter13 is much better on seed `272`, but iter18 is slightly better on `256` and `952`.
- The next safe experiment should not revert KILL ambiguity globally. A more targeted idea is to keep ambiguity bookkeeping but delay chasing leftover clusters immediately after a KILL unless their placement support is strong. That targets the repeated failure mechanism without returning to the older overcommit behavior everywhere.

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

## Deferred Weak KILL Leftovers Experiment

The trace comparison against iter13/iter16 showed repeated losses where iter18 chased leftover clusters immediately after a `KILL`. I tested a narrow policy that deferred zero-support leftover clusters for one hunt turn.

Result on the first validation gate:

```text
iter19_defer_weak_kill_leftovers 1..1000: 139729.033399
iter18_kill_ambiguity_same_len   1..1000: 139469.033399
delta: +260.000000
```

Conclusion: reject. The diagnostic pattern exists, but delaying every zero-support leftover is not selective enough. The next version should use richer evidence before suppressing a post-KILL chase, not a simple support-zero delay.

## Singleton Adjacent KILL Leftover Defer

Follow-up to the broad deferred-leftover rejection: defer only singleton post-`KILL` leftovers adjacent to committed killed cells and with zero continuation support.

```text
1..1000:    -3.000000 vs iter18
1001..3000: +93.000000 vs iter18
combined:   +90.000000 vs iter18
```

Conclusion: reject. The narrower touching-contamination rule is much less harmful than broad deferral, but it still loses by the second range. Post-`KILL` leftover suppression needs stronger evidence than singleton adjacency plus zero support.

## Complete-Fleet Posterior Prototype

A sampled complete-fleet posterior was tested because independent placement heatmaps ignore global non-overlap and scan-count consistency. The first rejection sampler produced mixed results:

```text
v1 broad:      +563 over 1..3000
v2 late <= 2N:  +60 over 1..3000, despite -501 on 1..1000
v3 late <= N:  +172 on 1..1000
```

Conclusion: reject the current implementation, not the model class. The result suggests complete-board reasoning can find wins, but naive random rejection plus score blending is too noisy and expensive. Next posterior work should be deterministic beam/backtracking over fleet states or a better weighted sampler with diagnostics on accepted sample count and posterior confidence.

## Iter21 Beam Loss Timing Diagnostics

Question: are iter21 beam losses mainly late-game cleanup failures, or earlier trajectory changes?

Method:

- Generated fresh iter18 baseline logs for `1..1000`, `1001..3000`, and `3001..5000`.
- Compared per-seed scores against iter21 canonical beam.
- Traced the top 30 iter21 losses and top 30 iter21 wins versus iter18.
- Classified first differing decision by turn and mode.

Artifacts:

- Per-seed deltas: `diagnostics/iter21_vs_iter18_seed_deltas.tsv`
- Delta summary: `diagnostics/iter21_vs_iter18_seed_deltas.tsv.summary`
- Trace divergence table: `diagnostics/iter21_vs_iter18_trace_divergence.tsv`
- Trace divergence summary: `diagnostics/iter21_vs_iter18_trace_divergence.tsv.summary`
- Trace roots: `diagnostics/traces_iter18_vs_iter21/iter18`, `diagnostics/traces_iter18_vs_iter21/iter21`
- Scripts: `tools/diagnose_seed_deltas.py`, `tools/diagnose_trace_divergence.py`

Per-seed distribution over `1..5000`:

```text
seeds:          5000
total delta:    +90.000000
iter21 better:  1851 seeds
iter21 worse:   1874 seeds
same:           1275 seeds
top 80 losses:  +6376.000000
top 80 wins:    -6263.000000
```

First-divergence timing for traced top-30 losses:

```text
turn 3: 22 seeds
turn 5:  8 seeds
mode: iter21 CHASE, iter18 HUNT in all 30
```

First-divergence timing for traced top-30 wins:

```text
turn 3: 28 seeds
turn 5:  2 seeds
mode: iter21 CHASE, iter18 HUNT in all 30
```

Inference: the beam's biggest wins and losses both originate in the opening trajectory, usually immediately after the first beam-influenced hunt shot creates a chase path while iter18 remains in hunt. The issue is not primarily late-game cleanup. This also explains why late-game-only beam variants did not solve the problem: by the time `remaining_cells <= N` or similar gates trigger, the major trajectory divergence has already happened or the beam has too little leverage left.

Implication for iter22: do not frame the beam as a late-game patch. The safer experiment is a confidence override only for early/uncertain hunt decisions, with strict guardrails, or diagnostics that compare heatmap-best versus beam-best before allowing the beam to create an early chase branch.

## Iter21 Top Swing Separator Diagnostics

Question: the top iter21 losses and wins are nearly symmetric (`top 80 losses +6376`, `top 80 wins -6263`). Is there an if/then condition that separates the good beam branches from the bad ones?

Additional diagnostics added:

- Enhanced traces with `GAME` and `RESULT` lines.
- Beam-stat traces logging the first beam posterior state count and heatmap/beam best-second gaps.
- Feature tables:
  - `diagnostics/iter21_top_swing_features.tsv`
  - `diagnostics/iter21_top_swing_early_window.tsv`
  - `diagnostics/iter21_beamstat_features.tsv`
  - `diagnostics/iter21_beamstat_gate_impact.tsv`
- Scripts:
  - `tools/diagnose_top_swing_features.py`
  - `tools/diagnose_early_window.py`
  - `tools/diagnose_beamstat_features.py`

Findings:

- N/P buckets and immediate result-window features did not cleanly separate wins from losses.
- The strongest available discriminator was beam posterior state count.
- On the traced top-60 swing seeds, allow-beam gates around low state count (`states <= ~53..70`) had the best net impact. This means the beam is more reliable when the complete-fleet posterior is concentrated into fewer surviving states, and more dangerous when many states survive and the truncated beam can become confidently wrong.

Interpretation:

The symmetrical variance is not late game and not explained by simple P/N buckets. It appears to be a posterior-confidence issue: when the beam state set is diffuse/high-count, its selected branch is high variance. When state count is low, the constraints are tight enough that beam guidance is more often useful.

Candidate iter22 condition:

```text
Use beam only if:
  beam_states <= 65   # tune 53, 59, 65, 70
  and heatmap is not already decisive
```

This should be tested as an allow-beam gate, not as a blend everywhere.

## rejected_iter22_beam_confidence_override_s65_h105

Reason: iter21's top wins/losses were nearly symmetric and first diverged at turns 3/5. Follow-up diagnostics suggested beam state count was the best available separator among traced swing seeds, with low state counts around `<= 53..70` preserving more top wins than losses.

Hypothesis: use the beam only as a confidence-gated hunt override, not as a global blend. Start with `BEAM_OVERRIDE_MAX_STATES = 65` and require the heatmap to be non-decisive (`HEATMAP_DECISIVE_RATIO = 1.05`). Also move beam magic constants to named top-level constants.

Experiment: started from iter21 canonical beam, replaced the `0.70/0.30` heatmap/beam blend with a gated override. If `beam_states <= 65` and `heat_best / heat_second <= 1.05`, score cells by normalized beam posterior; otherwise use iter18 heatmap scoring.

Result: rejected after the second validation range.

```text
1..1000:    iter22 139132.033399  iter18 139469.033399  delta -337.000000   seconds 209
1001..3000: iter22 283485.422478  iter18 281346.422478  delta +2139.000000  seconds 407
combined:   iter22 422617.455877  iter18 420815.455877  delta +1802.000000
```

Archive:

- Source: `versions/Battleships_rejected_iter22_beam_confidence_override_s65_h105_5fb51539fd87.rs`
- Validation: `benchmarks/validation_iter22_beam_confidence_override.tsv`
- Archive manifest: `archives/20260629T125818Z_rejected_iter22_beam_confidence_override_s65_h105_5fb51539fd87/manifest.txt`

Inference: the traced top-swing separator did not generalize. Low beam state count is not sufficient as a global allow condition, and converting blend to hard override made the second range substantially worse. Further beam use needs richer confidence diagnostics or per-decision counterfactual logging, not only state-count gating.

## rejected_iter23_beam_topk_rerank

Reason: iter22 showed that hard beam override was too violent. The next hypothesis was to use the complete-fleet beam only as a tie-breaker/veto among cells the original heatmap already liked, preventing beam from selecting an unrelated branch.

Hypothesis: take the top `K` heatmap candidates and rerank only those by `heat_score * (1 + factor * normalized_beam_score)`. This lets the beam discourage weak heatmap choices without choosing outside the heatmap's plausible set.

Experiment:

- Started from iter21 canonical beam.
- Added top-level constants for beam widths, placement limits, min states, `BEAM_RERANK_TOP_K`, and `BEAM_RERANK_FACTOR`.
- Preserved existing heatmap scoring for every cell.
- Collected candidates, sorted by heatmap score, took top `K=5`, and reranked with beam support.
- Tested factors `0.15` and `0.05`.

Results:

```text
topk5_f015:
  1..1000:     iter23 139380.033399  iter18 139469.033399  delta -89.000000
  1001..3000:  iter23 282355.422478  iter18 281346.422478  delta +1009.000000
  combined:    delta +920.000000

topk5_f005:
  1..1000:     iter23 138872.033399  iter18 139469.033399  delta -597.000000
  1001..3000:  iter23 281604.422478  iter18 281346.422478  delta +258.000000
  3001..5000:  iter23 282075.279304  iter18 282197.279304  delta -122.000000
  5001..10000: iter23 719395.572380  iter18 718206.572380  delta +1189.000000
  combined:    delta +728.000000
```

Archive:

- `versions/Battleships_rejected_iter23_beam_topk5_rerank_f015_0069b0e586bf.rs`
- Candidate archive: `versions/Battleships_iter23_beam_topk5_rerank_f005_candidate_c314141b94c0.rs`
- Rejected final: `versions/Battleships_rejected_iter23_beam_topk5_rerank_f005_c314141b94c0.rs`
- Validation: `benchmarks/validation_iter23_beam_topk_rerank.tsv`

Inference: top-K reranking is much less destructive than hard override and `factor=0.05` nearly validated through `1..5000`, but the final `5001..10000` range failed. The beam still introduces unstable early trajectory changes, just more softly. This remains a promising direction, but the current expensive beam is not robust enough to promote.

## Iter23 Counterfactual Branch Labels

Purpose: move beyond seed-level correlation by labeling the first rerank branch directly. For selected top iter23 loss/win seeds, the tooling finds the first HUNT decision where the top-K beam reranker chooses a different cell from pure heatmap, then reruns two counterfactual branches:

- A: force the heatmap cell at that branch turn, then continue the same iter23 policy.
- B: force the iter23 policy/reranked cell at that branch turn, then continue the same iter23 policy.

Label:

```text
beam_better = score(B) < score(A)
```

Artifacts:

- Instrumented source: `tmp_trace/Battleships_iter23_branch.rs`
- Dataset builder: `tools/build_counterfactual_dataset.py`
- Dataset: `diagnostics/iter23_counterfactual_branch_labels.tsv`
- Summary: `diagnostics/iter23_counterfactual_branch_labels.summary`
- Predicate ranking: `diagnostics/iter23_counterfactual_branch_label_predicates.tsv`
- Trace root: `diagnostics/counterfactual_iter23/`

Pilot scope: top 20 iter23 losses and top 20 iter23 wins versus iter18. Only seeds where iter23 actually reranked away from heatmap produce labeled rows.

Resulting labels:

```text
rows: 27
beam_better=True:  9
beam_better=False: 18
rows from top_loss seeds: 14  (beam better 2, beam worse 12)
rows from top_win seeds:  13  (beam better 7, beam worse 6)
```

Best simple separators so far:

```text
remaining_cells >= 26       -> beam good 3, beam bad 15
beam_entropy >= 3.60494     -> beam good 1, beam bad 13
beam_best >= 0.832888       -> beam good 3, beam bad 15
beam_second >= 0.832888     -> beam good 3, beam bad 15
```

Interpretation: the reranker is most dangerous when the branch happens while many ship cells remain and the beam posterior is diffuse/high-entropy. This is consistent with the earlier variance diagnosis: early high-uncertainty beam nudges create large trajectory swings. A plausible next tested rule is not another top-K/factor sweep, but an abstention gate such as:

```text
allow rerank only if remaining_cells < 26
and beam_entropy < 3.60
```

This is preliminary: the labeled dataset is small, but it now labels branch correctness directly rather than using whole-solver outcomes as a proxy.

## rejected_iter24_diverse_beam_sig20_m3

Reason: counterfactual diagnostics suggested the beam posterior may be mode-collapsed: early high-entropy/high-uncertainty beam nudges are often wrong, so we tried to improve the posterior itself rather than tune a gate.

Hypothesis: keep a more diverse set of beam states by limiting how many retained states may share the same occupancy signature over the top heatmap cells. This should reduce fake consensus from near-duplicate beam states.

Experiment:

- Started from iter21 canonical beam.
- Added top-level beam constants.
- Computed the top `BEAM_DIVERSITY_CELLS = 20` heatmap cells in `best_hunt_cell`.
- During every beam expansion, computed a 64-bit signature of each partial state's occupancy over those cells.
- Kept at most `BEAM_MAX_STATES_PER_SIGNATURE = 3` states per signature before applying normal beam width.

Result: rejected on the first gate.

```text
1..1000: iter24 139878.033399  iter18 139469.033399  delta +409.000000  seconds 184
```

Archive:

- Source: `versions/Battleships_rejected_iter24_diverse_beam_sig20_m3_f34293867ef8.rs`
- Validation: `benchmarks/validation_iter24_diverse_beam.tsv`
- Archive manifest: `archives/20260629T164749Z_rejected_iter24_diverse_beam_sig20_m3_f34293867ef8/manifest.txt`

Inference: this diversity constraint was too blunt. The beam's concentration over top heatmap cells is not purely fake consensus; forcing diversity there removed useful signal. If we revisit diversity, it should preserve high-probability consensus but diversify lower-ranked alternatives, or diversify by complete-fleet placement identity rather than top-cell occupancy.
