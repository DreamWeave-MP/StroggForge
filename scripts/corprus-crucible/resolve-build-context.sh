#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: resolve-build-context.sh <binary-name>}

# Auto-detect: if binary_name is a directory, build there.
if [ -d "$binary_name" ]; then
  build_dir=$binary_name
else
  build_dir=.
fi

{
  echo "build_dir=$build_dir"
  echo "target_dir=.corprus-crucible/target"
  echo "dist_dir=.corprus-crucible/dist"
} >> "$GITHUB_OUTPUT"
