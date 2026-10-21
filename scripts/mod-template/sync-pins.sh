#!/usr/bin/env bash
# Restore the template, theme and generator a site pins. With a jess.lock, exactly what it pins,
# from the publishers it names, and the lock is never rewritten. Without one, the site is new: the
# template is taken from its publisher at the version jess-workspace.toml asks for, and on the
# default branch the lock that sync wrote is committed and pushed, so a site's first push needs no
# jess on anyone's machine. Another run may push first; each attempt starts again from the
# branch's newest commit.
set -euo pipefail

usage='usage: sync-pins.sh <default-branch> <publishing: true|false>'
default_branch=${1:?$usage}
publishing=${2:?$usage}
publisher=${DREAMWEAVE_TEMPLATE_PUBLISHER:?DREAMWEAVE_TEMPLATE_PUBLISHER names the Mod Template site}

if [[ -f jess.lock ]]; then
  jess workspace sync --locked
  exit 0
fi

echo "No jess.lock yet: pinning the template from $publisher"
jess workspace sync --publisher "$publisher"

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

echo "::error::Could not push jess.lock to ${default_branch}. If the branch is protected, allow GitHub Actions to push to it, or run 'jess workspace sync --publisher ${publisher}' and commit jess.lock yourself."
exit 1
