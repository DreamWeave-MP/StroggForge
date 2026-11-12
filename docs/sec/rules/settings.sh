#!/usr/bin/env bash
# SEC-101's settings invariants (PR-01 and its neighbors), as read-only assertions over a GitHub
# repository's live settings. It changes nothing: every call is a GET. A planning prototype: it
# reports, and nothing in CI runs it. Exit 0 when every assertion holds, 1 when any fails.
#
#   docs/sec/rules/settings.sh DreamWeave-MP/StroggForge
set -euo pipefail

repo=${1:?usage: settings.sh <owner/repo>}
failures=0

check() {
  local id=$1 threat=$2 summary=$3 actual=$4 expected=$5
  if [[ "$actual" == "$expected" ]]; then
    printf 'ok    %s %s %s\n' "$id" "$threat" "$summary"
  else
    printf 'FAIL  %s %s %s (is %s, wants %s)\n' "$id" "$threat" "$summary" "$actual" "$expected"
    failures=$((failures + 1))
  fi
}

# A tag ruleset that refuses moving or deleting release tags: what makes `@vN` a fixed reference.
tag_rules=$(gh api "repos/$repo/rulesets" --jq '[.[] | select(.target == "tag")] | length')
check ST-01 T-TAG 'a tag ruleset exists' "$([[ "$tag_rules" -gt 0 ]] && echo yes || echo no)" yes

check ST-02 T-REL 'releases are immutable' \
  "$(gh api "repos/$repo/immutable-releases" --jq '.enabled')" true

permissions=$(gh api "repos/$repo/actions/permissions")
check ST-03 T-ACT 'actions must be pinned by full SHA' \
  "$(jq -r '.sha_pinning_required // false' <<<"$permissions")" true

check ST-04 T-CRED 'the workflow token defaults to read' \
  "$(gh api "repos/$repo/actions/permissions/workflow" --jq '.default_workflow_permissions')" read

analysis=$(gh api "repos/$repo" --jq '.security_and_analysis')
check ST-05 T-CRED 'secret scanning is on' "$(jq -r '.secret_scanning.status' <<<"$analysis")" enabled
check ST-06 T-CRED 'push protection is on' "$(jq -r '.secret_scanning_push_protection.status' <<<"$analysis")" enabled

branch=$(gh api "repos/$repo" --jq '.default_branch')
protected=$(gh api "repos/$repo/rules/branches/$branch" --jq 'length')
check ST-07 T-TAG "the default branch ($branch) has rules" "$([[ "$protected" -gt 0 ]] && echo yes || echo no)" yes

exit $(( failures > 0 ))
