#!/usr/bin/env bash
set -euo pipefail

vt_api_key=${1:-}

if [ -z "$vt_api_key" ]; then
  echo "::error::VirusTotal API key is required!"
  echo "Please provide vt_api_key input to this action"
  exit 1
fi

echo "✅ VirusTotal API key is set"
