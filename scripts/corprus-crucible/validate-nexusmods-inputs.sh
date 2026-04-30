#!/usr/bin/env bash
set -euo pipefail

nexus_api_key=${1:-}
nexus_group_id=${2:-}

if [ -z "$nexus_api_key" ] && [ -z "$nexus_group_id" ]; then
  exit 0
fi

if [ -z "$nexus_api_key" ]; then
  echo "::error::Nexus Mods upload requires NEXUS_API_KEY when nexus_group_id is set."
  exit 1
fi

if [ -z "$nexus_group_id" ]; then
  echo "::error::Nexus Mods upload requires nexus_group_id when NEXUS_API_KEY is set."
  exit 1
fi
