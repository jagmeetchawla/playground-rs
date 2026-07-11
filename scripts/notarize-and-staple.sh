#!/bin/bash
# Notarize a DMG via Apple's notarytool, then staple the notarization
# ticket so Gatekeeper can verify it offline. Without stapling, users on
# air-gapped/slow-network first launches hit Gatekeeper warnings or hangs.
#
# Requires a notarytool keychain profile named 'cloud-craft-ai-notarytool'
# (team-scoped so future CloudCraft apps reuse the same profile). Override
# via the NOTARY_PROFILE env var if a different name is needed.
#
# Usage: ./scripts/notarize-and-staple.sh <path-to-dmg>

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <path-to-dmg>" >&2
  exit 1
fi

DMG="$1"
PROFILE="${NOTARY_PROFILE:-cloud-craft-ai-notarytool}"

if [ ! -f "$DMG" ]; then
  echo "ERROR: DMG not found: $DMG" >&2
  exit 1
fi

echo "Notarizing: $(basename "$DMG")"
echo "  (this takes 1–3 minutes while Apple reviews)"

# Submit + wait for result. --wait polls until Apple returns a verdict.
xcrun notarytool submit "$DMG" --keychain-profile "$PROFILE" --wait

# Staple the notarization ticket into the DMG so Gatekeeper can verify offline
echo "Stapling notarization ticket…"
xcrun stapler staple "$DMG"

# Verify the staple took
echo "Verifying staple…"
xcrun stapler validate "$DMG"

# Final Gatekeeper check against the stapled DMG
echo "Verifying Gatekeeper acceptance…"
spctl -a -t open --context context:primary-signature -v "$DMG"

echo "  ✓ notarized + stapled + verified"
