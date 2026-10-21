#!/usr/bin/env bash
# Turn a copy of the DreamWeave Mod Template into a mod site. "Use this template" copies the whole
# repository, the build tool's source included; a site is not the tool, so the first run strips
# what only the template's own repository needs and commits the result. The condition is a state,
# not a count: the root jess.toml declares the template's own package, in a repository that is
# neither the template's nor a fork of it. After this commit it is false for good.
#
# Nothing a developer touched is deleted: every path stripped must be untouched since the copy's
# first commit, or the run stops and says what to do by hand.
set -euo pipefail

usage='usage: scaffold.sh <default-branch>'
default_branch=${1:?$usage}

[[ -f jess.toml ]] || exit 0
grep -q '^id = "org.dreamweave.mod-template"$' jess.toml || exit 0

echo "This repository is a copy of the DreamWeave Mod Template that still holds the tool's source."

tool=(src tests bench BENCHMARKS.md jess.toml)
page=content/template
strip=("${tool[@]}")
if [[ -f "$page/mod.toml" ]] && grep -q '^id = "74143198-50bf-434b-938e-816699a5d0a4"$' "$page/mod.toml"; then
  strip+=("$page")
fi

root=$(git rev-list --max-parents=0 HEAD | tail -1)
touched=$(git log --format=%H -n 1 -- "${strip[@]}" jess-workspace.toml config.toml)
if [[ "$touched" != "$root" ]]; then
  echo "::error::The tool's files were changed after the copy's first commit ($(git log --format='%h %s' -n 1 "$touched")), so nothing is removed. To make this repository a site by hand: delete ${tool[*]} and $page, and make jess-workspace.toml list no members and name the template under [dev-dependencies]."
  exit 1
fi

requirement=$(sed -n 's/^version = "\([0-9]*\)\.\([0-9]*\)\..*$/\1.\2/p' jess.toml | head -1)
[[ -n "$requirement" ]] || { echo "::error::jess.toml has no version to pin the template at."; exit 1; }

git rm -rq -- "${strip[@]}"
cat > jess-workspace.toml <<WORKSPACE
# This site as a jess workspace: no package of its own, and the DreamWeave Mod Template as the
# tool that builds it, pinned exactly in jess.lock. Move the version here to update the whole
# site: every template, stylesheet and script comes from the packages the lock pins.
manifest_version = 1

[workspace]
members = []

[dev-dependencies]
"org.dreamweave.mod-template" = "$requirement"
WORKSPACE
# The header's link to the template's own page goes with the page.
sed -i '/^    { name = "Template", url = "@\/template\/index.md" },$/d' config.toml
git add jess-workspace.toml config.toml

git config user.name 'github-actions[bot]'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
git commit --quiet -m "CLEANUP: Make this repository a DreamWeave site: the Mod Template is its tool, pinned in jess-workspace.toml, not its source"

for attempt in 1 2 3 4 5; do
  if git push --quiet origin "HEAD:refs/heads/${default_branch}"; then
    echo "Scaffolded: the tool's source is gone and jess-workspace.toml names the template $requirement."
    exit 0
  fi
  sleep $((attempt * 5))
done

echo "::error::Could not push the scaffold commit to ${default_branch}. If the branch is protected, allow GitHub Actions to push to it, or make the same change by hand: delete ${tool[*]} and $page, and make jess-workspace.toml list no members and name the template under [dev-dependencies]."
exit 1
