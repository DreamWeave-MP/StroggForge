#!/usr/bin/env bash
set -euo pipefail

nexus_api_key=${1:-}
nexus_group_ids=${NEXUS_GROUP_IDS:-}

if [ -z "$nexus_api_key" ] && [ -z "$nexus_group_ids" ]; then
  exit 0
fi

if [ -z "$nexus_api_key" ]; then
  echo "::error::Nexus Mods upload requires NEXUS_API_KEY when NEXUS_GROUP_IDS is set."
  exit 1
fi

if [ -z "$nexus_group_ids" ]; then
  echo "::error::Nexus Mods upload requires NEXUS_GROUP_IDS when NEXUS_API_KEY is set."
  exit 1
fi
