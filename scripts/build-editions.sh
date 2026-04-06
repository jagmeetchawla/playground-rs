#!/bin/bash
# Build multiple editions of Rustic Playground from the same codebase.
# Each edition gets its own app name, bundle ID, and configuration.
#
# Usage:
#   ./scripts/build-editions.sh              # build all editions
#   ./scripts/build-editions.sh rust         # build only the rust edition
#   ./scripts/build-editions.sh rust clang   # build rust and clang editions

set -euo pipefail

ALL_EDITIONS=("power" "rust" "clang" "zig" "swift")

if [ $# -gt 0 ]; then
  EDITIONS=("$@")
else
  EDITIONS=("${ALL_EDITIONS[@]}")
fi

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
  echo ""
  echo "✓ $edition edition built successfully"
done

echo ""
echo "All requested editions built. DMGs are in src-tauri/target/release/bundle/dmg/"
