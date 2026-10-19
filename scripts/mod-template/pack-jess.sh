#!/usr/bin/env bash
# Pack the Jess package a repository publishes, for the release the template is about to record.
# A `format = "jpk"` project's Package Source is the repository root; `jess pack` makes the
# archive and `jess package inspect` describes it, both into dist/jess, where dreamweave-release
# reads them. A repository without such a project packs nothing.
set -euo pipefail

if ! grep -qs '^format *= *"jpk"' content/*/mod.toml content/*/*/mod.toml; then
  echo "No project is a Jess package; nothing to pack."
  exit 0
fi

id=$(jess --format json package inspect . | jq -r '.id')
version=$(jess --format json package inspect . | jq -r '.version')
archive="dist/jess/${id}-${version}.jpk"
mkdir -p dist/jess
jess pack . --output "$archive"
jess --format json package inspect "$archive" > "$archive.json"
echo "Packed $archive"
