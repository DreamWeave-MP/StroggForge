#!/usr/bin/env bash
# bump-version.sh — update StroggForge self-referential version pins and tag the release.
set -euo pipefail

read -rp "Current version tag (e.g. v8): " OLD_TAG
read -rp "New version tag    (e.g. v9): " NEW_TAG

if [[ -z "$OLD_TAG" || -z "$NEW_TAG" ]]; then
  echo "Error: both tags must be non-empty." >&2
  exit 1
fi

if [[ "$OLD_TAG" == "$NEW_TAG" ]]; then
  echo "Error: old and new tags are identical." >&2
  exit 1
fi

FILES=(
  .github/workflows/rustGlobalBuild.yml
  .github/workflows/libGlobalBuild.yml
  .github/action_templates/rust_template.yaml
  .github/action_templates/lib_template.yaml
)

echo ""
echo "Replacing '${OLD_TAG}' → '${NEW_TAG}' in StroggForge self-references..."

for f in "${FILES[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "  WARNING: $f not found, skipping."
    continue
  fi
  sed -i \
    -e "s|StroggForge/\(.*\)@${OLD_TAG}|StroggForge/\1@${NEW_TAG}|g" \
    -e "s|ref: ${OLD_TAG}$|ref: ${NEW_TAG}|g" \
    -e "s|\`@${OLD_TAG}\`|\`@${NEW_TAG}\`|g" \
    "$f"
  echo "  updated: $f"
done

echo ""
echo "Verifying no StroggForge self-references to '${OLD_TAG}' remain..."
LEFTOVERS=$(grep -rn --include="*.yml" --include="*.yaml" --include="*.md" \
  -E "StroggForge/.*@${OLD_TAG}|ref: ${OLD_TAG}$" "${FILES[@]}" 2>/dev/null || true)

if [[ -n "$LEFTOVERS" ]]; then
  echo "WARNING: leftover references found:"
  echo "$LEFTOVERS"
else
  echo "All clear."
fi

echo ""
read -rp "Commit changes and create git tag '${NEW_TAG}'? [y/N] " CONFIRM
if [[ "${CONFIRM,,}" != "y" ]]; then
  echo "Skipped commit/tag. Changes are staged on disk only."
  exit 0
fi

git add "${FILES[@]}"
git commit -m "RELEASE: Bump self-referential version pins from ${OLD_TAG} to ${NEW_TAG}"
git tag "${NEW_TAG}"

echo ""
echo "Done. Committed and tagged '${NEW_TAG}'."
echo "Push with: git push && git push origin ${NEW_TAG}"
