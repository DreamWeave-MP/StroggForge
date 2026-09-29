#!/usr/bin/env bash
# Record releases in mod.lock on the default branch and push the commit: the tagged release a
# ./buildSite release run left in dist/release.json, and any declared crate versions crates.io has.
# Another run may push first; each attempt starts again from the branch's newest commit.
set -euo pipefail

usage='usage: commit-records.sh <default-branch>'
default_branch=${1:?$usage}

git config user.name 'github-actions[bot]'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'

if [[ -f dist/release.json ]]; then
  message=$(jq -r '"RELEASE: \(.name) \(.release.version)"' dist/release.json)
else
  message="RELEASE: Record the crate versions published to crates.io"
fi

for attempt in 1 2 3 4 5; do
  git fetch --quiet origin "+refs/heads/${default_branch}:refs/remotes/origin/${default_branch}"
  git checkout --quiet --force --detach "origin/${default_branch}"
  if [[ -f dist/release.json ]]; then
    ./buildSite record
  fi
  ./buildSite record-crates
  git add --all -- '*mod.lock'
  if git diff --cached --quiet; then
    exit 0
  fi
  git commit --quiet -m "$message"
  if git push --quiet origin "HEAD:refs/heads/${default_branch}"; then
    exit 0
  fi
  sleep $((attempt * 5))
done

echo "::error::Could not push the release record to ${default_branch}. If the branch is protected, allow GitHub Actions to push to it."
exit 1
