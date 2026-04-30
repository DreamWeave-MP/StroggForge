#!/usr/bin/env bash
set -euo pipefail

runner_os=${1:?usage: binary-suffix.sh <runner-os>}

if [ "$runner_os" = "Windows" ]; then
  echo "suffix=.exe" >> "$GITHUB_OUTPUT"
  echo "Binary suffix: '.exe' (Windows detected)"
else
  echo "suffix=" >> "$GITHUB_OUTPUT"
  echo "Binary suffix: '' ($runner_os detected)"
fi
