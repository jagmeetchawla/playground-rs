#!/bin/bash
# Strip the custom `.VolumeIcon.icns` from a DMG so that macOS shows its
# default removable-disk icon when the DMG is mounted. Tauri's bundler copies
# the app's .icns as the volume icon, which produces a confusing UX where the
# mounted volume looks identical to the app inside it. Removing the custom
# volume icon makes the drag-to-Applications metaphor clearer.
#
# Re-signs the DMG after modification because stripping the volume icon
# invalidates the existing signature. Notarization is NOT re-run here — the
# caller is expected to re-notarize + staple as part of the release flow
# (since that takes 1–3 min per DMG and doesn't belong in a fast-build step).
#
# Usage: ./scripts/strip-dmg-volume-icon.sh <path-to-dmg>

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <path-to-dmg>" >&2
  exit 1
fi

DMG="$1"
if [ ! -f "$DMG" ]; then
  echo "ERROR: DMG not found: $DMG" >&2
  exit 1
fi

# Resolve signing identity from tauri.conf.json (single source of truth)
SIGNING_IDENTITY=$(python3 -c "import json; print(json.load(open('src-tauri/tauri.conf.json'))['bundle']['macOS']['signingIdentity'])")
if [ -z "$SIGNING_IDENTITY" ]; then
  echo "ERROR: signingIdentity not found in src-tauri/tauri.conf.json" >&2
  exit 1
fi

BASENAME=$(basename "$DMG" .dmg)
RW_DMG="${TMPDIR:-/tmp}/${BASENAME}-rw-$$.dmg"
MOUNT_POINT="${TMPDIR:-/tmp}/${BASENAME}-mount-$$"

# Cleanup on exit in case of failure. Runs on every exit (success or fail).
# `|| true` on every conditional so set -e doesn't turn a benign "already
# cleaned up" state into a nonzero exit — that's the bug that made the
# script exit 1 even after printing its success line.
cleanup() {
  if mount | grep -q "$MOUNT_POINT"; then
    hdiutil detach "$MOUNT_POINT" -quiet 2>/dev/null || true
  fi
  [ -d "$MOUNT_POINT" ] && rmdir "$MOUNT_POINT" 2>/dev/null || true
  [ -f "$RW_DMG" ] && rm -f "$RW_DMG" || true
}
trap cleanup EXIT

echo "Stripping volume icon from: $(basename "$DMG")"

# 1. Convert compressed read-only DMG → read-write sparse
hdiutil convert "$DMG" -format UDRW -o "$RW_DMG" -quiet

# 2. Attach at an explicit mount point (avoids parsing hdiutil output)
mkdir -p "$MOUNT_POINT"
hdiutil attach "$RW_DMG" -mountpoint "$MOUNT_POINT" -noautoopen -nobrowse -quiet

# 3. Remove the custom volume icon + clear the "has custom icon" FinderInfo
#    attribute. Without the file, Finder falls back to the default disk icon
#    regardless of the attribute, but we strip both for cleanliness.
rm -f "$MOUNT_POINT/.VolumeIcon.icns"
xattr -d com.apple.FinderInfo "$MOUNT_POINT" 2>/dev/null || true

# 4. Detach and convert back to compressed read-only (UDZO), replacing original
hdiutil detach "$MOUNT_POINT" -quiet
rmdir "$MOUNT_POINT"
hdiutil convert "$RW_DMG" -format UDZO -ov -o "$DMG" -quiet
rm "$RW_DMG"

# 5. Re-sign the DMG (original signature invalidated by modification)
codesign --sign "$SIGNING_IDENTITY" --timestamp "$DMG"

echo "  ✓ stripped + re-signed"
