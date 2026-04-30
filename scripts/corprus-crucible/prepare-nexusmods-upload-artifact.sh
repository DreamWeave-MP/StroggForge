#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}
runner_os=${2:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}
runner_arch=${3:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}
group_id=${4:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}
nexus_archive=${5:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}
nexus_file_stem=${6:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}
release_name=${7:?usage: prepare-nexusmods-upload-artifact.sh <binary-name> <runner-os> <runner-arch> <group-id> <nexus-archive> <nexus-file-stem> <release-name>}

vt_analysis_bbcode=${VT_ANALYSIS_BBCODE:-}
artifact_name="nexus-${binary_name}-${runner_os}-${runner_arch}"
artifact_dir=".corprus-crucible/nexus-artifacts/${artifact_name}"
archive_existing_file=false

if [ "$release_name" = development ]; then
  archive_existing_file=true
fi

if [ ! -f "$nexus_archive" ]; then
  echo "::error::Expected Nexus archive at $nexus_archive"
  exit 1
fi

rm -rf "$artifact_dir"
mkdir -p "$artifact_dir"
cp "$nexus_archive" "$artifact_dir/"

{
  printf 'group_id=%q\n' "$group_id"
  printf 'filename=%q\n' "$(basename "$nexus_archive")"
  printf 'display_name=%q\n' "$nexus_file_stem"
  printf 'version=%q\n' "$release_name"
  printf 'archive_existing_file=%q\n' "$archive_existing_file"
} > "$artifact_dir/manifest.env"

{
  printf '%s release archive for %s-%s.\n\n' "$binary_name" "$runner_os" "$runner_arch"
  printf '%s\n' "$vt_analysis_bbcode"
} > "$artifact_dir/description.bbcode"

{
  echo "artifact_name=$artifact_name"
  echo "artifact_dir=$artifact_dir"
} >> "$GITHUB_OUTPUT"
