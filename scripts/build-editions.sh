#!/bin/bash
# Build multiple editions of Rustic Playground from the same codebase.
# Each edition gets its own app name, bundle ID, and configuration.
#
# Usage:
#   ./scripts/build-editions.sh              # build all editions (no notarize)
#   ./scripts/build-editions.sh rust         # build only the rust edition
#   ./scripts/build-editions.sh rust power   # build rust and power editions
#
#   NOTARIZE=1 ./scripts/build-editions.sh rust    # full release flow:
#                                                    build + strip + notarize + staple
#
# When to set NOTARIZE=1:
#   Release builds only. Each notarization round-trip to Apple takes 1–3 min,
#   so dev/test builds skip it. Set NOTARIZE=1 (or export it in your shell) when
#   you're building the final DMG for a release.
#
# Why the collect step exists:
#   Tauri's bundler wipes <target>/release/bundle/dmg/ at the start of every
#   `cargo tauri build`, so building two editions back-to-back leaves only
#   the last one. After each build, we copy the fresh DMG(s) out to
#   <target>/release/editions/<edition>/ so every edition survives the next
#   build.
#
# Why we use cargo metadata to find the target dir:
#   Whether cargo puts output in <repo>/target/ or <repo>/src-tauri/target/
#   depends on whether there's a [workspace] declaration in the root
#   Cargo.toml. Hardcoding either path is fragile. `cargo metadata` is the
#   authoritative source for the actual target directory.

set -euo pipefail

ALL_EDITIONS=("power" "rust" "clang" "zig" "swift")

if [ $# -gt 0 ]; then
  EDITIONS=("$@")
else
  EDITIONS=("${ALL_EDITIONS[@]}")
fi

# Resolve the cargo target directory dynamically. This works regardless of
# whether we're in a workspace, what CARGO_TARGET_DIR is set to, etc.
echo "Resolving cargo target directory…"
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps --manifest-path src-tauri/Cargo.toml 2>/dev/null \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['target_directory'])")
if [ -z "$TARGET_DIR" ] || [ ! -d "$TARGET_DIR" ] && [ ! -d "$(dirname "$TARGET_DIR")" ]; then
  echo "ERROR: failed to resolve cargo target directory via cargo metadata" >&2
  exit 1
fi
echo "  target dir: $TARGET_DIR"

DMG_DIR="$TARGET_DIR/release/bundle/dmg"
COLLECT_DIR="$TARGET_DIR/release/editions"
mkdir -p "$COLLECT_DIR"

for edition in "${EDITIONS[@]}"; do
  config="editions/${edition}.json"
  if [ ! -f "$config" ]; then
    echo "ERROR: $config not found — skipping $edition"
    continue
  fi
  echo ""
  echo "══════════════════════════════════════════════════"
  echo "  Building: $edition edition"
  echo "  Config:   $config"
  echo "══════════════════════════════════════════════════"
  echo ""
  VITE_EDITION="$edition" cargo tauri build --config "$config"

  # Strip the custom volume icon from each DMG so the mounted volume shows
  # macOS's default removable-disk icon (clearer drag-to-Applications UX
  # than having the volume icon identical to the app inside it). This also
  # re-signs the DMG since the modification invalidates the signature.
  if compgen -G "$DMG_DIR/*.dmg" > /dev/null; then
    for dmg in "$DMG_DIR"/*.dmg; do
      ./scripts/strip-dmg-volume-icon.sh "$dmg"
    done
  fi

  # Optional: notarize + staple. Opt-in via NOTARIZE=1 because each round-trip
  # takes 1–3 min and isn't needed for dev/test builds. Required for release.
  if [ "${NOTARIZE:-0}" = "1" ]; then
    if compgen -G "$DMG_DIR/*.dmg" > /dev/null; then
      for dmg in "$DMG_DIR"/*.dmg; do
        ./scripts/notarize-and-staple.sh "$dmg"
      done
    fi
  else
    echo "  (skipping notarization — set NOTARIZE=1 to enable)"
  fi

  # Move the fresh DMG(s) out of the bundle dir before the next edition's
  # build wipes it. Each edition gets its own subfolder.
  edition_out="$COLLECT_DIR/$edition"
  rm -rf "$edition_out"
  mkdir -p "$edition_out"
  if compgen -G "$DMG_DIR/*.dmg" > /dev/null; then
    for dmg in "$DMG_DIR"/*.dmg; do
      cp "$dmg" "$edition_out/"
      echo "  collected: $edition_out/$(basename "$dmg")"
    done
  else
    # Loud fail — silently skipping was the bug that hid the previous issue
    echo "" >&2
    echo "ERROR: no .dmg found in $DMG_DIR after building $edition" >&2
    echo "       The build either failed or Tauri put the DMG somewhere else." >&2
    echo "       Check the cargo tauri build output above for errors." >&2
    exit 1
  fi

  echo ""
  echo "✓ $edition edition built successfully"
done

echo ""
echo "══════════════════════════════════════════════════"
echo "  All requested editions collected in:"
echo "  $COLLECT_DIR/"
echo "══════════════════════════════════════════════════"
for edition in "${EDITIONS[@]}"; do
  edition_out="$COLLECT_DIR/$edition"
  if [ -d "$edition_out" ]; then
    for dmg in "$edition_out"/*.dmg; do
      [ -f "$dmg" ] && echo "  • $dmg"
    done
  fi
done
