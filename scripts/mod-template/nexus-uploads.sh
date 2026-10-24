#!/usr/bin/env bash
# The Nexus Mods uploads of a tagged release, as the GitHub Actions output `uploads`: a JSON list of
# { name, version, file_group_id, filename, primary } for the upload job's matrix, filenames under
# dist/.
#
# A mod's are the template's: dist/nexus.json, from its project's [nexusmods] file_group_id. A Jess
# application's are a program's, uploaded as the Rust workflow uploads one: each platform's zip
# (dist/<APPLICATION>-<System>-<Arch>.zip) to the file group NEXUS_GROUP_IDS names for it, under the
# Rust workflow's keys (`windows-x64-stable`, `macos-arm64-stable`, `linux-x64-stable`, ...), and
# none for a platform it names no group for. A program is not a mod manager's primary download.
#
# Environment: APPLICATION (jess_application, may be empty), NEXUS_GROUP_IDS (the secret, may be
# empty), GITHUB_OUTPUT.
set -euo pipefail

uploads=$(cat dist/nexus.json)

if [ -n "${APPLICATION:-}" ] && [ -n "${NEXUS_GROUP_IDS:-}" ]; then
  if ! jq -e 'type == "object"' <<< "$NEXUS_GROUP_IDS" > /dev/null 2>&1; then
    echo "::error::NEXUS_GROUP_IDS must be a JSON object of file group ids, keyed like windows-x64-stable"
    exit 1
  fi

  version=$(jq -r '.release.version' dist/release.json)
  shopt -s nullglob

  # The template names each platform's archive for its system as jess's runtimes are named, and
  # the Rust workflow keys its file groups by the same systems in lower case.
  for system in Windows macOS Linux Portmaster Android; do
    os=${system,,}
    for archive in dist/"$APPLICATION-$system"-*.zip; do
      file=${archive#dist/}
      arch=${file#"$APPLICATION-$system"-}
      arch=${arch%.zip}
      arch=${arch,,}
      key="$os-$arch-stable"

      group=$(jq -r --arg key "$key" '.[$key] // empty | tostring' <<< "$NEXUS_GROUP_IDS")
      if [ -z "$group" ]; then
        echo "::notice::NEXUS_GROUP_IDS names no file group for $key, so $file is not uploaded to Nexus Mods"
        continue
      fi

      uploads=$(jq -c \
        --arg name "$APPLICATION-$os-$arch-$version" \
        --arg version "$version" \
        --arg group "$group" \
        --arg file "$file" \
        '. + [{ name: $name, version: $version, file_group_id: $group, filename: $file, primary: false }]' \
        <<< "$uploads")
    done
  done
fi

echo "uploads=$uploads" >> "$GITHUB_OUTPUT"
