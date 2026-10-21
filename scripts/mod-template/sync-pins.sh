#!/usr/bin/env bash
# Restore the template, theme and generator a site pins.
#
# With a jess.lock: exactly what it pins, from the publishers it names, and the lock is never
# rewritten here.
#
# Without one, the site is new. The template is taken from DREAMWEAVE_PUBLISHERS, the DreamWeave
# sites that publish it and everything it depends on, at the version jess-workspace.toml asks for,
# and on the default branch the lock that sync wrote is committed and pushed, so a site's first
# push needs no jess on anyone's machine.
#
# Before those publishers exist there is nothing to sync from: every DreamWeave package site's CI
# needs every other site live. The repository variable DREAMWEAVE_BOOTSTRAP_SOURCES breaks that
# circle once: a list of `owner/repo@ref` git sources, cloned here and offered to an offline sync in
# place of any publisher, with no lock written to the repository. Set it on each of the six sites,
# push, push the release tags, then delete it; the next push pins from the publishers for good.
set -euo pipefail

usage='usage: sync-pins.sh <default-branch> <publishing: true|false>'
default_branch=${1:?$usage}
publishing=${2:?$usage}
publishers=${DREAMWEAVE_PUBLISHERS:?DREAMWEAVE_PUBLISHERS names the DreamWeave package sites}
bootstrap=${DREAMWEAVE_BOOTSTRAP_SOURCES:-}

if [[ -f jess.lock ]]; then
  jess workspace sync --locked
  exit 0
fi

if [[ -n "$bootstrap" ]]; then
  echo "No jess.lock, and DREAMWEAVE_BOOTSTRAP_SOURCES is set: syncing offline from git sources, writing no lock to the repository."
  sources=()
  for source in $bootstrap; do
    repo=${source%@*}
    ref=${source#*@}
    [[ "$source" == *@* ]] || ref=main
    if [[ "${repo,,}" == "${GITHUB_REPOSITORY,,}" ]]; then
      continue # this repository is a member of its own workspace, not something offered to it
    fi
    folder="${RUNNER_TEMP:-/tmp}/dreamweave-bootstrap/${repo//\//__}"
    rm -rf "$folder"
    git clone --quiet --depth 1 --branch "$ref" "https://github.com/$repo" "$folder"
    sources+=("$folder")
  done
  jess workspace sync --offline "${sources[@]}"
  exit 0
fi

echo "No jess.lock yet: pinning the template from $publishers"
flags=()
for url in $publishers; do flags+=(--publisher "$url"); done
jess workspace sync "${flags[@]}"

if [[ "$publishing" != true ]]; then
  echo "Not on the default branch: the lock is used for this run and not committed."
  exit 0
fi

git config user.name 'github-actions[bot]'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'

for attempt in 1 2 3 4 5; do
  git fetch --quiet origin "+refs/heads/${default_branch}:refs/remotes/origin/${default_branch}"
  if ! git diff --quiet HEAD "origin/${default_branch}" -- jess-workspace.toml; then
    echo "::error::jess-workspace.toml changed on ${default_branch} since this run started; the next run writes the lock."
    exit 1
  fi
  git add jess.lock
  git commit --quiet -m 'FEAT: Pin the DreamWeave Mod Template, theme and generator this site builds with'
  if git push --quiet origin "HEAD:refs/heads/${default_branch}"; then
    exit 0
  fi
  git reset --quiet --soft HEAD~1
  sleep $((attempt * 5))
done

echo "::error::Could not push jess.lock to ${default_branch}. If the branch is protected, allow GitHub Actions to push to it, or run 'jess workspace sync' with --publisher for each of: ${publishers}, and commit jess.lock yourself."
exit 1
