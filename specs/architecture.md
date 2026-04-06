ARCHITECTURE

---

v1.0 — Tauri Desktop App (current)

Stack
- Tauri 2.0       — app shell, Rust backend, macOS WKWebView
- Svelte 5        — frontend UI framework (runes: $state, $derived, $effect)
- Vite + pnpm     — frontend bundler
- Monaco Editor   — code editor (same engine as VS Code)
- Rust backend    — all Tauri commands, modularised across several files

Backend Module Structure

src-tauri/src/
├── lib.rs                   — app state, paths, validation, config, settings, window state, run()
├── languages/               — per-language modules (v0.3)
│   ├── mod.rs               — Lang enum, RunConfig enum, shared FileLanguage helpers
│   ├── rust.rs              — Cargo package: scaffold, run, check, export
│   ├── clang.rs             — C/C++ with clang: scaffold, compile+run, export
│   ├── zig.rs               — Zig: scaffold, zig run (direct), export
│   └── swift.rs             — Swift: scaffold, swiftc compile+run, export
├── playground_commands.rs   — thin dispatchers: CRUD + run/kill/check via Lang enum
├── cargo_commands.rs        — Cargo.toml get/save, add/remove dependency, toolchain info/check, wizard
├── content_commands.rs      — content file CRUD (9 commands)
├── export.rs                — thin router: Lang enum → per-language export
├── menu.rs                  — build_menu + rebuild_menu (macOS menu bar)
├── languages/rust_book.rs   — seed_rust_book command (20 Rust Book chapter projects)
├── languages/knr_book.rs    — seed_knr_book command (8 K&R C Book chapter projects)
├── languages/swift_book.rs  — seed_swift_book command (8 Swift Book chapter projects)
├── rustic_manifest.rs       — rustic.toml manifest CRUD (project type, flags, toolchain)
└── main.rs                  — Tauri entry point

Frontend Structure

ui/src/
├── App.svelte               — root layout, global state, menu listeners, window state, live checking
├── lib/
│   ├── Sidebar.svelte       — project/playground/file tree, drag-drop, context menus
│   ├── Editor.svelte        — Monaco wrapper, theme sync, diagnostics markers
│   ├── Output.svelte        — console panel, run blocks, streaming output
│   ├── SettingsModal.svelte  — settings panel (editor, appearance) [legacy, mostly moved to ToolchainWizard]
│   ├── NewPlaygroundModal.svelte — new playground dialog with template picker
│   ├── HelpModal.svelte     — Apple-style user guide with sidebar nav (⌘⇧/)
│   ├── AboutModal.svelte    — about dialog
│   ├── ProjectSwitcher.svelte — project list popover, new/rename/delete project
│   ├── ToolchainWizard.svelte — dual-mode: 5-step Welcome Wizard + Settings panel (⌘,)
│   ├── CopyToProjectModal.svelte — copy book playground to user project
│   ├── TabBar.svelte        — editor tab bar with language-aware badges
│   ├── templates.ts         — Rust, C, and C++ starter templates
│   ├── languages.ts         — frontend language registry (v0.3)
│   ├── editions.ts          — edition registry: EditionConfig, currentEdition() (v0.3.3)
│   ├── zig_templates.ts     — Zig starter templates (v0.3)
│   └── swift_templates.ts   — Swift starter templates (v0.3)
├── app.css                  — global CSS variables (dark/light themes)
└── main.ts                  — Svelte mount point

Module Responsibilities

lib.rs (coordination hub, ~550 lines):
- State structs: ActiveProject, RunningProcess, StdinHandle, CheckProcess, Config, Settings
- Path helpers: projects_dir, workspace_dir, bin_dir, content_dir, config_path, etc.
- Config/settings persistence: load_config, save_config, get/save_settings, get/save_window_state
- Validation: validate_name, validate_filename, safe_playground_path, safe_content_path
- Content helpers: ContentFile struct, is_text_file
- Toolchain resolution: cargo_path, which_cargo
- Project bootstrap: ensure_project, project_cargo_toml, playground_template
- run() entry point: state registration, menu setup, invoke_handler, menu event routing

playground_commands.rs (~500 lines):
- Project management: list/get_active/new/switch/rename/delete/duplicate project
- Playground CRUD: list/load/save/new/rename/delete/duplicate playground, workspace_path
- Saved snapshot system: snapshot_playground, revert_playground (see .saved/ section below)
- Process management: run_playground (streaming stdout/stderr via Channel), kill_playground (SIGTERM→SIGKILL process group), check_playground (cargo check with JSON diagnostics), cancel_check, send_stdin

cargo_commands.rs (~300 lines):
- Cargo.toml: get/save with TOML validation, add/remove dependency (format-preserving via toml_edit)
- Toolchain: get_toolchain_info, check_toolchain (comprehensive wizard check), complete_wizard

content_commands.rs (~120 lines):
- Content file CRUD: list/create/read/save/delete/rename/import, reveal_in_finder, get_content_file_path
- Import guard: 10 MB file size limit (MAX_CONTENT_FILE_SIZE)

export.rs (~400 lines):
- export_project: thin router — detects project type, dispatches to Rust or Clang export
- Rust path: export_rust_project + CLI_MAIN_RS (clap-based CLI runner, merged Cargo.toml)
- Clang path: export_clang_project + CLI_PLAYGROUND_SH (shell script runner, Makefile)
- Shared helper: copy_content_files (pure I/O, no language-specific logic)
- Separation: Rust and Clang exports are independent functions with own constants.
  Modifying one cannot break the other. Kept in the same file for discoverability.

menu.rs (~150 lines):
- build_menu: constructs full macOS menu bar (App, Project, Playground, Run, Edit, Help)
- rebuild_menu: Tauri command to reconstruct menu when state changes

book_chapters.rs (~2,700 lines):
- seed_rust_book: creates 20 Rust Book chapter projects with all playgrounds and attribution

knr_chapters.rs (~1,200 lines):
- seed_knr_book: creates 8 K&R C Book chapter projects (clang type) with attribution
- Fully separate module from book_chapters.rs — no shared code

Project Types & Language Module Architecture

Four project types: "rust" (default, first-class), "clang" (C/C++), "zig", "swift".

Rust is the first-class citizen. The app always starts with Rust toolchain detection,
defaults to creating Rust projects. Other languages are additive.

Architecture (v0.3):
- Backend: Lang enum in languages/mod.rs with exhaustive match dispatch.
  Each language has its own module (rust.rs, clang.rs, zig.rs, swift.rs).
  Shared FileLanguage helpers serve flat-directory languages (clang, zig, swift).
  Dispatcher files (playground_commands.rs, export.rs) are thin routers.
  Adding a new language = new module + new enum arm → compiler catches all missing arms.
- Frontend: languages.ts registry with LanguageConfig objects per language.
  Components check capabilities (lang.hasBuildFlags, lang.supportsLiveCheck)
  instead of hardcoded project type checks.
  Templates in separate files per language (templates.ts, zig_templates.ts, etc.).
- Manifest: rustic.toml [project] type field determines everything. Detection
  heuristic: rustic.toml → Cargo.toml → .zig → .swift → "clang".

Edition System (v0.3.3)

Multiple editions ship from one codebase. Each edition is configured by:
1. A Tauri config override (editions/*.json) — productName, identifier, window title
2. VITE_EDITION env var — frontend reads via import.meta.env, selects EditionConfig
3. editions.ts registry — EditionConfig per edition: languages, themes, branding

Edition isolation: each edition has a unique bundle identifier, which gives it
a separate ~/Library/Application Support/ directory. Editions coexist in
/Applications/ with different .app bundle names.

Version management: single VERSION file at project root, synced to Cargo.toml,
tauri.conf.json, and package.json via scripts/sync-version.sh (hooked into
Tauri's beforeDevCommand and beforeBuildCommand).

See specs/build-helper.md for full build/run/distribution instructions.

How It Works

1. App launches → frontend loads in WKWebView
2. Sidebar reads playground list via Tauri command → list_playgrounds()
3. User selects playground → load_playground(name) returns file contents → Monaco loads it
   → snapshot_playground(name) ensures a .saved/ baseline exists
4. User edits → changes held in editor state, live checking writes dirty code to source file
5. User hits Save (⌘S) →
   a. save_playground(name, content) writes to source file AND .saved/ snapshot
6. User hits Run (or ⌘R) →
   a. run_playground(name) spawns cargo run --bin <name> as a child process
   b. stdout/stderr streamed line-by-line via Tauri Channel → output panel
7. User hits Revert →
   a. revert_playground(name) reads .saved/ snapshot, restores source file, returns code
   b. Frontend updates editor with clean code, clears dirty flag
8. User closes dirty tab without saving →
   a. Frontend calls revert_playground to restore the source file on disk
9. Status indicator updated throughout: idle → compiling → running → done/error

Live Check Pipeline
1. User types → frontend debounces 300ms (+ one-in-flight queuing)
2. check_playground(name, code) saves code to source file, spawns cargo check --bin <name> --message-format json
3. Uses separate target/check-runs/ directory to avoid lock conflicts with cargo run
4. Parses JSON diagnostic output (file, line, col, message, severity)
5. Streams diagnostics via Tauri Channel → frontend converts to Monaco editor markers
6. Monaco renders squiggles + hover tooltips inline

Saved Snapshot System (.saved/)

Problem: check_playground writes dirty editor code to the source file on disk so
cargo can see it. This means the on-disk file drifts from the user's last explicit
save. Without protection, closing the app or reverting would lose the clean version.

Solution: a `.saved/` directory alongside the source files holds snapshot copies
of each playground at its last explicitly saved state.

Directory layout (Rust example):
  src/bin/
  ├── hello.rs          ← working copy (cargo reads this, check_playground writes here)
  └── .saved/
      └── hello.rs      ← last explicitly saved version (snapshot)

For flat-directory languages (C/C++, Zig, Swift) the .saved/ directory sits inside
the project root alongside the source files.

Lifecycle:
  Open tab     → snapshot_playground: creates .saved/<name> if it doesn't exist
  User types   → check_playground writes dirty code to source file (cargo needs it)
  Save (⌘S)    → save_playground writes to source file AND updates .saved/ snapshot
  Revert       → revert_playground reads .saved/ snapshot, restores source file,
                  returns clean code to frontend → editor updates
  Close dirty  → frontend calls revert_playground to restore source file on disk
  Rename       → rename_playground also renames the .saved/ snapshot
  Delete       → delete_playground also deletes the .saved/ snapshot
  New          → new_playground creates both source file and initial .saved/ snapshot

Commands (playground_commands.rs):
  snapshot_playground(name)  — ensure .saved/ snapshot exists (idempotent)
  revert_playground(name)    — read .saved/, write to source, return code
  save_playground(name, code) — write to source + .saved/ (atomic update)

Export safety: all export functions filter source files by extension (.rs, .c, .cpp,
.zig, .swift). The .saved/ directory is never included in exports.

Listing safety: list_playgrounds filters by file extension or is_file() check.
The .saved/ directory never appears as a playground.


Output Streaming
- stdout lines  → Channel message { stream: "stdout", line: "..." }
- stderr lines  → Channel message { stream: "stderr", line: "..." }
- process exit  → Channel message { stream: "complete", code: i32 }
- Frontend renders each stream with a different colour

Theme System
- Eight themes: System / Light / Dark / Auto (match language) / Rust / Clang / Zig / Swift
- System mode tracks prefers-color-scheme media query
- Auto mode switches theme based on active project's language type
- CSS custom properties on body (.theme-dark / .theme-light / .theme-rust / .theme-seagreen / .theme-zig / .theme-swift) for app chrome
- Monaco has paired themes per language
- Both sync reactively via Svelte $effect
- Each language theme has a distinct accent color and palette

Toolchain Detection & Setup
- Rust (required at startup):
    1. User's configured cargo_path from settings
    2. ~/.cargo/bin/cargo
    3. which cargo (PATH lookup)
  - If found: read version via cargo --version
  - If not found: show first-run setup wizard (Rust tab)
  - All cargo invocations use the resolved absolute path
- Clang C/C++ (detected on demand):
    1. xcrun --find clang
    2. clang --version for display
  - Not required at startup — detected when first Clang project is created
  - Wizard has a separate C/C++ tab showing status and install instructions
- Zig (detected on demand):
    1. which zig or PATH lookup
    2. zig version for display
  - Targets Zig 0.15.x — version_ok check in check_toolchain
  - Other versions may have breaking stdlib API changes (yellow warning in wizard/pill)
  - Not required at startup
- Swift (detected on demand):
    1. swiftc --version (ships with Xcode CLI tools)
  - Not required at startup — same install as clang (xcode-select --install)

Sibling tool resolution (rustup, rustc, rustfmt, cargo-clippy):
- macOS app bundles launched from /Applications get a minimal PATH that excludes
  ~/.cargo/bin — bare command names like "rustup" won't resolve
- Fix: derive the bin directory from the resolved cargo path via .parent(), then
  resolve sibling tools as absolute paths (e.g. /Users/x/.cargo/bin/rustup)
- This works regardless of install location — if cargo is at /opt/rust/bin/cargo,
  siblings resolve to /opt/rust/bin/rustup, etc.
- Falls back to bare name (PATH lookup) if the absolute path doesn't exist
- No separate settings for each tool — rustup puts all binaries in the same
  directory, so the cargo path is the single source of truth

---

v0 — CLI Runner (archived, see specs/archive/)

Approach
- Each playground is a standalone binary in src/bin/<name>.rs with a fn main() entry point.
- Cargo auto-discovers every .rs file in src/bin/ as a binary target.
- The runner (src/main.rs) is a thin clap CLI: list, interactive pick, run.
- cargo run <name> delegates to cargo run --bin <name>.

Why replaced
- CLI requires a terminal — adds friction for casual use.
- No editor, no output history, no persistent state.
- The Tauri app preserves the same src/bin/ playground model underneath — only the
  interface changes.

---

Pre-v0 — build.rs + macro approach (deleted, was playground-rs-alt/)

The original implementation used build.rs to scan src/ for *_playground.rs files,
generated _playgrounds.rs with a declarative macro, and dispatched via pub fn run().
Required: build.rs + include!() + custom macro + paste crate.
Replaced because Cargo's src/bin/ auto-discovery handles everything natively.
