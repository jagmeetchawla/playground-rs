#!/bin/bash
# Reads VERSION file and updates tauri.conf.json, Cargo.toml, and package.json.
# Hooked into Tauri's beforeDevCommand and beforeBuildCommand.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION=$(cat "$ROOT/VERSION" | tr -d '[:space:]')

if [ -z "$VERSION" ]; then
  echo "ERROR: VERSION file is empty" >&2
  exit 1
fi

# Cargo.toml — update version in [package] section (first occurrence only).
# Uses awk because BSD sed's `0,/.../` range is unreliable.
awk -v v="$VERSION" 'BEGIN{done=0} /^version = / && !done {sub(/"[^"]+"/, "\"" v "\""); done=1} {print}' \
  "$ROOT/src-tauri/Cargo.toml" > "$ROOT/src-tauri/Cargo.toml.tmp" \
  && mv "$ROOT/src-tauri/Cargo.toml.tmp" "$ROOT/src-tauri/Cargo.toml"

# tauri.conf.json — update top-level "version" field
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$ROOT/src-tauri/tauri.conf.json"

# package.json — update "version" field
sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$ROOT/ui/package.json"

echo "✓ Version synced to $VERSION"
