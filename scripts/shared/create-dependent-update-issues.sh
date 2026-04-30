#!/usr/bin/env bash
set -euo pipefail

dependent_repo_names=${1:-}
aur_package_name=${2:?AUR package name is required}
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

if [ -z "$dependent_repo_names" ] || [ "$dependent_repo_names" = '[]' ]; then
  echo "No dependent repositories configured."
  exit 0
fi

parsed_repos=$(python3 - "$dependent_repo_names" <<'PY'
import json
import sys

raw = sys.argv[1]
stripped = raw.strip()

if stripped.startswith("["):
    repos = json.loads(stripped)
    if not isinstance(repos, list) or not all(isinstance(repo, str) for repo in repos):
        raise SystemExit("dependent repository JSON must be an array of strings")
else:
    repos = stripped.splitlines()

for repo in repos:
    repo = repo.strip()
    if repo:
        print(repo)
PY
)

if [ -z "$parsed_repos" ]; then
  echo "No dependent repositories configured."
  exit 0
fi

mapfile -t target_repos <<< "$parsed_repos"

for target_repo in "${target_repos[@]}"; do
  bash "$script_dir/create-dependent-update-issue.sh" "$target_repo" "$aur_package_name"
done
