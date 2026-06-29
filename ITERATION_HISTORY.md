# Iteration History

This document records the main solver iterations, tuning sweeps, and rejected experiments for the MM165 Battleships solver. Lower score is better. Source snapshots and run artifacts are indexed in `versions/archive.tsv`; this file explains the intent and outcome behind those archived hashes.

## Current Best

Current active solver: `iter18_kill_ambiguity_same_len`

- Source: `versions/Battleships_iter18_kill_ambiguity_same_len_62877d07fd2e.rs`
- Hash: `62877d07fd2ec8f7ea7eb16f330ee8dc7be365c674abe51325c83e407417bb1a`
- Archive: `archives/20260626T004555Z_iter18_kill_ambiguity_same_len_62877d07fd2e`
- Validation: `benchmarks/validation_iter18_kill_ambiguity_same_len.tsv`

Key validation vs iter16:

```text
range       iter17          iter16          delta
1..1000     139876.033399   147673.033399   -7797
1001..3000  281811.422478   296704.422478   -14893
3001..5000  282439.279304   295901.279304   -13462
5001..10000 719451.572380   754752.572380   -35301
combined    1423578.307561  1495031.307561  -71453
```

The largest durable win came from exact placement-based KILL inference. The current solver also retains the scan-weighted heatmap and conditional opening scan policy from earlier successful iterations.

## Baselines

### iter0 random

Idea: shoot random unshot cells until all ships are killed.

- Source: `versions/Battleships_v0_random.rs`
- Hash: `569fc925e1a9ddc73cd231f3a1bcee2d7a3370e2e8810ef15acfc3587318e8ea`

Recent comparison over `1..5000`:

```text
range       iter0_random    iter17          iter17_vs_random
1..1000     206006.000000   139876.033399   -66129.966601
1001..3000  407199.000000   281811.422478   -125387.577522
3001..5000  412692.000000   282439.279304   -130252.720696
combined    1025897.000000  704126.735181   -321770.264819
```

### v0 checkerboard random

Idea: randomize cells on one checkerboard parity first, then randomize the other parity. This was tested as a stronger random-family baseline.

- Source: `versions/Battleships_v0_checkers_random_4260c81fd588.rs`
- Hash: `4260c81fd588d3f24c9d302944c455771e600527ebd27edd077b3295b566f346`
- Validation: `benchmarks/validation_iter17_vs_random_checkers_1_5000.tsv`

Result: worse than plain random by `+3537` over `1..5000`. Likely reason: length-1 ships make checkerboard coverage a bad assumption.

```text
range       iter0_random    checkers_random  iter17
1..1000     206006.000000   206600.000000    139876.033399
1001..3000  407199.000000   408968.000000    281811.422478
3001..5000  412692.000000   413866.000000    282439.279304
combined    1025897.000000  1029434.000000   704126.735181
```

## Accepted Mainline Iterations

### iter1

Idea: first non-random solver. This established the initial chase/hunt structure after the pure random baseline.

- Source: `versions/Battleships_iter1_8695d1295dee.rs`
- Archive: `archives/20260624T153010Z_iter1_8695d1295dee`

Outcome: accepted as the first meaningful baseline. Later iterations replaced it.

### rejected iter2 kill clusters

Idea: track hit clusters and infer killed clusters from `KILL` responses.

- Source: `versions/Battleships_iter2_kill_clusters_rejected_bee34ab8d698.rs`
- Archive: `archives/20260624T161231Z_iter2_kill_clusters_rejected_bee34ab8d698`

Outcome: rejected. The cluster model was too crude because ships can touch, so connected hits are not always a single ship.

### iter2 hit-compatible heatmap

Idea: use a placement heatmap that respects known hits/misses, rather than random hunt choices.

- Source: `versions/Battleships_iter2_hit_compatible_heatmap_5507d1a554b1.rs`
- Archive: `archives/20260624T162058Z_iter2_hit_compatible_heatmap_5507d1a554b1`

Outcome: accepted. Placement-compatible heatmaps were a durable direction and remain the core of later versions.

### iter3 kill segment inference

Idea: improve `KILL` handling by marking the most plausible killed segment instead of a whole connected cluster.

- Source: `versions/Battleships_iter3_kill_segment_inference_de40c6b33d55.rs`
- Archive: `archives/20260624T162541Z_iter3_kill_segment_inference_de40c6b33d55`

Outcome: accepted. Segment-based kill handling reduced damage from touching ships, though later exact placement inference superseded it.

### rejected iter4 low-P half zero scans

Idea: use cheap low-P scans mainly to identify zero-density half regions and mark them as misses.

- Source: `versions/Battleships_iter4_low_p_half_zero_scans_rejected_df57031e44b9.rs`
- Archive: `archives/20260624T164111Z_iter4_low_p_half_zero_scans_rejected_df57031e44b9`

Outcome: rejected. Early scan attempts often paid scan cost without feeding enough useful information back into the heatmap.

### iter4 hit-required heatmap

Idea: make chase/hunt scoring require consistency with active hit information more strongly.

- Source: `versions/Battleships_iter4_hit_required_heatmap_08fc5353f8b8.rs`
- Archive: `archives/20260624T164520Z_iter4_hit_required_heatmap_08fc5353f8b8`

Outcome: accepted. Better use of known hits improved targeting.

### iter5 single-hit damped chase

Idea: reduce overcommitment around a single hit. Single hits are weak orientation evidence, so chase should be damped until more structure appears.

- Source: `versions/Battleships_iter5_single_hit_damped_chase_2cbd7d9fea0c.rs`
- Archive: `archives/20260624T165311Z_iter5_single_hit_damped_chase_2cbd7d9fea0c`

Outcome: accepted. Damping single-hit chase was better than treating every hit as strong ship-orientation evidence.

### iter6 normalized hunt heatmap

Idea: normalize placement heatmap contributions by available legal placements per ship length.

- Source: `versions/Battleships_iter6_normalized_hunt_heatmap_e8e0bc1875ed.rs`
- Archive: `archives/20260624T165601Z_iter6_normalized_hunt_heatmap_e8e0bc1875ed`

Outcome: accepted. Normalization reduced bias toward lengths or regions with many raw placements.

### iter7 blended heatmap alpha 0.50

Idea: blend raw placement mass with normalized placement mass.

- Source: `versions/Battleships_iter7_blended_heatmap_a50_d5a44e0a1504.rs`
- Archive: `archives/20260624T170105Z_iter7_blended_heatmap_a50_d5a44e0a1504`

Outcome: accepted. A 50/50 blend became a stable default. Later alpha tuning failed to beat it.

### rejected iter8 Monte Carlo hunt overlay

Idea: add Monte Carlo posterior overlays on top of the deterministic heatmap.

- Source: `versions/Battleships_iter8_mc_hunt_overlay_rejected_0d658df3857a.rs`
- Archive: `archives/20260624T171011Z_iter8_mc_hunt_overlay_rejected_0d658df3857a`

Outcome: rejected. The added random sampling did not beat the deterministic placement heatmap enough to justify the noise/cost.

### iter8 tuned constants

Idea: tune heatmap/chase constants after iter7.

- Source: `versions/Battleships_iter8_tuned_constants_a60_sh50_cb0_6d9f52b6913b.rs`
- Archive: `archives/20260624T171759Z_iter8_tuned_constants_a60_sh50_cb0_6d9f52b6913b`

Outcome: accepted at the time. Later scan-aware work superseded it.

### rejected iter8 length-aware chase pruning

Idea: prune chase candidates that could not fit remaining ship lengths.

- Source: `versions/Battleships_iter8_length_aware_chase_pruning_rejected_581d3ee5c47a.rs`
- Archive: `archives/20260624T172829Z_iter8_length_aware_chase_pruning_rejected_581d3ee5c47a`

Outcome: rejected. Strict chase pruning was too brittle when hits from touching ships corrupted clusters.

### rejected iter8 single-hit axis scoring

Idea: give single-hit chase an axis-based score to pick a likely direction.

- Source: `versions/Battleships_iter8_single_hit_axis_scoring_rejected_9b68225ffeef.rs`
- Archive: `archives/20260625T005405Z_iter8_single_hit_axis_scoring_rejected_9b68225ffeef`

Outcome: rejected. Single-hit direction inference was too weak and often overfit noise.

### iter8 scan-density heatmap

Idea: keep scan results and use them as density information in hunt heatmap scoring.

- Source: `versions/Battleships_iter8_scan_density_heatmap_f41f82341238.rs`
- Archive: `archives/20260625T011916Z_iter8_scan_density_heatmap_f41f82341238`
- Validation: `benchmarks/validation_scan_density_vs_iter7_1_5000.tsv`

Outcome: accepted. This was the first scan implementation that clearly helped because scan results affected later shot probabilities instead of only marking zero regions.

### iter9 scan gate P <= 0.30

Idea: relax the scan gate so scans are used for more P values.

- Source: `versions/Battleships_iter9_scan_gate_p030_d1581732d025.rs`
- Archive: `archives/20260625T013510Z_iter9_scan_gate_p030_d1581732d025`
- Validation: `benchmarks/validation_iter9_scan_gate_p030.tsv`

Outcome: accepted. The earlier scan gate was too restrictive; scanning at moderately higher P still paid back.

### rejected iter10 quadrant density scans

Idea: add extra scans to derive cleaner quadrant counts.

- Source: `versions/Battleships_iter10_quadrant_density_scans_rejected_0eb7e8447508.rs`
- Archive: `archives/20260625T021540Z_iter10_quadrant_density_scans_rejected_0eb7e8447508`
- Validation: `benchmarks/validation_iter10_quadrant_density_scans.tsv`

Outcome: rejected. More exact quadrant data did not pay for the additional scan cost broadly enough.

### rejected iter10 scan density strength weighting half

Idea: adjust scan-density weighting strength.

- Source: `versions/Battleships_iter10_scan_density_strength_weighting_half_rejected_6257933941d8.rs`
- Archive: `archives/20260625T023040Z_iter10_scan_density_strength_weighting_half_rejected_6257933941d8`
- Validation: `benchmarks/validation_iter10_strength_weighting_half.tsv`

Outcome: rejected. The altered strength damaged calibration.

### iter10 scan threshold P <= 0.50

Idea: extend the scan policy to use scans up to `P <= 0.50`.

- Source: `versions/Battleships_iter10_scan_threshold_p050_d2fb414695bd.rs`
- Archive: `archives/20260625T025756Z_iter10_scan_threshold_p050_d2fb414695bd`

Outcome: accepted at the time. Later policy tuning refined this.

### iter11 scan threshold P <= 1.00

Idea: try scanning across the full P range.

- Source: `versions/Battleships_iter11_scan_threshold_p100_796110e48743.rs`
- Archive: `archives/20260625T030400Z_iter11_scan_threshold_p100_796110e48743`

Outcome: accepted as an experimental step but later superseded. Full-range scanning helped less reliably than conditional policies.

### iter12 conditional scan policy P <= 0.50

Idea: use scan policy conditional on P: heavier scanning at low P, simpler top/left scan policy otherwise.

- Source: `versions/Battleships_iter12_conditional_scan_policy_p050_a7e897ac68d0.rs`
- Archive: `archives/20260625T064242Z_iter12_conditional_scan_policy_p050_a7e897ac68d0`
- Validation: `benchmarks/validation_iter12_conditional_scan_policy_p050.tsv`

Outcome: accepted. Conditional scan scheduling was better than one fixed scan pattern everywhere.

### iter13 scan-weighted heatmap

Idea: make scan results affect placement scoring inside `build_probabilities()`, not only as a post-hoc cell multiplier.

- Source: `versions/Battleships_iter13_scan_weighted_heatmap_41c641453e88.rs`
- Archive: `archives/20260625T083738Z_iter13_scan_weighted_heatmap_41c641453e88`
- Validation: `benchmarks/validation_iter13_scan_weighted_heatmap.tsv`

Outcome: major accepted improvement. This confirmed that scan information is most useful when it changes placement weights before heatmap aggregation.

### rejected iter14 scan-aware chase

Idea: pass scan-aware probability maps into chase mode.

- Source: `versions/Battleships_rejected_iter14_scan_aware_chase_c94a65a0adae.rs`
- Archive: `archives/20260625T134419Z_rejected_iter14_scan_aware_chase_c94a65a0adae`
- Validation: `benchmarks/validation_iter14_scan_aware_chase.tsv`

Outcome: rejected. Chase mode became worse when scan priors were mixed in directly.

### rejected iter14b no second density scale

Idea: remove the second scan-density multiplier from `best_hunt_cell()` because scan weighting was already inside `build_probabilities()`.

- Source: `versions/Battleships_rejected_iter14b_no_second_density_scale_ed443fece560.rs`
- Archive: `archives/20260625T134457Z_rejected_iter14b_no_second_density_scale_ed443fece560`
- Validation: `benchmarks/validation_iter14b_no_second_density_scale.tsv`

Outcome: rejected. The second density scale still helped; scan placement weighting was more of a refinement than a replacement.

### rejected iter14c sqrt density scale

Idea: soften the second density scale with square root.

- Source: `versions/Battleships_rejected_iter14c_sqrt_density_scale_57554e204b5a.rs`
- Archive: `archives/20260625T134600Z_rejected_iter14c_sqrt_density_scale_57554e204b5a`
- Validation: `benchmarks/validation_iter14c_sqrt_density_scale.tsv`

Outcome: rejected. Softening the second scale hurt.

### rejected iter14d / iter14e scan weighting exponents

Idea: tune placement scan overlap exponents: overlap divided by sqrt(length), and sqrt(overlap).

- Sources:
  - `versions/Battleships_rejected_iter14d_scan_weight_overlap_sqrt_len_ced345b82b68.rs`
  - `versions/Battleships_rejected_iter14e_scan_weight_sqrt_overlap_cfba0b90152c.rs`
- Validations:
  - `benchmarks/validation_iter14d_scan_weight_overlap_sqrt_len.tsv`
  - `benchmarks/validation_iter14e_scan_weight_sqrt_overlap.tsv`

Outcome: rejected. The original softer `overlap / len` weighting was better overall.

### iter14f infer_killed_hits(n)

Idea: fix hardcoded board dimension in KILL inference so it uses `n` rather than `20`.

- Source: `versions/Battleships_iter14f_infer_killed_hits_n_b803257a430d.rs`
- Archive: `archives/20260625T092700Z_iter14f_infer_killed_hits_n_b803257a430d`
- Validation: `benchmarks/validation_iter14f_infer_killed_hits_n.tsv`

Outcome: accepted. Low-risk correctness fix.

### rejected iter15 length-aware KILL inference

Idea: infer killed ship length from remaining length distribution.

- Source: `versions/Battleships_rejected_iter15_length_aware_kill_inference_57a1626024f8.rs`
- Archive: `archives/20260625T134659Z_rejected_iter15_length_aware_kill_inference_57a1626024f8`
- Validation: `benchmarks/validation_iter15_length_aware_kill_inference.tsv`

Outcome: rejected. Length-aware inference overcommitted and caused major regressions.

### iter16 scan threshold P <= 0.25

Idea: retune conditional scan policy and lower threshold for four-quadrant scans.

- Source: `versions/Battleships_iter16_scan_threshold_p025_f1031a9ad5ac.rs`
- Archive: `archives/20260625T094628Z_iter16_scan_threshold_p025_f1031a9ad5ac`
- Tuning:
  - `benchmarks/tuning_iter16_scan_threshold_summary.tsv`
  - `benchmarks/tuning_iter16_scan_threshold_validation.tsv`

Outcome: accepted locally, but TC score was slightly worse than iter13 in one submit comparison. We kept the threshold because local validation favored it, but treated tiny threshold gains cautiously.

### iter17 placement-based KILL inference

Idea: exact placement-based KILL inference. On `KILL`, enumerate remaining ship lengths and placements through the final shot; accept a candidate only if every other cell in the placement is already an active hit. Mark only those active hits plus the final shot as dead. Fall back to longest segment if no exact placement exists.

- Source: `versions/Battleships_iter17_placement_based_kill_inference_41c343bbdea9.rs`
- Archive: `archives/20260625T095655Z_iter17_placement_based_kill_inference_41c343bbdea9`
- Validation: `benchmarks/validation_iter17_placement_based_kill_inference_1_10000.tsv`

Outcome: major accepted improvement. This is the current best.

## Later Rejected / Neutral Experiments After iter17

### rejected iter18 scored placement KILL inference

Idea: score multiple possible killed placements rather than using the exact placement inference as implemented in iter17.

- Source: `versions/Battleships_rejected_iter18_scored_placement_kill_inference_656292d88e84.rs`
- Archive: `archives/20260625T133618Z_rejected_iter18_scored_placement_kill_inference_656292d88e84`
- Validation: `benchmarks/validation_iter18_scored_placement_kill_inference.tsv`

Outcome: rejected. Added scoring worsened a good exact inference rule.

### rejected iter18 chase skip zero placement

Idea: skip chase candidates with zero placement support.

- Source: `versions/Battleships_rejected_iter18_chase_skip_zero_placement_e92e14c62944.rs`
- Archive: `archives/20260625T133645Z_rejected_iter18_chase_skip_zero_placement_e92e14c62944`
- Validation: `benchmarks/validation_iter18_chase_skip_zero_placement.tsv`

Outcome: rejected hard. Strictly filtering zero-support candidates was catastrophic because corrupted/touching hit clusters sometimes require escape moves.

### iter18 chase soft zero penalty neutral

Idea: penalize zero-support chase candidates instead of skipping them.

- Source: `versions/Battleships_iter18_chase_soft_zero_penalty_neutral_264aa0eeeb0d.rs`
- Archive: `archives/20260625T135748Z_iter18_chase_soft_zero_penalty_neutral_264aa0eeeb0d`
- Validation: `benchmarks/validation_iter18_chase_soft_zero_penalty.tsv`

Outcome: effectively neutral: `-5` over `1..5000`, all from `3001..5000`. Not promoted because the gain is noise-level and chase changes are risky.

### rejected heatmap alpha tuning

Idea: retune `HEATMAP_ALPHA` after iter17. Tested `0.35, 0.40, 0.45, 0.48, 0.52, 0.55, 0.58, 0.60, 0.62, 0.65` against current `0.50`.

- Tuning table: `benchmarks/tuning_iter17_heatmap_alpha.tsv`
- Sources: `versions/Battleships_rejected_heatmap_alpha_*.rs`

Outcome: rejected. `0.50` remained best over `1..5000`. Closest were `0.60` and `0.62`, but both won only on `1..1000` and lost later.

```text
alpha  total_delta_vs_iter17
0.35   +1934
0.40   +2251
0.45   +1547
0.48   +2117
0.52   +1253
0.55   +868
0.58   +1501
0.60   +323
0.62   +325
0.65   +549
```

### rejected iter18 endgame non-overlap search

Idea: in late game, enumerate non-overlapping remaining ship placements and build a probability map from consistent full-board placement sets.

- Source: `versions/Battleships_rejected_iter18_endgame_nonoverlap_search_d1a9b20579ab.rs`
- Archive: `archives/20260625T160254Z_rejected_iter18_endgame_nonoverlap_search_d1a9b20579ab`
- Validation: `benchmarks/validation_iter18_endgame_nonoverlap_search.tsv`

Outcome: rejected. It was worse on both tested ranges:

```text
1..1000     +213
1001..3000  +794
combined    +1007
```

Lesson: current `build_probabilities()` is already placement-based. Adding non-overlap board consistency late changed calibrated choices without enough benefit.

### rejected residual scan weighting

Idea: replace simple scan ratio weighting with residual overlap weighting: reward placements whose scan overlap is above expected in dense regions and below expected in sparse regions.

- Strong source: `versions/Battleships_rejected_iter18_residual_scan_weighting_strong_f758733f0afb.rs`
- Mild source: `versions/Battleships_rejected_iter18_residual_scan_weighting_mild_9c93e6f2872b.rs`
- Validation: `benchmarks/validation_iter18_residual_scan_weighting.tsv`

Outcome: rejected quickly on `1..1000`:

```text
strong  +789
mild    +1573
```

Lesson: the current scan weighting is crude but well-balanced. Re-centering around expected overlap damaged calibration.


### rejected iter18 KILL ambiguity common cells broad

Idea: when KILL inference had multiple plausible killed placements, keep up to four candidates and mark only active-hit cells common to all of them as dead.

- Source: `versions/Battleships_rejected_iter18_kill_ambiguity_common_cells_broad_81fdd374b5d7.rs`
- Archive: `archives/20260626T004555Z_rejected_iter18_kill_ambiguity_common_cells_broad_81fdd374b5d7`
- Diagnostic: `benchmarks/loss_diagnostics_iter18_kill_ambiguity_vs_iter17_1_1000.tsv`

Outcome: rejected. It was worse by `+911` on `1..1000`. The mistake was treating shorter valid placements as ambiguity peers, which left too many active hits uncommitted and damaged chase/hunt state.

### iter18 KILL ambiguity same-length candidates

Idea: preserve iter17 behavior when there is a single longest valid killed placement, but when multiple same-length longest placements are plausible, commit only the active-hit cells common to those same-length candidates. This reduces overcommitment in touching-ship ambiguity without letting shorter alternatives pollute the inference.

- Source: `versions/Battleships_iter18_kill_ambiguity_same_len_62877d07fd2e.rs`
- Archive: `archives/20260626T004555Z_iter18_kill_ambiguity_same_len_62877d07fd2e`
- Validation: `benchmarks/validation_iter18_kill_ambiguity_same_len.tsv`

Outcome: accepted. Full validation vs iter17:

```text
range       iter18          iter17          delta
1..1000     139469.033399   139876.033399   -407
1001..3000  281346.422478   281811.422478   -465
3001..5000  282197.279304   282439.279304   -242
5001..10000 718206.572380   719451.572380   -1245
combined    1421219.307561  1423578.307561  -2359
```


### rejected iter19 adaptive opening scan gate

Idea: replace fixed opening scans with a value gate: estimate scan value from expected zero-region elimination and density information, then scan only when estimated value exceeds scan cost. The goal was to skip expensive high-P scans that might quietly bleed score.

- Source: `versions/Battleships_rejected_iter19_adaptive_opening_scan_gate_27e69875a9ed.rs`
- Archive: `archives/20260626T005117Z_rejected_iter19_adaptive_opening_scan_gate_27e69875a9ed`
- Validation: `benchmarks/validation_iter19_adaptive_opening_scan_gate.tsv`

Outcome: rejected. The first range was a severe regression:

```text
1..1000  iter19 151053.071688  iter18 139469.033399  delta +11584.038289
```

Lesson: the existing fixed opening scans are doing more work than this simple expected-value gate captures. In particular, broad scan-density information remains valuable even when zero-region elimination is unlikely. Any future adaptive scan policy needs diagnostics around which scans were skipped and how that changed hunt misses.


### iter19 scan-feasibility hunt scoring

Idea: use scan constraints more exactly in hunt scoring. A second hunt-only placement map softly penalized placements that would make scan counts hard to satisfy after reserving that placement's cells. Zero-count scan overlap stayed a hard rejection. The map was blended into the normal hunt heatmap.

- Rejected 15% blend source: `versions/Battleships_rejected_iter19_scan_feasibility_hunt_blend15_209ebcaa54ec.rs`
- Near-neutral 5% blend source: `versions/Battleships_iter19_scan_feasibility_hunt_blend05_neutral_f2d7fb8fc564.rs`
- Validation: `benchmarks/validation_iter19_scan_feasibility_hunt.tsv`

Outcome: not promoted. The 15% blend was worse by `+232` over `1..5000`. The 5% blend was slightly positive over `1..10000`, but only by `-243` total and with mixed range results, so this is too small to trust as a new active version.

```text
variant  range        delta vs iter18
blend15  combined     +232 over 1..5000
blend05  1..1000      -658
blend05  1001..3000   +184
blend05  3001..5000   -406
blend05  5001..10000  +637
blend05  combined     -243
```

Lesson: exact-ish scan feasibility has a weak positive signal only at very low blend strength. It may be worth revisiting with diagnostics, but the current implementation is not strong enough to promote.


### rejected iter19 chase cluster soft priority

Idea: soften `best_chase_cell` cluster choice. The old rule used `cluster.len() * 100000`, making largest active cluster dominate even when clusters may be contaminated by touching ships. Two variants kept the local `chase_cell` support score and used smaller cluster bonuses.

- 1k source: `versions/Battleships_rejected_iter19_chase_soft_cluster_support1k_1a6f807ee0b1.rs`
- 10k source: `versions/Battleships_rejected_iter19_chase_soft_cluster_support10k_28f24eeb494c.rs`
- Validation: `benchmarks/validation_iter19_chase_soft_cluster_support.tsv`

Outcome: rejected. Both were worse over `1..5000`.

```text
variant  1..1000  1001..3000  3001..5000  combined
1k       -182     +190        +18         +26
10k      +2       +135        +9          +146
```

Lesson: the very strong largest-cluster priority is crude but still well-calibrated. Softer cluster choice helps a few seeds but loses the aggregate, likely because most multi-hit clusters really should be resolved before switching away.

## Scan Policy Sweeps

Several scan policy sweeps were run to avoid arbitrary thresholds:

- `benchmarks/tuning_iter10_scan_threshold/`
- `benchmarks/tuning_iter11_scan_threshold_high/`
- `benchmarks/tuning_iter12_scan_policy/`
- `benchmarks/tuning_iter16_scan_threshold_summary.tsv`
- `benchmarks/tuning_iter16_scan_threshold_validation.tsv`
- `benchmarks/tuning_iter17_scan_threshold_025_050.tsv`
- `benchmarks/tuning_iter17_scan_policy_full.tsv`

Important result from iter17 scan policy sweep:

- `threshold_0.225` was the best local score over `1..10000`, but only by about `366` vs `0.25`.
- `0.25` was kept because that margin is small enough to be overfit/noise.
- Four-quadrants-only and derived top-left/quadrant policies were worse.

Decision: do not chase tiny threshold wins unless a broader validation confirms them.

## Lessons Learned

1. Placement-compatible heatmaps are the core winning mechanism.
2. Exact KILL inference matters a lot because touching ships corrupt naive connected-cluster assumptions.
3. Scans help only when scan information updates the probability model. Scans that merely mark zero regions or add cost without heatmap integration regress.
4. The current scan weighting and second density multiplier are surprisingly well-calibrated. Attempts to remove, soften, or residualize them regressed.
5. Chase logic is fragile. Strict pruning or overusing scan priors in chase tends to hurt because hit clusters can be ambiguous or contaminated by touching ships.
6. Tiny threshold gains are suspect. We kept `P <= 0.25` rather than `0.225` because the observed advantage was too small to trust.
7. Checkerboard assumptions are bad as a general baseline here because ships of length 1 exist.
8. Full-board/non-overlap reasoning did not help as a late-game overlay, at least under the tested gates.

## Where To Look Before New Work

- Current best source: `versions/Battleships_iter17_placement_based_kill_inference_41c343bbdea9.rs`
- Active source: `Battleships.rs`
- Archive index: `versions/archive.tsv`
- Run index: `benchmarks/runs.tsv`
- Comparison benchmarks: `benchmarks/benchmarks.tsv`
- Major validation tables:
  - `benchmarks/validation_iter17_placement_based_kill_inference_1_10000.tsv`
  - `benchmarks/tuning_iter17_scan_policy_full.tsv`
  - `benchmarks/tuning_iter17_heatmap_alpha.tsv`
  - `benchmarks/validation_iter18_chase_soft_zero_penalty.tsv`
  - `benchmarks/validation_iter18_endgame_nonoverlap_search.tsv`
  - `benchmarks/validation_iter18_residual_scan_weighting.tsv`
  - `benchmarks/validation_iter17_vs_random_checkers_1_5000.tsv`

## Diagnostics Added

Loss diagnostics are documented in `LOSS_DIAGNOSTICS.md`. The initial workflow ranks per-seed losses from benchmark logs and writes TSVs with blank `classification` columns for later traced/manual labeling. Initial outputs include:

- `benchmarks/loss_diagnostics_iter17_vs_iter16_1_1000.tsv`
- `benchmarks/loss_diagnostics_iter17_vs_random_1_1000.tsv`
- `benchmarks/loss_diagnostics_iter17_vs_checkers_1_1000.tsv`

For iter17 vs iter16 on `1..1000`, iter17 still wins overall by `-7797`, but loses on 152 seeds. The largest losses are seeds `588`, `853`, `256`, `835`, `188`, `865`, `952`, `967`, `300`, `272`, and `22`.

## Suggested Next Directions

Prefer structural changes with broad justification over parameter or bucket fitting:

1. Better touched-ship modeling without strict chase filtering.
2. Safer KILL/active-hit bookkeeping that preserves ambiguity longer.
3. Conservative scan-aware reasoning that does not disturb the calibrated hunt score too much.
4. Diagnostics that explain where iter17 loses large margins before trying another policy change.

Avoid for now:

- More heatmap alpha sweeps around `0.50`.
- Small P-threshold tuning unless validated on a fresh large range.
- Strict chase candidate filtering.
- More scans without a clearly better way to consume the scan results.

## rejected_iter19_defer_weak_kill_leftovers

Reason: trace comparisons against iter13 and iter16 showed a repeated loss pattern after `KILL`: iter18 often kept a leftover active cluster and chased it immediately, while iter13/iter16 returned to hunt and avoided the loss.

Hypothesis: if a leftover post-KILL cluster had no legal placement support, delaying that cluster for one hunt turn would avoid bad chase overcommit while preserving iter18's same-length ambiguity bookkeeping.

Experiment: defer zero-support leftover clusters created after a `KILL` for one hunt turn, instead of immediately adding them back to `active_clusters`.

Result: rejected at the first gate.

```text
1..1000: iter19_defer_weak_kill_leftovers 139729.033399
1..1000: iter18_kill_ambiguity_same_len    139469.033399
delta: +260.000000
```

Archive:

- Source: `versions/Battleships_rejected_iter19_defer_weak_kill_leftovers_e7c0d5da2c9f.rs`
- Run: `benchmarks/20260626T032437Z_iter19_defer_weak_kill_leftovers`
- Archive manifest: `archives/20260626T032511Z_rejected_iter19_defer_weak_kill_leftovers_e7c0d5da2c9f/manifest.txt`

Inference: the trace pattern was real, but this intervention is too blunt. Some zero-support leftovers still need to be chased immediately, or the one-turn hunt delay loses more than it recovers.

## rejected_iter19_touching_leftover_defer_size1

Reason: the broader `rejected_iter19_defer_weak_kill_leftovers` trial showed that blindly delaying all zero-support post-`KILL` leftovers was too blunt, but trace diagnostics still showed a real failure mode where iter18 chased contaminated leftovers after a `KILL` while iter13/iter16 returned to hunt.

Hypothesis: restrict the delay to the most plausible touching-ship contamination case: a singleton leftover cluster adjacent to the committed killed cells, with zero legal placement support. This should suppress bad post-`KILL` overcommit without delaying useful multi-cell or supported chase contexts.

Experiment: after `KILL`, split leftover hits as usual, but defer only clusters with `len == 1`, adjacency to the committed killed cells, and zero continuation support. Deferred clusters are promoted after one hunt action.

Result: rejected after the second validation range.

```text
1..1000:    iter19 139466.033399  iter18 139469.033399  delta -3.000000
1001..3000: iter19 281439.422478  iter18 281346.422478  delta +93.000000
combined:   iter19 420905.455877  iter18 420815.455877  delta +90.000000
```

Archive:

- Source: `versions/Battleships_rejected_iter19_touching_leftover_defer_size1_44cbd2f29d25.rs`
- Validation: `benchmarks/validation_iter19_touching_leftover_defer_size1.tsv`
- Runs: `benchmarks/20260629T061042Z_iter19_touching_leftover_defer_size1`, `benchmarks/20260629T061118Z_iter19_touching_leftover_defer_size1`
- Archive manifest: `archives/20260629T061203Z_rejected_iter19_touching_leftover_defer_size1_44cbd2f29d25/manifest.txt`

Inference: the selective rule was less damaging than the broad defer policy, but still not beneficial. Even singleton adjacent zero-support leftovers sometimes contain useful chase signal, or the one-turn delay disrupts timing more than it prevents overcommit.

## rejected_iter19_particle_posterior

Reason: leaderboard movement suggested our independent-placement heatmap had saturated and was missing complete-board reasoning. The intended structural improvement was to estimate `P(cell occupied | all observations)` by sampling complete non-overlapping fleets, rather than counting each ship placement independently.

Hypothesis: a sampled complete-fleet posterior would use global constraints that the heatmap ignores: non-overlap between remaining ships, exact scan-count consistency, remaining ship inventory, and all known hits on one board.

Experiment: added bitset-backed single-ship placements and a randomized complete-fleet rejection sampler. Sampled boards had to avoid miss/dead cells, cover known `Hit` cells, avoid overlap, and satisfy scan rectangle counts including already-dead cells. The posterior occupancy was normalized and blended into `best_hunt_cell`.

Results:

```text
v1 broad posterior:
  1..1000:    iter19 139349.033399  iter18 139469.033399  delta -120.000000  seconds 126
  1001..3000: iter19 282029.422478  iter18 281346.422478  delta +683.000000  seconds 250
  combined:   delta +563.000000

v2 late only, remaining_cells <= 2N:
  1..1000:    iter19 138968.033399  iter18 139469.033399  delta -501.000000  seconds 78
  1001..3000: iter19 281907.422478  iter18 281346.422478  delta +561.000000  seconds 157
  combined:   delta +60.000000

v3 endgame only, remaining_cells <= N:
  1..1000:    iter19 139641.033399  iter18 139469.033399  delta +172.000000  seconds 62
```

Archive:

- `versions/Battleships_rejected_iter19_particle_posterior_v1_broad_2f7fef9fa6ab.rs`
- `versions/Battleships_rejected_iter19_particle_posterior_v2_late_c25938f7e699.rs`
- `versions/Battleships_rejected_iter19_particle_posterior_v3_endgame_dad42e2ab167.rs`
- Validation: `benchmarks/validation_iter19_particle_posterior.tsv`

Inference: complete-board posterior reasoning is still the most plausible missing model class, but this naive rejection sampler is not good enough. It is slow and unstable across ranges. The strong `1..1000` result for `v2` says the idea can help, but the `1001..3000` regression says the sampler/blend is too noisy or biased. A better version should use deterministic/beam enumeration of high-probability fleet states, or use the sampler only as diagnostics until it produces stable posterior estimates.

## iter20_precomputed_beam_posterior

Reason: leaderboard movement suggested the independent-placement heatmap was missing complete-board reasoning. We corrected the architecture to use a separate generated placement model rather than rebuilding placements at runtime.

Model artifact:

- Generator: `tools/generate_placement_model.py`
- Generated model: `placement_model_generated.rs`
- Model hash: `a3c9ce3e9f85c185f2207818d7d232bb9e15327e8070e7bb057615257c577df1`
- The development solver can use `include!("placement_model_generated.rs")`; final packaging should follow actual contest submission rules, not assumptions.

Experiments:

```text
precomputed_beam_v1:
  1..1000:    iter20 138435.033399  iter18 139469.033399  delta -1034.000000  seconds 233

precomputed_beam_dedup:
  1..1000:    iter20 138768.033399  iter18 139469.033399  delta -701.000000   seconds 236
  inference: exact occupied-bitset dedup was slower/no better and hurt state diversity.

beam_late_fallback:
  1..1000:    iter20 138263.033399  iter18 139469.033399  delta -1206.000000  seconds 240
  1001..3000: iter20 282452.422478  iter18 281346.422478  delta +1106.000000  seconds 469
  combined:   delta -100.000000
  inference: strong first-range signal but too mixed and too slow to accept.

beam_evidence_score:
  1..1000:    iter20 139111.033399  iter18 139469.033399  delta -358.000000   seconds 277
  inference: first evidence-consistency scoring formula regressed versus weak-prior beam.
```

Archives:

- `versions/Battleships_iter20_beam_late_fallback_mixed_baseline_a7e9309fbaf1.rs`
- `versions/Battleships_rejected_iter20_precomputed_beam_dedup_5f0b22e4ebab.rs`
- `versions/Battleships_rejected_iter20_beam_evidence_score_7239ba4b8c21.rs`
- Validation: `benchmarks/validation_iter20_precomputed_beam.tsv`

Inference: the precomputed complete-fleet beam is the first model-class change with a large local gain, but the current beam is unstable across ranges and very slow. The weak-prior beam outperformed the first evidence scorer, so the next work should not simply add penalties. It should improve the beam search mechanics: canonical handling of identical ships, state representation by placement indices instead of cloned bitsets, and a better future-feasibility bound before scoring tweaks.

## rejected_iter21_canonical_same_len_beam

Reason: iter20's precomputed complete-fleet beam showed a large first-range gain but was slow and unstable. One likely waste was generating equivalent permutations for identical ships of the same length.

Hypothesis: carry each placement's precomputed index in the beam state and, when placing another ship of the same length, only allow nondecreasing placement indices. This canonicalizes equal-length ship ordering and should reduce duplicate fleet prefixes without changing the intended fleet model.

Experiment: started from `versions/Battleships_iter20_beam_late_fallback_mixed_baseline_a7e9309fbaf1.rs`, added placement indices to `BeamPlacement`, tracked `last_len`/`last_index` in `BeamState`, and skipped same-length placements with lower indices.

Result: rejected after broader validation.

```text
1..1000:    iter21 138383.033399  iter18 139469.033399  delta -1086.000000  seconds 206
1001..3000: iter21 281983.422478  iter18 281346.422478  delta +637.000000   seconds 396
3001..5000: iter21 282736.279304  iter18 282197.279304  delta +539.000000   seconds 403
combined:   iter21 703102.735181  iter18 703012.735181  delta +90.000000
```

Archive:

- Candidate archive: `versions/Battleships_iter21_canonical_same_len_beam_candidate_4246f0ada2e2.rs`
- Rejected archive: `versions/Battleships_rejected_iter21_canonical_same_len_beam_4246f0ada2e2.rs`
- Validation: `benchmarks/validation_iter21_canonical_same_len_beam.tsv`

Inference: canonicalizing identical ships reduced runtime and improved the iter20 second-range regression, but the model still failed by `3001..5000`. The beam line still needs a better decision about when to trust the posterior, or a less biased state selection policy. Mechanics alone did not make it stable.

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
