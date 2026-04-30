#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: create-release-archive.sh <binary-name> <runner-os> <runner-arch> <built-binary-name> <build-dir> <dist-dir> [include-files]}
runner_os=${2:?usage: create-release-archive.sh <binary-name> <runner-os> <runner-arch> <built-binary-name> <build-dir> <dist-dir> [include-files]}
runner_arch=${3:?usage: create-release-archive.sh <binary-name> <runner-os> <runner-arch> <built-binary-name> <build-dir> <dist-dir> [include-files]}
built_binary_name=${4:?usage: create-release-archive.sh <binary-name> <runner-os> <runner-arch> <built-binary-name> <build-dir> <dist-dir> [include-files]}
build_dir=${5:?usage: create-release-archive.sh <binary-name> <runner-os> <runner-arch> <built-binary-name> <build-dir> <dist-dir> [include-files]}
dist_dir=${6:?usage: create-release-archive.sh <binary-name> <runner-os> <runner-arch> <built-binary-name> <build-dir> <dist-dir> [include-files]}
include_files=${7:-}

archive_name="${binary_name}-${runner_os}-${runner_arch}.zip"
archive_path="$PWD/$archive_name"
bundle_name="${binary_name}-${runner_os}-${runner_arch}.bundle"
include_names=("$built_binary_name" "$bundle_name")

if [ -n "$include_files" ]; then
  IFS=',' read -r -a user_files <<< "$include_files"
  for file in "${user_files[@]}"; do
    file=${file#"${file%%[![:space:]]*}"}
    file=${file%"${file##*[![:space:]]}"}
    if [ -f "./$build_dir/$file" ]; then
      include_name=$(basename "$file")
      cp "./$build_dir/$file" "$dist_dir/$include_name"
      include_names+=("$include_name")
    fi
  done
fi

pushd "$dist_dir" >/dev/null
7z a -tzip "$archive_path" "${include_names[@]}"
popd >/dev/null
