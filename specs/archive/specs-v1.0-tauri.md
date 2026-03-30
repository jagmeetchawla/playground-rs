# SPECS ARCHIVE — v1.0 Tauri GUI

```
Status
- Version:  v1.0
- Archived: 2026-03-30
- Era:      Tauri + Svelte + Monaco GUI app (current implementation)
- Source:   git commit 1d21dfc — "feat: add Tauri GUI app — Swift Playgrounds-inspired Rust playground"
- Note:     Archived proactively while specs are still current — so the v1.0 baseline
            is frozen before any v1.x or v2.0 updates are made to the active spec files.
```

---

## SPECIFICATION

SPECIFICATION

Status
- Version: v1.0 draft
- Date: 2026-03-29
- Owner: Jagmeet Chawla

---

Product

What
A native macOS desktop app — built with Tauri — that wraps the existing Rust playground
runner in a Swift Playgrounds-inspired UI. Write Rust, see errors live, hit Run, see
output. Nothing else needed.

Why
The CLI runner works well but requires a terminal. A GUI removes that friction, makes the
playground feel like a first-class tool, and opens it up to a wider audience. The Swift
Playgrounds model is the right reference point: clean, focused, distraction-free, with
code on one side and output on the other.

---

UI Layout

Three-panel layout, inspired by Swift Playgrounds:

```
┌──────────────────────────────────────────────────────────┐
│  toolbar: [≡]  Rust Playground  [● idle]  [▶ Run / ■ Stop] │
├──────────────┬──────────────────────┬─────────────────────┤
│  sidebar     │   editor (Monaco)    │  output panel        │
│              │                      │                       │
│  • hello     │  fn main() {         │  > cargo run hello    │
│  ● chapter3  │      println!(...)   │  Hello, world!        │
│  • chapter4  │  }                   │                       │
│  • ...       │  ~~~ error squiggle  │                       │
│  [+ New]     │                      │  [✕ Clear]            │
└──────────────┴──────────────────────┴─────────────────────┘
```

● = unsaved changes indicator

Panels:
- Sidebar (left)   — playground list, new button, right-click menu, collapsible
- Editor (center)  — Monaco editor, Rust syntax highlighting, live error squiggles
- Output (right)   — stdout (white), stderr (red), compiler errors (amber), streaming

Toolbar:
- App name / hamburger menu (left) — opens settings
- Status indicator: idle / checking / compiling / running / error (center)
- Run button → becomes Stop button while running (never both visible at once)
- Keyboard: Cmd+R to run, Cmd+. to stop, Cmd+S to save, Cmd+N to new playground

---

Playground Management

Sidebar right-click context menu on any playground:
- Rename — inline rename in sidebar, updates src/bin/<old>.rs → src/bin/<new>.rs
- Delete — confirmation dialog, removes src/bin/<name>.rs
- Duplicate — creates src/bin/<name>_copy.rs, selects it in editor

New playground ([+ New] button):
- Prompts for name (inline input in sidebar)
- Creates src/bin/<name>.rs with fn main() template
- Loads it in editor, ready to type

Empty state (no playgrounds):
- Editor area shows: "No playgrounds yet — click [+ New] to create your first one"
- Sidebar shows only the [+ New] button

---

Live Error Checking

cargo check runs automatically in the background as you type (debounced 500ms after
last keystroke). Errors and warnings are surfaced inline in Monaco as squiggles — red
for errors, amber for warnings — with hover tooltips showing the message.

This is not live execution. No binary is compiled or run. cargo check is type-checking
only — fast, no side effects. The same data RustRover (JetBrains) uses under the hood.

Levels:
- v1.0  cargo check debounced on change → Monaco markers (inline squiggles + hover)
- v2.0  rust-analyzer LSP → full inline hints, completions, go-to-definition (RustRover-level)

---

Toolchain Setup

On first launch the app detects whether a Rust toolchain is present and guides the user
through setup if needed. The goal: zero manual terminal steps for a new user.

First-run wizard (shown once, skipped on subsequent launches if toolchain is configured):

  App launches → detect Rust
        │
        ├─ Found → "Rust <version> found at <path>"
        │           [Use this]  [Choose a different path]
        │
        └─ Not found → "Rust is not installed"
                        [Install via rustup]       ← downloads + runs rustup-init, streams progress
                        [Choose existing path]     ← file picker for custom / nvm-style installs
                        [Open rustup.rs manually]  ← opens browser, user installs themselves

Install via rustup:
- Downloads rustup-init for the current platform from static.rust-lang.org
- Runs it non-interactively (rustup-init -y --no-modify-path)
- Streams install progress to the wizard UI
- On completion, verifies cargo is callable and records the path

Settings panel (hamburger menu → Settings):
- Toolchain: current cargo path + version, [Detect again], [Change path]
- Appearance: theme (dark / light / system), editor font size, tab size
- Toolchain path stored in Tauri app config, used for all cargo invocations

---

Settings

Accessible via hamburger menu (top-left) or Cmd+,:

Toolchain
- Current cargo path and Rust version
- [Detect again] — re-runs detection
- [Change path] — file picker

Appearance
- Theme: Dark / Light / System (default: System)
- Editor font size: 12 / 13 / 14 / 16 / 18 (default: 14)
- Tab size: 2 / 4 (default: 4)

---

Keyboard Shortcuts

Cmd+R        Run current playground
Cmd+.        Stop running playground
Cmd+S        Save current playground (without running)
Cmd+N        New playground
Cmd+W        Close / deselect current playground
Cmd+,        Open settings
Cmd+\        Toggle sidebar

---

Constraints

Product
- macOS only for v1.0
- CLI/script playgrounds only — fn main(), stdout/stderr output
- Live error checking via cargo check — not live execution
- Explicit Run button / Cmd+R for execution
- Playground files are the source of truth — editor saves back to src/bin/<name>.rs

Technical
- Tauri 2.0
- Svelte + Vite for frontend
- Monaco Editor for code editing
- Rust toolchain: detected, installed via rustup, or user-specified — not bundled
- App ships as a .app / .dmg

---

Exclusions
- No live execution — running the binary on every keystroke
- No rust-analyzer LSP in v1.0 — cargo check is sufficient
- No inline result values (Swift Playgrounds AI feature)
- No dependency management UI — edit Cargo.toml manually for now
- No debugger
- No git integration
- No multi-file playgrounds
- No Windows / Linux support in v1.0
- No bundled Rust toolchain — toolchain is detected, installed via rustup, or user-specified

---

Notes
- Svelte chosen over React for leanness — less boilerplate, no virtual DOM
- Monaco is the right editor — Rust syntax highlighting built in, LSP-ready for v2.0
- RustRover (JetBrains) is the target experience for v2.0 — rust-analyzer LSP
- cargo check debounce: 500ms is the right balance — responsive but not thrashing
- Output streaming is critical UX — do not wait for process to finish before showing output
- Stop button replaces Run button (never both) — Cmd+. is macOS stop convention
- Panel resize drag-and-drop, persistent across sessions (Tauri store plugin)
- Window size + position persisted across restarts
- Unsaved dot (●) follows VS Code / RustRover convention — familiar to developers

---

## ARCHITECTURE

v1.0 — Tauri Desktop App

Stack
- Tauri 2.0       — app shell, Rust backend, macOS WKWebView
- Svelte + Vite   — frontend UI framework (lean, no virtual DOM)
- Monaco Editor   — code editor (same engine as VS Code)
- Rust backend    — existing runner logic exposed as Tauri commands

Project Structure

```
playground-rs/
├── src/
│   ├── main.rs              — Tauri app entry point
│   └── bin/                 — playgrounds (unchanged from CLI era)
│       ├── hello.rs
│       ├── chapter3.rs
│       └── ...
├── ui/                      — Svelte + Monaco frontend
│   ├── src/
│   │   ├── App.svelte       — root layout (sidebar + tabbar + editor + output)
│   │   ├── lib/Sidebar.svelte
│   │   ├── lib/TabBar.svelte
│   │   ├── lib/Editor.svelte
│   │   └── lib/Output.svelte
│   ├── index.html
│   └── package.json
├── src-tauri/               — Tauri config and Rust backend
│   ├── src/lib.rs           — all Tauri commands
│   ├── entitlements.plist   — macOS entitlements (non-sandboxed developer tool)
│   └── tauri.conf.json
├── specs/
├── Cargo.toml               — workspace root
└── README.md
```

How it works

1. App launches → frontend loads in WKWebView
2. Sidebar reads playground list via Tauri command → list_playgrounds()
3. User selects playground → opens as a tab; load_playground(name) returns file contents
4. User edits → changes held in per-tab state (tabCode map)
5. User hits Run (or Cmd+R) →
   a. save_playground(name, content) writes back to src/bin/<name>.rs
   b. run_playground(name, channel) spawns cargo run --bin <name> --target-dir target/playground-runs
   c. stdout/stderr streamed line-by-line via Tauri Channel API → per-tab output panel
6. Status indicator updated: idle → compiling → running → done/error (per tab)

Tauri Commands
- list_playgrounds()                    → Vec<String>
- load_playground(name)                 → String
- save_playground(name, content)        → Result
- new_playground(name)                  → Result
- rename_playground(old_name, new_name) → Result
- delete_playground(name)              → Result
- duplicate_playground(name)           → Result<String>
- run_playground(name, on_output)       → streams via Channel<serde_json::Value>
- workspace_path()                      → String

Security
- Non-sandboxed (developer tool — explicit opt-out via entitlements.plist)
- Path traversal prevention: name whitelist validation ([a-z][a-z0-9_]*, max 64 chars)
  + canonicalized path check against bin_dir
- Big red warning in README — compile from source only, no binary distributed
- Storage: dev = src/bin/, production = ~/Library/Application Support/com.playground-rs.app/

Separate target-dir
- cargo run --target-dir target/playground-runs prevents Cargo.lock conflict
  between tauri dev (which holds the main target/) and playground compilation

---

v1 CLI (archived — see specs-v1-cli.md)

src/bin/ approach with clap CLI runner. Replaced by the Tauri GUI.
src/bin/ playground files are preserved unchanged as the storage format.

---

v0 (archived — see specs-v0.md)

build.rs + paste crate + _playground.rs convention. Replaced by src/bin/ approach.

---

## ACCEPTANCE CRITERIA

Version
- Spec version: v1.0
- Last updated: 2026-03-29

Overview
Tauri desktop app — Swift Playgrounds-inspired Rust playground for macOS.
CLI/script playgrounds only (fn main() + stdout/stderr). Explicit run button, no live execution.

---

LAUNCH

[ ] App launches
    Given: macOS machine with Rust toolchain configured
    When:  User opens playground-rs.app
    Then:  App window opens, sidebar shows all playgrounds from src/bin/, editor is empty

[ ] Playground list loads
    Given: src/bin/ contains .rs files
    When:  App launches
    Then:  Sidebar lists all playground names (without .rs extension), alphabetically sorted

[ ] Empty state shown
    Given: src/bin/ contains no .rs files
    When:  App launches
    Then:  Sidebar shows only [+ New], editor area shows "No playgrounds yet — click [+ New]"

[ ] Window state restored
    Given: User resized and repositioned the window in a previous session
    When:  App launches
    Then:  Window opens at the same size and position as last time

---

TOOLCHAIN SETUP

[ ] Existing toolchain detected
[ ] Use detected toolchain
[ ] Custom path accepted
[ ] Install via rustup succeeds
[ ] Install progress is visible
[ ] Settings shows current toolchain
[ ] Re-detect works
[ ] Invalid path rejected

---

LIVE ERROR CHECKING

[ ] Errors appear as squiggles
[ ] Warnings appear as squiggles
[ ] Hover tooltip shows message
[ ] Squiggles clear when error is fixed
[ ] Live check does not execute code
[ ] Status shows checking

---

EDITOR

[ ] Select playground loads file
[ ] Unsaved changes indicator shown
[ ] Unsaved indicator clears on save
[ ] Edit persists on run
[ ] Cmd+S saves without running

---

PLAYGROUND MANAGEMENT

[ ] New playground created
[ ] Playground renamed
[ ] Playground deleted
[ ] Delete cancelled
[ ] Playground duplicated

---

OUTPUT

[ ] stdout shown in output panel
[ ] stderr shown in output panel
[ ] Compiler errors shown in output panel
[ ] Output streams live
[ ] Clear output works (Clear button wipes only the active tab's history)
[ ] Output preserved per tab across tab switches
[ ] Runs append with divider rather than replacing previous output

---

RUN / STOP

[ ] Run button disabled while running
[ ] Stop kills running playground
[ ] Cmd+R triggers run
[ ] Cmd+. stops running playground

---

STATUS

[ ] Status shows compiling
[ ] Status shows running
[ ] Status shows error
[ ] Status returns to idle
[ ] Status is per-tab (switching tabs shows that tab's actual status)

---

SETTINGS

[ ] Settings opens
[ ] Theme changes apply immediately
[ ] Font size change applies to editor
[ ] Settings persist across restarts

---

UI / UX

[ ] Panels are resizable
[ ] Sidebar is collapsible
[ ] Dark mode follows system
[ ] App ships as .app
[ ] File tabs open per playground (TabBar)
[ ] Tab close (×) prompts if dirty
[ ] Cmd+W closes active tab
