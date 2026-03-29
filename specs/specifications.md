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

Architecture (see architecture.md for full detail)

- Tauri 2.0     — app shell, Rust backend, macOS WKWebView
- Svelte        — frontend UI framework (lean, no virtual DOM)
- Monaco Editor — code editor (same engine as VS Code)
- Backend       — existing runner logic wrapped as Tauri commands
- Playgrounds   — src/bin/<name>.rs, unchanged from CLI version
- Streaming     — Tauri event system streams stdout/stderr line by line to the UI
- Live check    — cargo check runs debounced, errors piped back as Monaco markers

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

Acceptance Criteria
See acceptance-criteria.md

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
