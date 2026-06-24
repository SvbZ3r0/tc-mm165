# Battleships Solver Plan

## Goal

Build a competitive Rust solver incrementally, always keeping a runnable baseline so each new idea can be compared on:

- raw score
- runtime per seed
- failure rate / invalid actions
- implementation complexity

The final target is a hybrid posterior solver:

1. Maintain candidate ship placements.
2. Maintain Monte Carlo samples of complete valid boards.
3. Use scan results as exact constraints.
4. Choose between `SCAN` and `SHOOT` by expected information versus cost.
5. Switch to greedy local shooting when there are known unresolved hits or when the heatmap is sharp.

## Core Problem Observations

The grid is small: `N` is only `8..20`, but there are many ships. Ships can touch, so a killed ship does not imply neighboring cells are empty.

Shots cost exactly `1`. Scans cost:

```text
P * log2(N*N + 2 - area)
```

This makes large scans relatively cheap and tiny scans often bad. For `N=20`, an area-1 scan costs about:

```text
P * log2(401) ~= 8.65P
```

So a tiny scan can cost almost nine shots when `P` is high. We should only use small scans when `P` is very low and the answer changes many future decisions.

## Final Solver Architecture

### State

Track:

- grid cell state: unknown, miss, unresolved hit, killed/confirmed occupied
- whether a cell has already been shot
- remaining ship counts by length
- unresolved hit clusters
- scan constraints: rectangle plus exact count
- optional board samples representing plausible complete worlds

### Candidate Placements

For each remaining ship length, generate all horizontal and vertical placements that:

- stay inside the grid
- avoid known misses
- avoid already resolved killed cells when modeling only remaining ships
- are compatible with unresolved hits when required
- do not overlap other ships in sampled complete boards

This gives a basic placement heatmap even before Monte Carlo is introduced.

### Monte Carlo Posterior

Maintain a pool of complete valid board samples. Each sample places all remaining ships without overlap and satisfies known evidence:

- misses are empty
- unresolved hits are occupied
- killed cells are already accounted for
- scan counts match previous `SCAN` results

From the samples:

```text
cell_probability = samples_with_ship_at_cell / sample_count
```

For each candidate scan rectangle:

```text
distribution[count] = number of samples with that count in rectangle
entropy = -sum(p * log2(p))
```

This directly estimates scan information value.

### Action Policy

If there is an unresolved hit:

```text
target mode:
    shoot adjacent/aligned posterior-best cell
```

Otherwise:

```text
if posterior is sharp:
    shoot max-probability cell
else:
    evaluate scan candidates
    scan only if information/cost beats threshold for current P
```

### P-Based Scan Policy

For `P <= 0.25`:

- scan aggressively
- use recursive partitioning
- scan halves, quadrants, bands, and high-mass regions
- shoot when a cell becomes very likely

For `0.25 < P <= 0.6`:

- balanced mode
- scan only when it splits uncertainty well
- prefer large rectangles with high entropy
- shoot high-probability cells sooner

For `P > 0.6`:

- mostly shoot
- consider one or two large early scans if uncertainty is diffuse
- avoid small/local scans
- resolve hits by shooting, not scanning

## Candidate Scan Rectangles

Do not start by evaluating every rectangle every turn. Generate a focused candidate set:

- horizontal halves
- vertical halves
- quadrants
- thirds
- wide row bands
- wide column bands
- prefix rectangles
- rectangles around top heatmap clusters
- rectangles that split high-probability mass roughly in half
- occasional rectangles around unresolved ambiguity, but not tiny ones

Later, if runtime allows, test broader rectangle enumeration for `N <= 20`.

## Incremental Build Plan

Each version should be preserved or reproducible by a small config flag so we can benchmark regressions.

### Version 0: Random Shooter

Purpose:

- establish tester wiring
- confirm Rust compile/run flow
- measure baseline runtime
- measure random raw score distribution

Behavior:

```text
while ships remain:
    shoot a random unshot cell
```

Expected:

- always valid
- poor score
- very fast

Benchmark:

- seeds `1..10`
- then `1..100`
- record average score, median score, max runtime

### Version 1: Static Pattern Shooter

Purpose:

- beat random with almost no state complexity

Behavior:

- shoot a parity/checkerboard or length-aware pattern first
- then fill remaining cells
- when hit, still keep simple behavior or immediately chase adjacent cells

Expected:

- better than random
- still very fast

### Version 2: Hit-Chase Mode

Purpose:

- avoid wasting shots after discovering a ship

Behavior:

```text
if unresolved hit exists:
    if multiple aligned hits:
        shoot at either end
    else:
        shoot best unshot neighbor
else:
    use current hunt policy
```

Important:

- resolve active hits before returning to hunting
- this reduces `KILL` ambiguity

Expected:

- large improvement over random/pattern shooting
- minimal runtime cost

### Version 3: Placement Heatmap

Purpose:

- use ship length distribution and known misses

Behavior:

- generate legal placements for remaining ship lengths
- score each unknown cell by how many weighted placements cover it
- shoot max-score cell when not in target mode

Expected:

- better hunt-phase shots
- still deterministic and simple

Benchmark:

- compare average raw score versus Version 2
- confirm heatmap rebuild is cheap enough for worst-case `N=20`

### Version 4: Conservative Ship Count Updates

Purpose:

- improve heatmap quality as ships are killed

Behavior:

- decrement remaining ship count on `KILL`
- infer killed length from active hit chain when possible
- if ambiguous, decrement nearest plausible remaining length

Risk:

- touching ships can make length inference wrong

Mitigation:

- keep target mode focused on one active cluster
- avoid treating neighboring unknown cells as empty after a kill

### Version 5: Opening Scans

Purpose:

- test whether scans help before implementing full entropy logic

Behavior:

- for low and medium `P`, run a small fixed scan schedule early:
  - horizontal half
  - vertical half
  - quadrants or thirds
- store scan constraints
- initially use scan results only for coarse region prioritization

Expected:

- useful when `P` is low
- may hurt when `P` is high

Benchmark:

- split results by `P` bucket
- verify high-`P` mode does not regress

### Version 6: Scan-Constrained Heatmap

Purpose:

- make scans actually affect posterior probabilities

Behavior:

- keep all scan constraints
- downweight placements or sampled boards that disagree with scan counts
- simplest first approximation:
  - score regions with positive remaining scan mass higher
  - avoid regions proven empty by scan

Expected:

- low-`P` and medium-`P` improvement

### Version 7: Monte Carlo Complete Boards

Purpose:

- replace independent placement heatmap with a real posterior estimate

Behavior:

- repeatedly sample complete boards from remaining ship inventory
- reject samples violating known misses, hits, and scan constraints
- compute cell probabilities from accepted samples

Implementation notes:

- cap sample generation by time or attempt count
- reuse previous samples when possible
- if too few valid samples survive, fall back to placement heatmap

Expected:

- sharper probabilities
- better scan scoring
- more runtime pressure

### Version 8: Entropy-Based Scan Selection

Purpose:

- choose scans only when they are worth their cost

Behavior:

For each candidate rectangle:

```text
cost = P * log2(N*N + 2 - area)
entropy = entropy of count distribution over samples
scan_score = entropy / cost
```

Policy:

- shoot immediately if max cell probability is high
- otherwise scan if best scan score clears a threshold
- thresholds depend on `P`

Expected:

- strongest low-`P` performance
- controlled medium-`P` usage
- almost no high-`P` scan waste

### Version 9: Tuning and Regression Harness

Purpose:

- tune thresholds empirically
- prevent overfitting to a few seeds

Benchmark groups:

- examples: `1..10`
- quick provisional proxy: `1..100`
- wider local set: `1..500` or `1..1000` when time allows
- buckets by `N`, `S`, and `P`

Metrics:

- average raw score
- median raw score
- 90th percentile raw score
- max runtime
- invalid/failure count
- average number of scans
- average number of shots

## Benchmark Discipline

Every meaningful change should answer:

1. Did average score improve?
2. Did worst-case runtime stay safe?
3. Did any seed become invalid?
4. Which `P` bucket improved or regressed?
5. Is the extra complexity justified?

Use small seed ranges while developing and larger ranges before keeping a strategy.

Suggested local commands once Rust is installed:

```powershell
rustc --edition=2024 -O Battleships.rs -o Battleships.exe
java -jar visualizer/tester.jar -exec ".\Battleships.exe" -seed 1 -novis -noimages
```

For batches, use the marathon local tester options from `mm-local-tester-parameters.md`, or add a small script that runs many seeds and extracts final scores.

## First Concrete Milestone

The first milestone is intentionally modest:

1. Implement Version 0 random shooter in Rust.
2. Compile locally.
3. Run seeds `1..10`.
4. Record baseline score and runtime.
5. Add Version 2 hit-chase mode.
6. Compare against baseline.

Only after that should we add heatmaps and scans.

