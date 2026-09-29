#!/usr/bin/env bash
# Package each PortMaster archive with the repository's muOS app files as a muOS app:
# <binary>-Portmaster-ARM64.zip becomes <binary>-Portmaster-ARM64.muxapp, a zip holding the
# PortMaster build with the app directory's files beside it.
set -euo pipefail

usage='usage: package-muxapp.sh <muxapp-dir> <archive-dir> <output-dir>'
muxapp_dir=${1:?$usage}
archive_dir=${2:?$usage}
output_dir=${3:?$usage}

if [[ ! -d "$muxapp_dir" ]]; then
  echo "::error::muxapp_dir '${muxapp_dir}' is not a directory in this repository."
  exit 1
fi

shopt -s nullglob
archives=("$archive_dir"/*-Portmaster-ARM64.zip)
shopt -u nullglob

if (( ${#archives[@]} == 0 )); then
  echo "::error::No PortMaster archives in ${archive_dir}. muxapp_dir packages the PortMaster build, so it needs enable_portmaster."
  exit 1
fi

mkdir -p "$output_dir"
output_dir=$(cd "$output_dir" && pwd)

for archive in "${archives[@]}"; do
  name=$(basename "$archive" .zip)
  work=$(mktemp -d)
  unzip -q "$archive" -d "$work/app"
  cp -a "$muxapp_dir/." "$work/app/"
  find "$work/app" -maxdepth 1 -name '*.sh' -exec chmod +x {} +
  (cd "$work/app" && zip -qr -X "$output_dir/${name}.muxapp" .)
  rm -rf "$work"
  echo "Packaged ${output_dir}/${name}.muxapp"
done
