#!/usr/bin/env bash
set -euo pipefail

generator=${1:-.stroggforge/.github/scripts/gen_benchmarks.py}

if [ ! -f "$generator" ]; then
  echo "error: benchmark documentation generator not found: $generator" >&2
  exit 1
fi

CARGO_TERM_COLOR=never cargo bench 2>&1 | tee benchmark-output.txt
python3 "$generator"
