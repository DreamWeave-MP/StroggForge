#!/usr/bin/env bash
set -euo pipefail

nexus_group_ids=${1:-}
runner_os=${2:?usage: resolve-nexusmods-group-id.sh <group-ids-json> <runner-os> <runner-arch> <release-name>}
runner_arch=${3:?usage: resolve-nexusmods-group-id.sh <group-ids-json> <runner-os> <runner-arch> <release-name>}
release_name=${4:?usage: resolve-nexusmods-group-id.sh <group-ids-json> <runner-os> <runner-arch> <release-name>}

if [ -z "$nexus_group_ids" ]; then
  {
    echo "group_id="
    echo "nexus_platform="
    echo "nexus_channel="
  } >> "$GITHUB_OUTPUT"
  exit 0
fi

python3 - "$nexus_group_ids" "$runner_os" "$runner_arch" "$release_name" "$GITHUB_OUTPUT" <<'PY'
import json
import sys

raw_group_ids, runner_os, runner_arch, release_name, github_output = sys.argv[1:]

os_names = {
    "Linux": "linux",
    "Windows": "windows",
    "macOS": "macos",
}

try:
    os_name = os_names[runner_os]
except KeyError:
    raise SystemExit(f"::error::Unsupported Nexus Mods runner OS: {runner_os}")

arch = runner_arch.lower()
platform = f"{os_name}-{arch}"
channel = "dev" if release_name == "development" else "stable"
key = f"{platform}-{channel}"

try:
    group_ids = json.loads(raw_group_ids)
except json.JSONDecodeError as exc:
    raise SystemExit(f"::error::NEXUS_GROUP_IDS is not valid JSON: {exc}")

if not isinstance(group_ids, dict):
    raise SystemExit("::error::NEXUS_GROUP_IDS must be a JSON object")

group_id = group_ids.get(key, "")

if group_id is None:
    group_id = ""

if group_id and not isinstance(group_id, (str, int)):
    raise SystemExit(f"::error::NEXUS_GROUP_IDS['{key}'] must be a string or integer")

if isinstance(group_id, int):
    group_id = str(group_id)

if not group_id and channel == "stable":
    raise SystemExit(f"::error::Missing required Nexus Mods group ID for '{key}'")

if not group_id:
    print(f"::notice::No Nexus Mods group ID configured for optional '{key}' upload; skipping Nexus Mods upload.")

with open(github_output, "a", encoding="utf-8") as output:
    output.write(f"group_id={group_id}\n")
    output.write(f"nexus_platform={platform}\n")
    output.write(f"nexus_channel={channel}\n")
PY
