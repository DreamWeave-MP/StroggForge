#!/usr/bin/env bash
set -euo pipefail

crate_name=${1:?usage: resolve-crate-manifest.sh <crate-name>}

manifest=$(
  cargo metadata --no-deps --format-version 1 |
    jq -r --arg name "$crate_name" '
      [
        .packages[]
        | select((.name | gsub("-"; "_")) == ($name | gsub("-"; "_")))
        | .manifest_path
      ][0] // empty
    '
)

if [ -z "$manifest" ] || [ "$manifest" = "null" ]; then
  echo "error: could not find Cargo manifest for crate '$crate_name'" >&2
  exit 1
fi

echo "$manifest"
