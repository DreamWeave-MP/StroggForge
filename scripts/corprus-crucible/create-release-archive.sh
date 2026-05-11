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

resolve_include_file() {
  local file=$1
  local requested_path="./$build_dir/$file"
  local requested_dir requested_name search_dir candidate candidate_name
  local requested_name_lower candidate_name_lower

  if [ -f "$requested_path" ]; then
    printf '%s\n' "$requested_path"
    return 0
  fi

  requested_dir=$(dirname "$file")
  requested_name=$(basename "$file")
  requested_name_lower=$(printf '%s' "$requested_name" | tr '[:upper:]' '[:lower:]')

  if [ "$requested_dir" = . ]; then
    search_dir="./$build_dir"
  else
    search_dir="./$build_dir/$requested_dir"
  fi

  if [ ! -d "$search_dir" ]; then
    return 1
  fi

  for candidate in "$search_dir"/*; do
    [ -f "$candidate" ] || continue
    candidate_name=$(basename "$candidate")
    candidate_name_lower=$(printf '%s' "$candidate_name" | tr '[:upper:]' '[:lower:]')
    if [ "$candidate_name_lower" = "$requested_name_lower" ]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  return 1
}

if [ -n "$include_files" ]; then
  IFS=',' read -r -a user_files <<< "$include_files"
  for file in "${user_files[@]}"; do
    file=${file#"${file%%[![:space:]]*}"}
    file=${file%"${file##*[![:space:]]}"}
    if resolved_file=$(resolve_include_file "$file"); then
      include_name=$(basename "$resolved_file")
      include_name_lower=$(printf '%s' "$include_name" | tr '[:upper:]' '[:lower:]')
      case "$include_name_lower" in
        readme.md)
          include_name="$binary_name-README.md"
          ;;
        license)
          include_name="$binary_name-LICENSE"
          ;;
      esac
      cp "$resolved_file" "$dist_dir/$include_name"
      include_names+=("$include_name")
    else
      echo "::notice::Include file '$file' not found under $build_dir; skipping"
    fi
  done
fi

pushd "$dist_dir" >/dev/null
7z a -tzip "$archive_path" "${include_names[@]}"
popd >/dev/null
