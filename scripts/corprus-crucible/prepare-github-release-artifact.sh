#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: prepare-github-release-artifact.sh <binary-name> <runner-os> <runner-arch> <archive-path>}
runner_os=${2:?usage: prepare-github-release-artifact.sh <binary-name> <runner-os> <runner-arch> <archive-path>}
runner_arch=${3:?usage: prepare-github-release-artifact.sh <binary-name> <runner-os> <runner-arch> <archive-path>}
archive_path=${4:?usage: prepare-github-release-artifact.sh <binary-name> <runner-os> <runner-arch> <archive-path>}

artifact_name="github-release-${binary_name}-${runner_os}-${runner_arch}"
artifact_dir=".corprus-crucible/github-release-artifacts/$artifact_name"

mkdir -p "$artifact_dir"
cp "$archive_path" "$artifact_dir/"
printf '%s' "${VT_ANALYSIS:-}" > "$artifact_dir/${artifact_name}.vt.md"

{
  echo "artifact_name=$artifact_name"
  echo "artifact_dir=$artifact_dir"
} >> "$GITHUB_OUTPUT"
