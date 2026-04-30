#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir>}
suffix=${2:-}
build_dir=${3:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir>}
target_dir=${4:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir>}
dist_dir=${5:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir>}

built_binary_name="${binary_name}${suffix}"
built_binary="$target_dir/release/$built_binary_name"
release_binary="$dist_dir/$built_binary_name"

mkdir -p "$target_dir" "$dist_dir"
CARGO_TARGET_DIR="$target_dir" cargo build --release --manifest-path "$build_dir/Cargo.toml"

if [ ! -f "$built_binary" ]; then
  echo "::error::Expected built binary at $built_binary"
  exit 1
fi

cp "$built_binary" "$release_binary"
chmod +x "$release_binary" 2>/dev/null || true

{
  echo "binary_name=$built_binary_name"
  echo "release_binary=$release_binary"
} >> "$GITHUB_OUTPUT"
