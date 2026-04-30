#!/usr/bin/env bash
set -euo pipefail

target_repo=${1:?target repository is required}
aur_package_name=${2:?AUR package name is required}

repo_name=${GITHUB_EVENT_REPOSITORY_NAME:-${GITHUB_REPOSITORY#*/}}
version=${GITHUB_REF_NAME:?GITHUB_REF_NAME is required}
source_repo=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}
server_url=${GITHUB_SERVER_URL:-https://github.com}
actor=${GITHUB_ACTOR:?GITHUB_ACTOR is required}

issue_title="📦 Update ${repo_name} dependency to ${version}"
release_url="${server_url}/${source_repo}/releases/tag/${version}"

issue_body=$(cat <<EOF
## 🚨 Dependency Update Required: [${source_repo}](${server_url}/${source_repo})

**New Version:** [${version}](${release_url})
**Released By:** @${actor} **on** $(date -u +"%Y-%m-%d %H:%M UTC")
[${repo_name} on the AUR](https://aur.archlinux.org/packages/${aur_package_name})

### 📋 Required Actions
Please test integration with ${repo_name} version ${version}.
Close this issue via linked pull request when the integration is confirmed working.

*This issue was automatically created by GitHub Actions.*
EOF
)

issue_args=(
  --repo "$target_repo"
  --title "$issue_title"
  --body "$issue_body"
  --assignee "$actor"
)

if gh label list --repo "$target_repo" --json name --jq '.[] | select(.name == "enhancement") | .name' | grep -qxF enhancement; then
  issue_args+=(--label enhancement)
else
  echo "Label 'enhancement' does not exist in ${target_repo}; creating issue without labels."
fi

echo "Creating issue in ${target_repo}..."
gh issue create "${issue_args[@]}"
