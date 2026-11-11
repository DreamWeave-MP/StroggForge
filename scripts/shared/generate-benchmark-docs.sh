#!/usr/bin/env bash
set -euo pipefail

# BENCH_FILTER: passed to every bench target after `--`; empty runs every benchmark.
# BENCH_ALL_FEATURES: `true` benchmarks with every feature enabled.
generator=${1:-.stroggforge/.github/scripts/gen_benchmarks.py}

if [ ! -f "$generator" ]; then
  echo "error: benchmark documentation generator not found: $generator" >&2
  exit 1
fi

args=(bench)
if [ "${BENCH_ALL_FEATURES:-false}" = true ]; then
  args+=(--all-features)
fi
if [ -n "${BENCH_FILTER:-}" ]; then
  args+=(-- "$BENCH_FILTER")
fi

CARGO_TERM_COLOR=never cargo "${args[@]}" 2>&1 | tee benchmark-output.txt

# Windows runners name their interpreter python, not python3.
python=$(command -v python3 || command -v python)
"$python" "$generator"
