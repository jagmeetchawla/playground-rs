#!/bin/bash
# Bump the rustic-rust Homebrew cask in cloudcraft-ai/homebrew-tap to point at a
# freshly-released DMG. Computes the sha256 from the local DMG (identical bytes
# to the uploaded asset), rewrites version + sha256 in the cask via the GitHub
# API, and commits directly to the tap's default branch.
#
# No local clone of the tap is required — just `gh` authenticated with write
# access to cloudcraft-ai/homebrew-tap.
#
# ORDER MATTERS: run this AFTER the GitHub Release asset is uploaded, because
# the cask's `url` points at the release download. If you bump first, the URL
# 404s until the asset lands. Typical release tail:
#   NOTARIZE=1 ./scripts/build-editions.sh rust
#   gh release create v<ver> "dist/rust/Rustic.Rust_<ver>_aarch64.dmg" ...
#   ./scripts/bump-homebrew-cask.sh "dist/rust/Rustic.Rust_<ver>_aarch64.dmg"
#
# Usage: ./scripts/bump-homebrew-cask.sh <path-to-dmg> [version]
#   version defaults to the VERSION file.
#   DRY_RUN=1  compute + rewrite but do NOT commit (prints the diff instead).

set -euo pipefail

TAP_REPO="${TAP_REPO:-cloudcraft-ai/homebrew-tap}"
CASK_PATH="${CASK_PATH:-Casks/rustic-rust.rb}"

if [ $# -lt 1 ]; then
  echo "Usage: $0 <path-to-dmg> [version]" >&2
  exit 1
fi

DMG="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${2:-$(cat "$REPO_ROOT/VERSION")}"

if [ ! -f "$DMG" ]; then
  echo "ERROR: DMG not found: $DMG" >&2
  exit 1
fi
command -v gh >/dev/null || { echo "ERROR: gh (GitHub CLI) not found" >&2; exit 1; }

SHA256="$(shasum -a 256 "$DMG" | awk '{print $1}')"
echo "Bumping $CASK_PATH in $TAP_REPO"
echo "  version: $VERSION"
echo "  sha256:  $SHA256"

# Pull the current cask + its blob sha (the Contents API needs the blob sha to
# update an existing file). Rewrite only the version/sha256 stanza lines so the
# url/desc/zap lines are never touched.
BLOB_SHA="$(gh api "repos/$TAP_REPO/contents/$CASK_PATH" --jq '.sha')"
NEW_CONTENT="$(gh api "repos/$TAP_REPO/contents/$CASK_PATH" --jq '.content' | base64 -d | sed -E \
  -e "s|^(  version )\"[^\"]*\"|\1\"$VERSION\"|" \
  -e "s|^(  sha256 )\"[^\"]*\"|\1\"$SHA256\"|")"

# Fail loudly if either substitution didn't land (e.g. the cask was reformatted).
printf '%s\n' "$NEW_CONTENT" | grep -q "version \"$VERSION\"" || { echo "ERROR: version not updated" >&2; exit 1; }
printf '%s\n' "$NEW_CONTENT" | grep -q "sha256 \"$SHA256\""  || { echo "ERROR: sha256 not updated"  >&2; exit 1; }

if [ "${DRY_RUN:-0}" = "1" ]; then
  echo "  (DRY_RUN — not committing. Resulting version/sha lines:)"
  printf '%s\n' "$NEW_CONTENT" | grep -E "^  (version|sha256) "
  exit 0
fi

B64="$(printf '%s\n' "$NEW_CONTENT" | base64)"
gh api -X PUT "repos/$TAP_REPO/contents/$CASK_PATH" \
  -f message="rustic-rust $VERSION" \
  -f content="$B64" \
  -f sha="$BLOB_SHA" \
  --jq '{committed: .commit.sha}'
echo "  ✓ tap bumped — brew install --cask $TAP_REPO/../rustic-rust now serves $VERSION"
