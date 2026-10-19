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
name="${id}-${version}.jpk"
# jess refuses to write an archive inside the Package Source it packs, so it is packed outside the
# repository and moved into dist/jess, where dreamweave-release reads it.
staging=$(mktemp -d)
jess pack . --output "$staging/$name"
mkdir -p dist/jess
mv "$staging/$name" "dist/jess/$name"
rmdir "$staging"
jess --format json package inspect "dist/jess/$name" > "dist/jess/$name.json"
echo "Packed dist/jess/$name"
