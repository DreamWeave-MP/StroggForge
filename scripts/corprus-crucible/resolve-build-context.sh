#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: resolve-build-context.sh <binary-name> [cargo-package]}
cargo_package=${2:-}

if [ -n "$cargo_package" ]; then
  # An explicit Cargo package (or `workspace`) builds from the repository root with package
  # selection, so the binary may live in any workspace member.
  build_dir=.
elif [ -d "$binary_name" ]; then
  # Auto-detect: if binary_name is a directory, build there.
  build_dir=$binary_name
else
  build_dir=.
fi

{
  echo "build_dir=$build_dir"
  echo "target_dir=.corprus-crucible/target"
  echo "dist_dir=.corprus-crucible/dist"
} >> "$GITHUB_OUTPUT"
