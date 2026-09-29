#!/usr/bin/env bash
# Lint each program exactly as its release is built for one platform: the same package or
# workspace selection, the same --bin, the platform's own features from cargo-build-args.sh, and
# the same --target. A test job lints with --all-features; only this catches code that is dead,
# or does not compile, in the feature set a platform actually ships.
set -euo pipefail

usage='usage: clippy-release.sh <binary-names-json> <cargo-package> <platform-os> <platform-arch> [rust-target] [target-dir]'
binary_names=${1:?$usage}
cargo_package=${2-}
platform_os=${3:?$usage}
platform_arch=${4:?$usage}
rust_target=${5:-}
target_dir=${6:-}
script_dir=$(dirname "$0")

# A JSON array of Cargo binary names, which cannot hold quotes or commas. Read without jq, which
# the builder containers lack, and without mapfile, which macOS's bash 3.2 lacks.
binaries=()
names=${binary_names#*[}
names=${names%]*}
IFS=, read -r -a entries <<<"$names"
for entry in ${entries[@]+"${entries[@]}"}; do
  entry=$(printf '%s' "$entry" | sed -e 's/^[[:space:]]*"//' -e 's/"[[:space:]]*$//')
  [ -n "$entry" ] || continue
  case "$entry" in
    *[!A-Za-z0-9_-]*)
      echo "::error::clippy-release.sh: '$entry' in $binary_names is not a Cargo binary name"
      exit 1
      ;;
  esac
  binaries+=("$entry")
done
if [ ${#binaries[@]} -eq 0 ]; then
  echo "::error::clippy-release.sh: no binary names in $binary_names"
  exit 1
fi

for binary_name in "${binaries[@]}"; do
  if [ -n "$cargo_package" ]; then
    build_dir=.
  elif [ -d "$binary_name" ]; then
    build_dir=$binary_name
  else
    build_dir=.
  fi

  feature_args_file=$(mktemp)
  bash "$script_dir/feature-args.sh" "$platform_os" "$platform_arch" "$rust_target" "$binary_name" > "$feature_args_file"
  # An empty array is unbound under set -u in bash 3.2.
  feature_args=()
  while IFS= read -r feature_arg; do
    feature_args+=("$feature_arg")
  done < "$feature_args_file"
  rm -f "$feature_args_file"

  clippy=(cargo clippy --manifest-path "$build_dir/Cargo.toml")
  case "$cargo_package" in
    "") clippy+=(--bin "$binary_name") ;;
    workspace) clippy+=(--workspace --bin "$binary_name") ;;
    *) clippy+=(--package "$cargo_package" --bin "$binary_name") ;;
  esac
  if [ ${#feature_args[@]} -gt 0 ]; then
    clippy+=("${feature_args[@]}")
  fi
  if [ -n "$rust_target" ]; then
    clippy+=(--target "$rust_target")
  fi
  clippy+=(-- -W clippy::pedantic -D warnings)

  echo "Linting $binary_name as released for $platform_os-$platform_arch: ${clippy[*]}"
  if [ -n "$target_dir" ]; then
    CARGO_TARGET_DIR="$target_dir" "${clippy[@]}"
  else
    "${clippy[@]}"
  fi
done
