#!/usr/bin/env bash
# Record releases in mod.lock on the default branch and push the commit: the tagged release a
# dreamweave-release run left in dist/release.json, any declared crate versions crates.io has, and
# a program's tagged releases published to GitHub before the repository was a site.
# Another run may push first; each attempt starts again from the branch's newest commit.
set -euo pipefail

usage='usage: commit-records.sh <default-branch>'
default_branch=${1:?$usage}

git config user.name 'github-actions[bot]'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'

if [[ -f dist/release.json ]]; then
  message=$(jq -r '"RELEASE: \(.name) \(.release.version)"' dist/release.json)
else
  message="RELEASE: Record the releases published to crates.io and GitHub"
fi

for attempt in 1 2 3 4 5; do
  git fetch --quiet origin "+refs/heads/${default_branch}:refs/remotes/origin/${default_branch}"
  git checkout --quiet --force --detach "origin/${default_branch}"
  # The default branch's own pins: the template that records a release is the one its lock names.
  jess workspace sync --locked
  if [[ -f dist/release.json ]]; then
    jess workspace run --trusted dreamweave-record -- dist/release.json
  fi
  # Releases crates.io and GitHub already have are recorded by the Python tool until the template
  # package carries record-crates and record-releases; a site without ./buildSite records none.
  if [[ -x ./buildSite ]]; then
    ./buildSite record-crates
    if ./buildSite record-releases --help >/dev/null 2>&1; then
      ./buildSite record-releases
    fi
  fi
  # Listed rather than added by pattern: before a site's first release there is no mod.lock at
  # all, and git add fails on a pathspec that matches nothing.
  mapfile -t locks < <(git ls-files --modified --others --exclude-standard -- '*mod.lock')
  if (( ${#locks[@]} == 0 )); then
    exit 0
  fi
  git add -- "${locks[@]}"
  git commit --quiet -m "$message"
  if git push --quiet origin "HEAD:refs/heads/${default_branch}"; then
    exit 0
  fi
  sleep $((attempt * 5))
done

echo "::error::Could not push the release record to ${default_branch}. If the branch is protected, allow GitHub Actions to push to it."
exit 1
