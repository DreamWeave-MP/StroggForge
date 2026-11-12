#!/usr/bin/env bash
set -euo pipefail

usage='usage: publish-github-release-artifacts.sh <release-name> <body-file> <download-dir> <github-repository>'
release_name=${1:?$usage}
body_file=${2:?$usage}
download_dir=${3:?$usage}
github_repository=${4:?$usage}

# A tag may start with `-`; as a release name it would reach gh as an option (`--help` publishes
# nothing and exits 0). No release is named like that, so refuse it.
if [[ "$release_name" == -* ]]; then
  echo "::error::Refusing release name '$release_name': it starts with '-'."
  exit 1
fi

tmp_files=()
cleanup() {
  rm -f "${tmp_files[@]}"
}
trap cleanup EXIT

shopt -s nullglob
release_files=("$download_dir"/*.zip "$download_dir"/*.muxapp)
shopt -u nullglob

if (( ${#release_files[@]} == 0 )); then
  echo "::error::No GitHub Release zip or muxapp artifacts found in ${download_dir}."
  exit 1
fi

gh release upload "$release_name" "${release_files[@]}" --repo "$github_repository" --clobber

if [[ -s "$body_file" ]]; then
  updated_body=$(mktemp)
  tmp_files+=("$updated_body")

  current_body=$(gh release view "$release_name" --repo "$github_repository" --json body --jq '.body // ""')
  {
    printf '%s\n\n' "$current_body"
    cat "$body_file"
  } > "$updated_body"

  gh release edit "$release_name" --repo "$github_repository" --notes-file "$updated_body"
else
  echo "No release notes were appended."
fi
