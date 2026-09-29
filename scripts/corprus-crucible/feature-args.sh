#!/usr/bin/env bash
# Print the Cargo feature arguments a platform's release is built with, one per line: whatever the
# repository's .stroggforge/cargo-build-args.sh emits for the platform tuple, checked to be feature
# arguments only. Nothing when the repository has no hook. Diagnostics go to stderr.
set -euo pipefail

usage='usage: feature-args.sh <platform-os> <platform-arch> <rust-target> <binary-name>'
platform_os=${1:?$usage}
platform_arch=${2:?$usage}
rust_target=${3:-}
binary_name=${4:?$usage}
cargo_args_hook=.stroggforge/cargo-build-args.sh

case "$platform_arch" in
  X64) platform_arch=x64 ;;
  X86) platform_arch=x86 ;;
esac

if [ ! -f "$cargo_args_hook" ]; then
  exit 0
fi
if [ ! -x "$cargo_args_hook" ]; then
  echo "::error::$cargo_args_hook exists but is not executable" >&2
  exit 1
fi

hook_output=$(mktemp)
trap 'rm -f "$hook_output"' EXIT
"$cargo_args_hook" "$platform_os" "$platform_arch" "$rust_target" "$binary_name" > "$hook_output"

args=()
expecting_features_value=false
while IFS= read -r cargo_arg || [ -n "$cargo_arg" ]; do
  [ -n "$cargo_arg" ] || continue
  if [ "$expecting_features_value" = true ]; then
    args+=("$cargo_arg")
    expecting_features_value=false
    continue
  fi
  case "$cargo_arg" in
    --features|-F)
      args+=("$cargo_arg")
      expecting_features_value=true
      ;;
    --features=*|-F=*|--no-default-features|--all-features)
      args+=("$cargo_arg")
      ;;
    *)
      echo "::error::$cargo_args_hook emitted non-feature Cargo argument '$cargo_arg'; only --features, -F, --no-default-features, and --all-features are allowed" >&2
      exit 1
      ;;
  esac
done < "$hook_output"

if [ "$expecting_features_value" = true ]; then
  echo "::error::$cargo_args_hook ended after --features/-F without a feature list" >&2
  exit 1
fi

if [ ${#args[@]} -gt 0 ]; then
  printf 'Cargo feature args from %s for %s-%s:\n' "$cargo_args_hook" "$platform_os" "$platform_arch" >&2
  printf '  %s\n' "${args[@]}" >&2
  printf '%s\n' "${args[@]}"
else
  echo "$cargo_args_hook emitted no Cargo feature args for $platform_os-$platform_arch" >&2
fi
