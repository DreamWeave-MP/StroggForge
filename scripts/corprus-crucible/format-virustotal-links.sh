#!/usr/bin/env bash
set -euo pipefail

display_filename=${1:?usage: format-virustotal-links.sh <display-filename> [analysis]}
analysis=${2:-}

if [ -z "$analysis" ]; then
  echo "vt_analysis=" >> "$GITHUB_OUTPUT"
  echo "vt_analysis_bbcode=" >> "$GITHUB_OUTPUT"
  exit 0
fi

analysis_text=
analysis_bbcode=
IFS=',' read -r -a entries <<< "$analysis"

for entry in "${entries[@]}"; do
  IFS='=' read -r _file url <<< "$entry"
  analysis_text+="- [${display_filename} on VirusTotal](${url})"$'\n'
  analysis_bbcode+="[LIST][*][URL=${url}]${display_filename} on VirusTotal[/URL][/LIST]"$'\n'
done

{
  echo "vt_analysis<<EOF"
  printf '%s' "$analysis_text"
  echo "EOF"
  echo "vt_analysis_bbcode<<EOF"
  printf '%s' "$analysis_bbcode"
  echo "EOF"
} >> "$GITHUB_OUTPUT"
