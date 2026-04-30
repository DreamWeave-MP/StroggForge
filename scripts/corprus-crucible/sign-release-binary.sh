#!/usr/bin/env bash
set -euo pipefail

release_binary=${1:?usage: sign-release-binary.sh <release-binary> <bundle-path>}
bundle_path=${2:?usage: sign-release-binary.sh <release-binary> <bundle-path>}

if [ ! -f "$release_binary" ]; then
  echo "::error::Release binary not found: $release_binary"
  exit 1
fi

cosign sign-blob -y "$release_binary" --bundle "$bundle_path"
