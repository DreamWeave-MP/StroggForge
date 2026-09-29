#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch] [cargo-package]}
suffix=${2:-}
build_dir=${3:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch] [cargo-package]}
target_dir=${4:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch] [cargo-package]}
dist_dir=${5:?usage: build-release-binary.sh <binary-name> <suffix> <build-dir> <target-dir> <dist-dir> [rust-target] [platform-os] [platform-arch] [cargo-package]}
rust_target=${6:-}
platform_os=${7:-}
platform_arch=${8:-}
cargo_package=${9:-}

# setup-llvm pins CARGO_BUILD_TARGET on Windows, which moves the output under the triple too.
cargo_target=${rust_target:-${CARGO_BUILD_TARGET:-}}

built_binary_name="${binary_name}${suffix}"
if [ -n "$cargo_target" ]; then
  built_binary="$target_dir/$cargo_target/release/$built_binary_name"
else
  built_binary="$target_dir/release/$built_binary_name"
fi
release_binary="$dist_dir/$built_binary_name"
feature_args_file=$(mktemp)
trap 'rm -f "$feature_args_file"' EXIT
bash "$(dirname "$0")/feature-args.sh" "$platform_os" "$platform_arch" "$rust_target" "$binary_name" > "$feature_args_file"
# macOS runs bash 3.2: no mapfile, and an empty array is unbound under set -u.
feature_cargo_args=()
while IFS= read -r feature_arg; do
  feature_cargo_args+=("$feature_arg")
done < "$feature_args_file"
feature_cargo_arg_count=${#feature_cargo_args[@]}

mkdir -p "$target_dir" "$dist_dir"
rm -f "$built_binary"
cargo_build=(cargo build --release)
case "$cargo_package" in
  "")
    ;;
  workspace)
    # Any workspace member may provide the binary; --bin keeps the build to that one target.
    cargo_build+=(--workspace --bin "$binary_name")
    ;;
  *)
    cargo_build+=(--package "$cargo_package" --bin "$binary_name")
    ;;
esac
if [ "$feature_cargo_arg_count" -gt 0 ]; then
  cargo_build+=("${feature_cargo_args[@]}")
fi
if [ -n "$rust_target" ]; then
  CARGO_TARGET_DIR="$target_dir" "${cargo_build[@]}" --target "$rust_target" --manifest-path "$build_dir/Cargo.toml"
else
  CARGO_TARGET_DIR="$target_dir" "${cargo_build[@]}" --manifest-path "$build_dir/Cargo.toml"
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
