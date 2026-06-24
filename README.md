# Battleships Marathon Solver

This repo contains a Rust solver for the Topcoder Marathon Battleships problem, plus local tooling for comparing solver iterations over stable seed ranges.

## Important Files

- `Battleships.rs` - current working solver. Edit this for the next iteration.
- `versions/` - archived solver snapshots.
- `versions/archive.tsv` - index of archived versions, source hashes, and associated runs.
- `benchmark.sh` - compile, run, compare, and archive helper.
- `benchmarks/` - raw benchmark runs and comparison tables.
- `archives/` - immutable run archives tied to a source hash.
- `visualizer/tester.jar` - local Topcoder tester.
- `versions/Battleships_v0_random.rs` - random baseline.

## Benchmark Concepts

There are three useful run types:

- `run` - compile and benchmark a solver iteration.
- `bench` - run a solver and register it as a comparison benchmark.
- `archive` - snapshot a solver source file and copy its associated run artifacts under a content hash.

A lower score is better. The tester reports raw cost, and `benchmark.sh` aggregates per-seed scores into a total cost for each seed range.

## Basic Usage

Run the random baseline and register it as a benchmark:

```bash
./benchmark.sh bench random versions/Battleships_v0_random.rs 1,100
```

Run the current solver against the same seed range:

```bash
./benchmark.sh run current Battleships.rs 1,100
```

The current run will write:

```text
benchmarks/<timestamp>_current/summary.tsv
benchmarks/<timestamp>_current/comparison.tsv
```

`summary.tsv` records the aggregate score for the run. `comparison.tsv` compares that score against all registered benchmarks with the same seed spec.

## Standard Seed Sets

Use quick runs while developing:

```bash
./benchmark.sh run iter2 Battleships.rs 1,10
```

Use `1,100` for normal iteration comparisons:

```bash
./benchmark.sh run iter2 Battleships.rs 1,100
```

Use larger ranges before archiving a serious candidate:

```bash
./benchmark.sh run iter2 Battleships.rs 1,500
./benchmark.sh run iter2 Battleships.rs 1,1000
```

If no seed specs are provided, the wrapper runs all default ranges:

```text
1,10 1,100 1,500 1,1000
```

Example:

```bash
./benchmark.sh run iter2 Battleships.rs
```

## Marking Benchmarks

Any run can be marked as a long-term benchmark:

```bash
./benchmark.sh mark benchmarks/<timestamp>_iter2 iter2_reference
```

Registered benchmarks are stored in:

```text
benchmarks/benchmarks.tsv
```

All wrapper runs are stored in:

```text
benchmarks/runs.tsv
```

## Comparing Existing Runs

Regenerate a comparison table for an existing run:

```bash
./benchmark.sh compare benchmarks/<timestamp>_iter2
```

This writes or updates:

```text
benchmarks/<timestamp>_iter2/comparison.tsv
```

Important comparison columns:

- `current_score` - aggregate cost for the run being compared.
- `benchmark_score` - aggregate cost for a registered benchmark with the same seed spec.
- `delta_current_minus_benchmark` - negative means current is better.
- `ratio_current_over_benchmark` - below `1.0` means current is better.

## Archiving Iterations

Before starting the next improvement, archive the current source and the run you want tied to it:

```bash
./benchmark.sh archive iter1 Battleships.rs benchmarks/20260624T152544Z_current
```

This creates:

```text
versions/Battleships_iter1_<hash12>.rs
archives/<timestamp>_iter1_<hash12>/manifest.txt
archives/<timestamp>_iter1_<hash12>/run/summary.tsv
archives/<timestamp>_iter1_<hash12>/run/comparison.tsv
```

The full source hash and archive location are appended to:

```text
versions/archive.tsv
```

Current archived `iter1` reference:

```text
source_sha256: 8695d1295deeb4db135a04202ed86c6707d6ea530fee7f6b9f2aefe4620a0bbd
archived_source: versions/Battleships_iter1_8695d1295dee.rs
run_dir: benchmarks/20260624T152544Z_current
archive_dir: archives/20260624T153010Z_iter1_8695d1295dee
```

## Recommended Iteration Workflow

1. Edit `Battleships.rs`.
2. Smoke test on a small range:

```bash
./benchmark.sh run iter2_smoke Battleships.rs 1,10
```

3. Benchmark against registered references:

```bash
./benchmark.sh run iter2 Battleships.rs 1,100
```

4. Inspect:

```bash
cat benchmarks/<timestamp>_iter2/summary.tsv
cat benchmarks/<timestamp>_iter2/comparison.tsv
```

5. If the result is worth preserving, archive it:

```bash
./benchmark.sh archive iter2 Battleships.rs benchmarks/<timestamp>_iter2
```

6. Optionally mark it as a future benchmark:

```bash
./benchmark.sh mark benchmarks/<timestamp>_iter2 iter2_reference
```

## Manual Compile And Run

Compile a solver manually:

```bash
rustc --edition=2024 -O Battleships.rs -o /tmp/bs_current
```

Run one seed range manually:

```bash
java -jar visualizer/tester.jar -exec "/tmp/bs_current" -seed 1,100 -novis
```

Use `-novis` for benchmark runs so the tester does not try to open the Swing visualizer.
