#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./benchmark.sh run <label> <solution.rs> [seed_spec ...]
  ./benchmark.sh bench <label> <solution.rs> [seed_spec ...]
  ./benchmark.sh mark <run_dir> [benchmark_label]
  ./benchmark.sh compare <run_dir>
  ./benchmark.sh archive <version_label> <solution.rs> <run_dir>

Back-compatible shorthand:
  ./benchmark.sh <label> <solution.rs> [seed_spec ...]

Examples:
  ./benchmark.sh bench random versions/Battleships_v0_random.rs 1,100
  ./benchmark.sh run iter1 Battleships.rs
  ./benchmark.sh mark benchmarks/20260624T120000Z_iter1 iter1_reference
  ./benchmark.sh compare benchmarks/20260624T123000Z_iter2
  ./benchmark.sh archive iter1 Battleships.rs benchmarks/20260624T123000Z_iter1

If no seed_spec is provided, these are used:
  1,10 1,100 1,500 1,1000

State files:
  benchmarks/runs.tsv        all benchmark wrapper runs
  benchmarks/benchmarks.tsv  runs you marked as reference benchmarks
  versions/archive.tsv       archived source hashes and run dirs

Per-run outputs:
  benchmarks/<timestamp>_<label>/
    solution.rs
    binary
    metadata.txt
    run_<seed_spec>.log
    summary.tsv
    comparison.tsv
EOF
}

sanitize_label() {
  local label="$1"
  if [[ ! "$label" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "error: label may only contain letters, numbers, '.', '_', and '-'" >&2
    exit 2
  fi
}

init_state() {
  mkdir -p benchmarks
  if [[ ! -f benchmarks/runs.tsv ]]; then
    echo -e "run_dir\ttimestamp_utc\tlabel\tsolution\tseed_spec\tscore\tseconds\tlog" > benchmarks/runs.tsv
  fi
  if [[ ! -f benchmarks/benchmarks.tsv ]]; then
    echo -e "benchmark_label\trun_dir\ttimestamp_utc\tlabel\tsolution\tseed_spec\tscore\tseconds\tlog" > benchmarks/benchmarks.tsv
  fi
}

extract_run_timestamp() {
  local run_dir="$1"
  if [[ -f "$run_dir/metadata.txt" ]]; then
    awk -F': ' '$1 == "timestamp_utc" { print $2; found = 1; exit } END { if (!found) print "NA" }' "$run_dir/metadata.txt"
  else
    echo "NA"
  fi
}

mark_benchmark() {
  local run_dir="$1"
  local benchmark_label="${2:-}"

  init_state
  if [[ ! -f "$run_dir/summary.tsv" ]]; then
    echo "error: run summary not found: $run_dir/summary.tsv" >&2
    exit 2
  fi

  if [[ -z "$benchmark_label" ]]; then
    benchmark_label="$(basename "$run_dir")"
  fi
  sanitize_label "$benchmark_label"

  local timestamp
  timestamp="$(extract_run_timestamp "$run_dir")"

  awk -v bench="$benchmark_label" -v dir="$run_dir" -v ts="$timestamp" '
    BEGIN { FS = OFS = "\t" }
    NR == 1 { next }
    { print bench, dir, ts, $1, $2, $3, $4, $5, $6 }
  ' "$run_dir/summary.tsv" >> benchmarks/benchmarks.tsv

  echo "marked benchmark: $benchmark_label -> $run_dir"
}

compare_run() {
  local run_dir="$1"
  init_state

  local summary="$run_dir/summary.tsv"
  local comparison="$run_dir/comparison.tsv"

  if [[ ! -f "$summary" ]]; then
    echo "error: run summary not found: $summary" >&2
    exit 2
  fi

  {
    echo -e "seed_spec\tcurrent_label\tcurrent_score\tbenchmark_label\tbenchmark_score\tdelta_current_minus_benchmark\tratio_current_over_benchmark\tcurrent_log\tbenchmark_log"
    awk '
      BEGIN { FS = OFS = "\t" }
      FNR == NR {
        if (FNR > 1) {
          bench_count[$6]++
          i = bench_count[$6]
          bench_label[$6, i] = $1
          bench_score[$6, i] = $7
          bench_log[$6, i] = $9
        }
        next
      }
      FNR > 1 {
        seed = $3
        current_label = $1
        current_score = $4
        current_log = $6
        if (!(seed in bench_count)) {
          print seed, current_label, current_score, "NO_MATCHING_BENCHMARK", "NA", "NA", "NA", current_log, "NA"
          next
        }
        for (i = 1; i <= bench_count[seed]; i++) {
          bscore = bench_score[seed, i]
          delta = "NA"
          ratio = "NA"
          if (current_score ~ /^[-+]?[0-9]+([.][0-9]+)?$/ && bscore ~ /^[-+]?[0-9]+([.][0-9]+)?$/) {
            delta = current_score - bscore
            ratio = (bscore == 0 ? "NA" : current_score / bscore)
          }
          print seed, current_label, current_score, bench_label[seed, i], bscore, delta, ratio, current_log, bench_log[seed, i]
        }
      }
    ' benchmarks/benchmarks.tsv "$summary"
  } > "$comparison"

  echo "comparison: $comparison"
}

archive_version() {
  local version_label="$1"
  local solution="$2"
  local run_dir="$3"

  sanitize_label "$version_label"
  init_state

  if [[ ! -f "$solution" ]]; then
    echo "error: solution file not found: $solution" >&2
    exit 2
  fi
  if [[ ! -d "$run_dir" ]]; then
    echo "error: run directory not found: $run_dir" >&2
    exit 2
  fi
  if [[ ! -f "$run_dir/summary.tsv" ]]; then
    echo "error: run summary not found: $run_dir/summary.tsv" >&2
    exit 2
  fi

  mkdir -p versions archives

  local timestamp source_hash archived_source archive_dir manifest archive_index
  timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
  source_hash="$(sha256sum "$solution" | awk '{print $1}')"
  archived_source="versions/Battleships_${version_label}_${source_hash:0:12}.rs"
  archive_dir="archives/${timestamp}_${version_label}_${source_hash:0:12}"
  manifest="${archive_dir}/manifest.txt"
  archive_index="versions/archive.tsv"

  mkdir -p "$archive_dir"
  cp "$solution" "$archived_source"
  cp -R "$run_dir" "${archive_dir}/run"

  {
    echo "timestamp_utc: $timestamp"
    echo "version_label: $version_label"
    echo "solution: $solution"
    echo "source_sha256: $source_hash"
    echo "archived_source: $archived_source"
    echo "run_dir: $run_dir"
    echo "archived_run_dir: ${archive_dir}/run"
    echo "run_summary: ${archive_dir}/run/summary.tsv"
    echo "run_comparison: ${archive_dir}/run/comparison.tsv"
  } > "$manifest"

  if [[ ! -f "$archive_index" ]]; then
    echo -e "timestamp_utc\tversion_label\tsource_sha256\tarchived_source\trun_dir\tarchive_dir" > "$archive_index"
  fi
  echo -e "${timestamp}\t${version_label}\t${source_hash}\t${archived_source}\t${run_dir}\t${archive_dir}" >> "$archive_index"

  echo "archived source: $archived_source"
  echo "archive manifest: $manifest"
}

run_solution() {
  local mode="$1"
  local label="$2"
  local solution="$3"
  shift 3

  sanitize_label "$label"
  init_state

  if [[ ! -f "$solution" ]]; then
    echo "error: solution file not found: $solution" >&2
    exit 2
  fi

  local tester="visualizer/tester.jar"
  if [[ ! -f "$tester" ]]; then
    echo "error: tester jar not found: $tester" >&2
    exit 2
  fi

  local seeds
  if [[ $# -eq 0 ]]; then
    seeds=("1,10" "1,100" "1,500" "1,1000")
  else
    seeds=("$@")
  fi

  local timestamp out_dir binary summary metadata
  timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
  out_dir="benchmarks/${timestamp}_${label}"
  binary="${out_dir}/binary"
  summary="${out_dir}/summary.tsv"
  metadata="${out_dir}/metadata.txt"

  mkdir -p "$out_dir"
  cp "$solution" "${out_dir}/solution.rs"

  echo "label:    $label"
  echo "solution: $solution"
  echo "output:   $out_dir"
  echo

  rustc --edition=2024 -O "$solution" -o "$binary"

  {
    echo "timestamp_utc: $timestamp"
    echo "label: $label"
    echo "solution: $solution"
    echo "solution_sha256: $(sha256sum "$solution" | awk '{print $1}')"
    echo "tester: $tester"
    echo "tester_sha256: $(sha256sum "$tester" | awk '{print $1}')"
    echo "rustc: $(rustc --version)"
    echo "java: $(java -version 2>&1 | head -1)"
    echo "uname: $(uname -a)"
  } > "$metadata"

  echo -e "label\tsolution\tseed_spec\tscore\tseconds\tlog" > "$summary"

  for seed_spec in "${seeds[@]}"; do
    safe_seed="${seed_spec//[^A-Za-z0-9._-]/_}"
    log="${out_dir}/run_${safe_seed}.log"

    echo "running seed ${seed_spec}"
    start_s="$(date +%s)"
    java -jar "$tester" -exec "$binary" -seed "$seed_spec" -novis -printRuntime 5000 > "$log" 2>&1
    end_s="$(date +%s)"

    score="$(python3 - "$log" <<'PY_SCORE'
from pathlib import Path
import re
import sys
text = Path(sys.argv[1]).read_text(errors="replace")
values = [float(x) for x in re.findall(r'(?<![\w.])[-+]?\d+\.\d+(?![\w.])', text)]
print("NA" if not values else f"{sum(values):.6f}")
PY_SCORE
)"
    seconds="$((end_s - start_s))"

    echo -e "${label}\t${solution}\t${seed_spec}\t${score}\t${seconds}\t${log}" >> "$summary"
    echo -e "${out_dir}\t${timestamp}\t${label}\t${solution}\t${seed_spec}\t${score}\t${seconds}\t${log}" >> benchmarks/runs.tsv
  done

  if [[ "$mode" == "bench" ]]; then
    mark_benchmark "$out_dir" "$label"
  fi

  compare_run "$out_dir"

  echo
  echo "summary: $summary"
  echo "run_dir: $out_dir"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

cmd="$1"
shift

case "$cmd" in
  run|bench)
    if [[ $# -lt 2 ]]; then
      usage >&2
      exit 2
    fi
    run_solution "$cmd" "$@"
    ;;
  mark)
    if [[ $# -lt 1 || $# -gt 2 ]]; then
      usage >&2
      exit 2
    fi
    mark_benchmark "$@"
    ;;
  compare)
    if [[ $# -ne 1 ]]; then
      usage >&2
      exit 2
    fi
    compare_run "$1"
    ;;
  archive)
    if [[ $# -ne 3 ]]; then
      usage >&2
      exit 2
    fi
    archive_version "$@"
    ;;
  *)
    if [[ $# -lt 1 ]]; then
      usage >&2
      exit 2
    fi
    run_solution run "$cmd" "$@"
    ;;
esac
