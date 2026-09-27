#!/usr/bin/env bash
set -euo pipefail

# Resolve the toolchain for the MSRV check. `auto` reads the highest `rust-version` declared by any
# workspace member, since one toolchain has to build all of them; anything else is used as given.
msrv=${1:?usage: resolve-msrv.sh <auto|toolchain>}

if [ "$msrv" = auto ]; then
  msrv=$(
    cargo metadata --no-deps --format-version 1 |
      jq -r '
        .workspace_members as $members
        | [.packages[] | select(.id as $id | $members | index($id)) | .rust_version // empty]
        | .[]
      ' |
      sort -V |
      tail -n 1
  )

  if [ -z "$msrv" ]; then
    echo "::error::msrv is 'auto' but no workspace member declares rust-version in Cargo.toml"
    exit 1
  fi
fi

if ! [[ "$msrv" =~ ^[0-9]+\.[0-9]+(\.[0-9]+)?$ ]]; then
  echo "::error::msrv must be 'auto' or a Rust version such as 1.88 or 1.88.0, got '$msrv'"
  exit 1
fi

echo "Checking against Rust $msrv"
echo "toolchain=$msrv" >> "${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"
