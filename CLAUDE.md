# CLAUDE.md — Rustic Playground

> Orientation guide for Claude Code CLI. Read this before making any changes.
> For the active feature spec, read `specs/specifications.md`.

---

## What This Is

**Rustic Playground** — a macOS desktop app for running Rust experiments, inspired by Swift Playgrounds. Write code, press ⌘R, see output stream live. No terminal required.

- Each **project** is a Cargo workspace with its own `Cargo.toml`
- Each **playground** is a `.rs` file in `src/bin/` — a standalone `fn main()` binary
- The app compiles with `cargo run --bin <name>` and streams stdout/stderr in real time

---

## Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| App shell | Tauri 2 | Rust backend, WKWebView frontend |
| Frontend | Svelte 5 (runes) | `$state`, `$derived`, `$effect` — NOT Svelte 4 options API |
| Editor | Monaco | `architecture.md` references React + Monaco — React is stale, Monaco is correct |
| Build | Vite + pnpm | Frontend bundler |
| Backend | Rust (modular) | Tauri commands split across lib.rs + 6 modules |
| Permissions | Tauri 2 capabilities | `src-tauri/capabilities/default.json` |

> **Important:** `specs/architecture.md` was the v1.0 design doc — now updated to reflect the actual implementation. The codebase uses Svelte 5 + Monaco.

---

## Build Commands

```sh
# Development (hot reload)
cargo tauri dev

# Production build
cargo tauri build

# Backend checks only (fast — no frontend needed)
cd src-tauri && cargo check
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy -- -D warnings

# Frontend
cd ui && pnpm install
cd ui && pnpm dev          # standalone Vite dev server (no Tauri)
cd ui && pnpm build        # production bundle
```

---

## Data Storage

All runtime data lives under:
```
~/Library/Application Support/com.rustic-playground.app/
├── config.json              ← active_project, cargo_path, wizard_completed
├── window-state.json        ← window geometry, panel sizes, open tabs
└── projects/
    └── <project-name>/
        ├── Cargo.toml
        ├── src/bin/
        │   └── <playground>.rs
        └── content/         ← runtime assets (PLAYGROUND_CONTENT env var)
```

---

## Repository File Map

| Path | Purpose |
|---|---|
| `src-tauri/src/lib.rs` | App state, paths, validation, config/settings, window state, run() entry (~550 lines) |
| `src-tauri/src/playground_commands.rs` | Project CRUD + playground CRUD + run/kill/check/cancel/stdin (~450 lines) |
| `src-tauri/src/cargo_commands.rs` | Cargo.toml management, toolchain checks, setup wizard (~300 lines) |
| `src-tauri/src/content_commands.rs` | Content file CRUD commands (~120 lines) |
| `src-tauri/src/export.rs` | CLI_MAIN_RS const + export_project command (~230 lines) |
| `src-tauri/src/menu.rs` | macOS menu bar builder + rebuild_menu command (~150 lines) |
| `src-tauri/src/book_chapters.rs` | `seed_rust_book` command — all 20 Rust Book chapter data (~2,700 lines) |
| `src-tauri/tauri.conf.json` | App config: identifier, window defaults, bundle settings |
| `src-tauri/capabilities/default.json` | Tauri 2 IPC permissions — every API call needs an entry here |
| `ui/src/App.svelte` | Root layout, all global state, menu event listeners, window state persistence |
| `ui/src/lib/Sidebar.svelte` | Project/playground/file tree, drag-drop, context menus |
| `ui/src/lib/Editor.svelte` | Monaco wrapper, theme sync, diagnostics markers |
| `ui/src/lib/Output.svelte` | Console panel, run blocks, streaming output |
| `ui/src/lib/SettingsModal.svelte` | Settings panel (editor, appearance, toolchain) |
| `ui/src/lib/NewPlaygroundModal.svelte` | New playground dialog with template picker |
| `ui/src/lib/templates.ts` | 11 starter templates with auto-deps |
| `ui/src/lib/HelpModal.svelte` | Help overlay (⌘⇧/) |
| `ui/src/lib/AboutModal.svelte` | About dialog |
| `ui/src/app.css` | Global CSS variables (dark/light theme definitions) |
| `specs/specifications.md` | Active spec — read before any feature work |
| `specs/roadmap.md` | Released / in-progress / next-up / parked ideas |
| `specs/conventions.md` | Naming rules, code style |
| `specs/workflow.md` | Workflow steps, spec lifecycle, change checklist |
| `specs/archive/` | Historical specs — read-only context |
| `ONBOARDING.md` | Project history, data model deep-dive, known gotchas table |

---

## Current Version Status

**v0.1.8.1** — shipped (production testing fixes: toolchain detection on app bundles, serde_json template dep, stop-and-run confirmation dialog, menu items for Copy Code/Export Project/Rename Playground, menu sync with active tab, rustup.rs link in wizard).

**v0.1.8** — shipped (new app icon, Rust theme, live error checking, dark/light/system themes, export, Rust Book polish, backend modularization, 70 unit tests, stdin fix, light theme readability, live theme preview).

After v0.1.8.1: pause features, ship website, DMG distribution, wiki, announcements.

---

## Critical Tauri / WKWebView Gotchas

These will burn you if you don't know them. From hard-won experience:

### 1. Tauri 2 Capabilities — every API call needs a permission
Every `window.*`, `dialog.*`, etc. call from JS must be explicitly allowed in `src-tauri/capabilities/default.json`. Calls that aren't listed silently fail (no error in console). When adding new Tauri API usage, check the capabilities file first.

```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    "core:window:allow-set-size",
    "core:window:allow-set-position",
    ...
  ]
}
```

### 2. WKWebView eats `window.confirm` and standard dialogs
`window.confirm()`, `window.alert()`, `window.prompt()` do nothing in WKWebView. Use Tauri dialog plugin or build custom modal UI instead.

### 3. macOS keyboard events don't reach WKWebView for system shortcuts
⌘R, ⌘S, ⌘N etc. are intercepted by macOS before the WebView sees them. They must be wired through Tauri's menu system (`MenuItemBuilder`) and received via `listen('menu:<event>', ...)` in the frontend.

### 4. Menu `.enabled()` is baked at build time — no post-hoc toggling
`build_menu()` in `lib.rs` reconstructs the entire menu on every state change. Menu items cannot be enabled/disabled after construction. Always rebuild the menu when enabled states need to change.

### 5. Process kill requires the full process tree
`cargo run` spawns child processes. Killing just the `cargo` PID leaves the compiled binary running. Use `kill_pg` (kill process group) or recurse through children. The current implementation uses SIGTERM then SIGKILL with a 500 ms grace period.

### 6. Serde `u32` rejects JavaScript floats silently
JavaScript drag math produces floats (e.g. `324.5`). Rust `u32` will not deserialize a float — serde fails and the `invoke()` call throws. Always `Math.round()` numeric values before passing them to Tauri commands that expect integer types. This affects all panel sizes in `saveWindowState()`.

### 7. `window.innerWidth/Height` vs Tauri window API
Use `window.innerWidth` / `window.innerHeight` (browser API) to read window dimensions. The Tauri `outerSize()` and `scaleFactor()` APIs require capabilities and are unnecessary for this use case.

### 8. `_restoring` guard for save-during-restore
During `onMount`, restoring state (setting `sidebarW`, `outputH`, etc.) triggers reactive `$effect` watchers that would call `saveWindowState()` before the UI is fully restored. Guard with a `let _restoring = true` flag, set to `false` after restore completes.

---

## Svelte 5 Conventions

This codebase uses **Svelte 5 runes** exclusively. Do not use Svelte 4 syntax.

```svelte
<!-- State -->
let count = $state(0)
let doubled = $derived(count * 2)

<!-- Side effects -->
$effect(() => { /* runs when deps change */ })

<!-- Props -->
let { onclose }: { onclose: () => void } = $props()

<!-- Bindable props -->
let { value = $bindable() } = $props()
```

Never use `export let`, `$:`, or `writable()` stores in new code.

---

## Conventions

From `specs/conventions.md`:

- **Playground names:** `src/bin/<name>.rs` — lowercase, no suffix (not `name_playground.rs`)
- **Entry point:** always `fn main()`, never `pub fn run()`
- **Self-contained:** no shared modules between playgrounds
- **Dependencies:** lean — all playgrounds share one `Cargo.toml`
- **Naming:** descriptive of the concept, not the chapter/source
- **Code style:** idiomatic Rust, clear names, comments only where behavior is non-obvious
- **Before committing:** `cargo fmt` + `cargo clippy -- -D warnings`

---

## Backend Pattern — Adding a New Tauri Command

```rust
// 1. Define the command in src-tauri/src/lib.rs
#[tauri::command]
async fn my_command(arg: String, app: AppHandle) -> Result<String, String> {
    // ...
    Ok("result".to_string())
}

// 2. Register in invoke_handler (bottom of lib.rs)
tauri::generate_handler![
    // ... existing commands ...
    my_command,
]

// 3. If it uses a new Tauri API, add the capability to:
// src-tauri/capabilities/default.json
```

---

## Menu Pattern — Adding a Menu Item

All menus are built in `build_menu()` in `lib.rs`. To add a new item:

1. Add the `MenuItemBuilder` call in the appropriate submenu
2. Add the `id → "menu:event-name"` mapping in the menu event handler
3. Add `listen('menu:event-name', handler)` in `App.svelte`'s unlistener block

---

## Workflow

1. Read `specs/specifications.md` before any feature work
2. Read `specs/architecture.md` for structural decisions (note: it's stale re: editor — see tech stack above)
3. Prefer small, focused diffs
4. Run `cargo fmt` + `cargo clippy -- -D warnings` before finalizing
5. When a spec iteration is complete: archive to `specs/archive/spec-v<N>.md`, refresh `specs/specifications.md`

---

## Roadmap Summary

**Parked (deliberately):**

- **v2.0 — Native Swift + Monaco hybrid:** SwiftUI app with Monaco in an isolated WKWebView for the editor pane only. All other UI (sidebar, output, toolbar, menus) native SwiftUI/AppKit. Parked until current Tauri version hits a genuine hard wall or feels "done." See `specs/roadmap.md` for full architecture diagram.

- **v3.0 — Containerization:** Use Apple's `apple/containerization` framework (macOS 26+) to run playground code in an OCI container — true isolation without App Sandbox limitations. Long-term safety investment.

- **v4.0 — Multi-language:** Python, Go, etc. via the same project/playground model. Far future.

**Next up (v0.1.7):** dependency manager UI, playground templates, output improvements.

---

## Security Model

- Playground names validated as `[a-z][a-z0-9_]*` — path traversal blocked at API layer
- Tauri IPC only accepts calls from the app's own WebView origin
- `cargo` invoked via absolute path (`~/.cargo/bin/cargo`), never shell string interpolation
- The app is **intentionally unsandboxed** — like Xcode/Terminal, it must execute arbitrary code
- Playground runs use `target/playground-runs/` to avoid lock conflicts with `cargo check`

---

## Attribution

The **Rust Book Examples** (loaded via Help → Load Rust Book Examples…) are based on
_The Rust Programming Language_ by Steve Klabnik and Carol Nichols.
License: MIT / Apache-2.0. © Rust Project Developers (2010).
Source: https://github.com/rust-lang/book

Playground code is original educational Rust — not verbatim from the book.
An `attribution.md` is placed in every chapter's `content/` folder.
