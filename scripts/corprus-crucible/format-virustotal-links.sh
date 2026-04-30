#!/usr/bin/env bash
set -euo pipefail

display_filename=${1:?usage: format-virustotal-links.sh <display-filename> [analysis]}
analysis=${2:-}

if [ -z "$analysis" ]; then
  echo "vt_analysis=" >> "$GITHUB_OUTPUT"
  exit 0
fi

analysis_text=
IFS=',' read -r -a entries <<< "$analysis"

for entry in "${entries[@]}"; do
  IFS='=' read -r _file url <<< "$entry"
  analysis_text+="- [${display_filename} on VirusTotal](${url})"$'\n'
done

{
  echo "vt_analysis<<EOF"
  printf '%s' "$analysis_text"
  echo "EOF"
} >> "$GITHUB_OUTPUT"
