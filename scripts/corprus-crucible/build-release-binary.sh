#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch]}
suffix=${2:-}
build_dir=${3:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch]}
target_dir=${4:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch]}
dist_dir=${5:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch]}
rust_target=${6:-}
platform_os=${7:-}
platform_arch=${8:-}

built_binary_name="${binary_name}${suffix}"
if [ -n "$rust_target" ]; then
  built_binary="$target_dir/$rust_target/release/$built_binary_name"
else
  built_binary="$target_dir/release/$built_binary_name"
fi
release_binary="$dist_dir/$built_binary_name"
cargo_args_hook=.stroggforge/cargo-build-args.sh
extra_cargo_args=()

if [ -f "$cargo_args_hook" ]; then
  if [ ! -x "$cargo_args_hook" ]; then
    echo "::error::$cargo_args_hook exists but is not executable"
    exit 1
  fi

  echo "Using Cargo build args from $cargo_args_hook"
  hook_output=$(mktemp)
  "$cargo_args_hook" "$platform_os" "$platform_arch" "$rust_target" "$binary_name" > "$hook_output"

  while IFS= read -r cargo_arg || [ -n "$cargo_arg" ]; do
    [ -n "$cargo_arg" ] || continue
    case "$cargo_arg" in
      --manifest-path|--manifest-path=*|--target|--target=*|--target-dir|--target-dir=*|--release|--bin|--bin=*)
        echo "::error::$cargo_args_hook emitted structural Cargo argument '$cargo_arg'; StroggForge owns build location, target, profile, and expected binary path"
        exit 1
        ;;
    esac
    extra_cargo_args+=("$cargo_arg")
  done < "$hook_output"
  rm -f "$hook_output"

  if [ "${#extra_cargo_args[@]}" -gt 0 ]; then
    printf 'Extra Cargo build args from %s:\n' "$cargo_args_hook"
    printf '  %s\n' "${extra_cargo_args[@]}"
  else
    echo "$cargo_args_hook emitted no extra Cargo build args"
  fi
fi

mkdir -p "$target_dir" "$dist_dir"
if [ -n "$rust_target" ]; then
  CARGO_TARGET_DIR="$target_dir" cargo build --release "${extra_cargo_args[@]}" --target "$rust_target" --manifest-path "$build_dir/Cargo.toml"
else
  CARGO_TARGET_DIR="$target_dir" cargo build --release "${extra_cargo_args[@]}" --manifest-path "$build_dir/Cargo.toml"
fi

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
