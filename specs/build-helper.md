BUILD HELPER
============

How to build, run, and distribute Rustic Playground editions.

---

VERSION MANAGEMENT
──────────────────

Single source of truth: the `VERSION` file in the project root.

    $ cat VERSION
    0.3.3

This version is automatically synced to three files on every build/dev run:
- `src-tauri/tauri.conf.json` (bundle version — shown in About modal)
- `src-tauri/Cargo.toml` (Rust crate version — shown in build output)
- `ui/package.json` (frontend package version)

The sync is handled by `scripts/sync-version.sh`, which is hooked into
Tauri's `beforeDevCommand` and `beforeBuildCommand` in `tauri.conf.json`.

To bump the version:
    $ echo "0.3.4" > VERSION

The next `cargo tauri dev` or `cargo tauri build` will sync automatically.
No need to manually edit the three files.

---

EDITIONS
────────

The app ships as multiple editions from one codebase. Each edition is
defined by two things:

1. A Tauri config override (JSON) — sets app name, bundle ID, window title
2. The VITE_EDITION env var — tells the frontend which edition to render

Edition configs live in `editions/`:

    editions/
    ├── rust.json      Rustic Playground — The Rust Edition
    ├── power.json     Rustic Playground — Power Edition
    ├── clang.json     Rustic Playground — The C Edition
    ├── zig.json       Rustic Playground — The Zig Edition
    └── swift.json     Rustic Playground — The Swift Edition

Frontend edition registry: `ui/src/lib/editions.ts`
- EditionConfig interface: id, displayName, tagline, languages, defaultTheme,
  defaultProjectType, isSingleLanguage
- currentEdition() reads VITE_EDITION env var, defaults to "power"

Currently shipping: Rust Edition (first) and Power Edition (later).
Other single-language editions exist in the registry but are not distributed.

---

EDITION ISOLATION
─────────────────

Each edition uses a different bundle identifier:

    Edition         Identifier                      App Data Path
    ──────────────  ──────────────────────────────  ──────────────────────────────────────────────
    Rust Edition    com.rustic-playground.rust       ~/Library/Application Support/com.rustic-playground.rust/
    Power Edition   com.rustic-playground.power      ~/Library/Application Support/com.rustic-playground.power/
    Dev (no flag)   com.rustic-playground.app        ~/Library/Application Support/com.rustic-playground.app/

This means:
- Editions can be installed side by side in /Applications/
- Each has its own config, projects, window state, and settings
- Running one edition does not affect another's data
- Dev builds (no edition flag) use a separate storage path

---

DEV COMMANDS
────────────

Run without edition (uses base tauri.conf.json, com.rustic-playground.app):

    cargo tauri dev

Run a specific edition:

    VITE_EDITION=rust cargo tauri dev --config editions/rust.json
    VITE_EDITION=power cargo tauri dev --config editions/power.json

The --config flag deep-merges the edition JSON over tauri.conf.json.
VITE_EDITION is read by the frontend at build time via import.meta.env.

Note: The --config path is relative to where you run the command.
Always run from the project root, not from src-tauri/.

---

BUILD COMMANDS
──────────────

Build a specific edition DMG:

    VITE_EDITION=rust cargo tauri build --config editions/rust.json
    VITE_EDITION=power cargo tauri build --config editions/power.json

Output: src-tauri/target/release/bundle/dmg/

Clean build (recommended when icons or major config changed):

    cd src-tauri && cargo clean && cd ..
    VITE_EDITION=rust cargo tauri build --config editions/rust.json

Build multiple editions (script):

    ./scripts/build-editions.sh rust power
    ./scripts/build-editions.sh              # builds all editions

---

BACKEND-ONLY CHECKS (no frontend needed)
─────────────────────────────────────────

    cd src-tauri && cargo check
    cd src-tauri && cargo fmt
    cd src-tauri && cargo clippy -- -D warnings

---

FRONTEND-ONLY DEV (no Tauri needed)
────────────────────────────────────

    cd ui && pnpm install
    cd ui && pnpm dev          # Vite dev server at localhost:1420
    cd ui && pnpm build        # production bundle to ui/dist/

---

ICONS
─────

Icons are organized by edition in `assets/`:

    assets/
    ├── GitHub/              Screenshots for README, website
    ├── The Rust Edition/    Rust Edition icon (1024x1024, Apple Icon Composer)
    └── Power Edition/       Power Edition icon (placeholder)

Icon sizes in src-tauri/icons/ are generated from the 1024x1024 source.
Apple HIG requires 824x824 artwork centered on 1024x1024 canvas (100px
padding on each side).

Generation tools:
- sips (macOS built-in) — resize PNGs
- iconutil (macOS built-in) — create .icns from .iconset
- Python PIL — create .ico for Windows

IMPORTANT: After replacing icons, run `cargo clean` in src-tauri/.
Tauri caches icons in the target directory and won't pick up changes
without a clean build.

---

DELETING APP DATA (for clean testing)
──────────────────────────────────────

    rm -rf ~/Library/Application\ Support/com.rustic-playground.rust
    rm -rf ~/Library/Application\ Support/com.rustic-playground.power
    rm -rf ~/Library/Application\ Support/com.rustic-playground.app

---

TROUBLESHOOTING
───────────────

"invalid value for '--config'" error:
  The --config path is relative to CWD. Run from project root, not src-tauri/.

Version mismatch in build output:
  Check that VERSION file has the correct version. The sync script runs
  automatically but verify with: ./scripts/sync-version.sh

Icon not updating after replacement:
  Run `cd src-tauri && cargo clean` before rebuilding. Tauri caches icons.

Dev port conflict (two editions running simultaneously in dev):
  Both use localhost:1420. This only affects dev mode, not production builds.
  To run two editions in dev simultaneously, change devUrl port for one.
