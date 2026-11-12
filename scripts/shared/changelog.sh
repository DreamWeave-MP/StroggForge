#!/usr/bin/env bash
set -euo pipefail

output=${1:-CHANGELOG.md}
repo_url="${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

# A ref named like an option (`--output=…`) can be made with `git update-ref` though `git tag`
# refuses it; it is no release, and passed to git log it would be read as an option.
mapfile -t tags < <(git tag --sort=-version:refname 2>/dev/null | grep -v '^-' || true)

emit_commits() {
  local range=$1

  git log --format="format:%h %s" --end-of-options "$range" | while read -r hash msg; do
    echo "- [$hash]($repo_url/commit/$hash) - $msg"
  done
}

{
  echo "# Changelog"
  echo ""

  if [ "${#tags[@]}" -eq 0 ]; then
    echo "## Unreleased"
    echo ""
    emit_commits "HEAD"
    echo ""
  else
    unreleased=$(git log --format="format:%h %s" --end-of-options "${tags[0]}..HEAD")
    if [ -n "$unreleased" ]; then
      echo "## Unreleased"
      echo ""
      echo "$unreleased" | while read -r hash msg; do
        echo "- [$hash]($repo_url/commit/$hash) - $msg"
      done
      echo ""
    fi

    for i in "${!tags[@]}"; do
      tag=${tags[$i]}
      echo "## $tag"
      echo ""

      if [ $((i + 1)) -lt "${#tags[@]}" ]; then
        emit_commits "${tags[$((i + 1))]}..${tag}"
      else
        emit_commits "$tag"
      fi

      echo ""
    done
  fi
} > "$output"
