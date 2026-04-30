#!/usr/bin/env bash
set -euo pipefail

binary_name=${1:?usage: prepare-nexusmods-archive.sh <binary-name> <runner-os> <runner-arch> <nexus-platform> <release-name>}
runner_os=${2:?usage: prepare-nexusmods-archive.sh <binary-name> <runner-os> <runner-arch> <nexus-platform> <release-name>}
runner_arch=${3:?usage: prepare-nexusmods-archive.sh <binary-name> <runner-os> <runner-arch> <nexus-platform> <release-name>}
nexus_platform=${4:?usage: prepare-nexusmods-archive.sh <binary-name> <runner-os> <runner-arch> <nexus-platform> <release-name>}
release_name=${5:?usage: prepare-nexusmods-archive.sh <binary-name> <runner-os> <runner-arch> <nexus-platform> <release-name>}

source_archive="${binary_name}-${runner_os}-${runner_arch}.zip"
safe_release_name=${release_name//\//-}
nexus_file_stem="${binary_name}-${nexus_platform}-${safe_release_name}"
nexus_archive="${nexus_file_stem}.zip"

if [ ! -f "$source_archive" ]; then
  echo "::error::Expected release archive at $source_archive"
  exit 1
fi

cp "$source_archive" "$nexus_archive"

{
  echo "nexus_archive=$nexus_archive"
  echo "nexus_file_stem=$nexus_file_stem"
} >> "$GITHUB_OUTPUT"
