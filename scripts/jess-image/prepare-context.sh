#!/usr/bin/env bash
# Lays out the jess image's build context from a jess release's archives: each archive's binary
# under runtimes/, named for the archive (jess-Linux-X64, jess-Windows-X64.exe, ...), and the Linux
# archive's licenses and README under licenses/.
set -euo pipefail

usage='usage: prepare-context.sh <archive-dir> <context-dir>'
archives=${1:?$usage}
context=${2:?$usage}

mkdir -p "$context/runtimes" "$context/licenses"
shopt -s nullglob
found=0

for archive in "$archives"/jess-*.zip; do
  stem=$(basename "$archive" .zip)
  unpacked=$(mktemp -d)
  unzip -q "$archive" -d "$unpacked"

  if [[ -f "$unpacked/jess.exe" ]]; then
    cp "$unpacked/jess.exe" "$context/runtimes/$stem.exe"
  elif [[ -f "$unpacked/jess" ]]; then
    cp "$unpacked/jess" "$context/runtimes/$stem"
    chmod 755 "$context/runtimes/$stem"
  else
    echo "error: $archive holds no jess binary" >&2
    exit 1
  fi

  if [[ "$stem" == jess-Linux-X64 ]]; then
    cp "$unpacked"/LICENSE-* "$context/licenses/"
    cp "$unpacked/jess-README.md" "$context/licenses/README.md"
  fi

  rm -rf "$unpacked"
  found=$((found + 1))
done

if [[ ! -x "$context/runtimes/jess-Linux-X64" ]]; then
  echo "error: no jess-Linux-X64.zip among the $found archives in $archives; the image runs that one" >&2
  exit 1
fi

ls -l "$context/runtimes"
