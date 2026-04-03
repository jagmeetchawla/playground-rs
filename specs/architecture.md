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
├── playground_commands.rs   — project CRUD (7 commands) + playground CRUD (8) + run/kill/check/cancel/stdin
├── cargo_commands.rs        — Cargo.toml get/save, add/remove dependency, toolchain info/check, wizard
├── content_commands.rs      — content file CRUD (9 commands: list/create/read/save/delete/rename/import/reveal/path)
├── export.rs                — CLI_MAIN_RS const + export_project command
├── menu.rs                  — build_menu + rebuild_menu (macOS menu bar)
├── book_chapters.rs         — seed_rust_book command (20 Rust Book chapter projects)
└── main.rs                  — Tauri entry point

Frontend Structure

ui/src/
├── App.svelte               — root layout, global state, menu listeners, window state, live checking
├── lib/
│   ├── Sidebar.svelte       — project/playground/file tree, drag-drop, context menus
│   ├── Editor.svelte        — Monaco wrapper, theme sync, diagnostics markers
│   ├── Output.svelte        — console panel, run blocks, streaming output
│   ├── SettingsModal.svelte  — settings panel (editor, appearance, toolchain)
│   ├── NewPlaygroundModal.svelte — new playground dialog with template picker
│   ├── HelpModal.svelte     — help overlay (⌘⇧/)
│   ├── AboutModal.svelte    — about dialog
│   └── templates.ts         — 11 starter templates with auto-deps
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

playground_commands.rs (~450 lines):
- Project management: list/get_active/new/switch/rename/delete/duplicate project
- Playground CRUD: list/load/save/new/rename/delete/duplicate playground, workspace_path
- Process management: run_playground (streaming stdout/stderr via Channel), kill_playground (SIGTERM→SIGKILL process group), check_playground (cargo check with JSON diagnostics), cancel_check, send_stdin

cargo_commands.rs (~300 lines):
- Cargo.toml: get/save with TOML validation, add/remove dependency (format-preserving via toml_edit)
- Toolchain: get_toolchain_info, check_toolchain (comprehensive wizard check), complete_wizard

content_commands.rs (~120 lines):
- Content file CRUD: list/create/read/save/delete/rename/import, reveal_in_finder, get_content_file_path

export.rs (~230 lines):
- CLI_MAIN_RS: embedded v0.1 CLI runner (clap-based, interactive picker)
- export_project: exports active project as standalone CLI playground with merged Cargo.toml

menu.rs (~150 lines):
- build_menu: constructs full macOS menu bar (App, Project, Playground, Run, Edit, Help)
- rebuild_menu: Tauri command to reconstruct menu when state changes

book_chapters.rs (~2,700 lines):
- seed_rust_book: creates 20 chapter projects with all playgrounds and attribution

How It Works

1. App launches → frontend loads in WKWebView
2. Sidebar reads playground list via Tauri command → list_playgrounds()
3. User selects playground → load_playground(name) returns file contents → Monaco loads it
4. User edits → changes held in editor state, live checking runs in background
5. User hits Run (or ⌘R) →
   a. save_playground(name, content) writes back to src/bin/<name>.rs
   b. run_playground(name) spawns cargo run --bin <name> as a child process
   c. stdout/stderr streamed line-by-line via Tauri Channel → output panel
6. Status indicator updated throughout: idle → compiling → running → done/error

Live Check Pipeline
1. User types → frontend debounces 300ms (+ one-in-flight queuing)
2. check_playground(name, code) saves code, spawns cargo check --bin <name> --message-format json
3. Uses separate target/check-runs/ directory to avoid lock conflicts with cargo run
4. Parses JSON diagnostic output (file, line, col, message, severity)
5. Streams diagnostics via Tauri Channel → frontend converts to Monaco editor markers
6. Monaco renders squiggles + hover tooltips inline

Output Streaming
- stdout lines  → Channel message { stream: "stdout", line: "..." }
- stderr lines  → Channel message { stream: "stderr", line: "..." }
- process exit  → Channel message { stream: "complete", code: i32 }
- Frontend renders each stream with a different colour

Theme System
- Three modes: System / Light / Dark (persisted in settings)
- System mode tracks prefers-color-scheme media query
- CSS custom properties on body (.theme-dark / .theme-light) for app chrome
- Monaco has paired themes: playground-dark / playground-light
- Both sync reactively via Svelte $effect

Toolchain Detection & Setup
- On launch: check common paths in order:
    1. User's configured cargo_path from settings
    2. ~/.cargo/bin/cargo
    3. which cargo (PATH lookup)
- If found: read version via cargo --version
- If not found: show first-run setup wizard (3-state UI)
- All cargo invocations use the resolved absolute path

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

Pre-v0 — build.rs + macro approach (see playground-rs-alt/)

The original implementation used build.rs to scan src/ for *_playground.rs files,
generated _playgrounds.rs with a declarative macro, and dispatched via pub fn run().
Required: build.rs + include!() + custom macro + paste crate.
Replaced because Cargo's src/bin/ auto-discovery handles everything natively.
