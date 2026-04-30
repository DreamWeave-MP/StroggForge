#!/usr/bin/env bash
set -euo pipefail

names_json=${1:?usage: docs-index.sh <names-json> [output-path]}
output=${2:-target/doc/index.html}

primary=$(jq -r '.[0]' <<< "$names_json" | tr '-' '_')

if [ -z "$primary" ] || [ "$primary" = "null" ]; then
  echo "error: names JSON must contain at least one entry" >&2
  exit 1
fi

mkdir -p "$(dirname "$output")"
printf '<meta http-equiv="refresh" content="0; url=%s/index.html">\n' "$primary" > "$output"
