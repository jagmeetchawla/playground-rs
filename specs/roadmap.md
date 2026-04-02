ROADMAP
=======

This file tracks in-progress versions, future features, and parked ideas.
Active specs live in specifications.md. Completed specs live in archive/.

---

RELEASED
────────

v0.1.5 — Multiple Projects + Unified Storage
  Status: complete — released 2026-04-01
  See specs/archive/specs-v0.1.5.md


---

IN PROGRESS
───────────

v0.1.6 — Editor Experience + App Polish
  Status: partially complete — released 2026-04-01
  See specs/archive/ for completed spec

  Shipped (5 of 8):
  1. Stop button — actually kill the process (SIGTERM + SIGKILL fallback)
  5. Window state persistence — size, position, tabs, sidebar width
  6. Resizable panels — drag sidebar and output panel borders
  7. Hide Left Panel button — Cmd+Shift+L, matches Safari/Xcode
  8. Layout switch — toggle output panel bottom ↔ right

  NOT shipped (moved to v0.1.7):
  2. Live error checking — cargo check squiggles in CodeMirror
  3. Toolchain setup wizard — first-run rustup detection and install
  4. Settings panel — font, theme, tab size, cargo path (Cmd+,)

v0.1.6.3 — Interactive Console (stdin support)
  Status: complete — released 2026-04-02

  Features:
  1. Interactive stdin — pipe stdin to running playground process
     - Backend: pipe child stdin, store handle, add `send_stdin` Tauri command
     - Frontend: input field in Output panel when program is running
     - User input echoed as visible line in console output
     - Enables playgrounds that use std::io::stdin() for prompts/input

---

IN PROGRESS
───────────

v0.1.7 — Settings, Polish, and Deferred v0.1.6 Features
  Status: in progress — started 2026-04-02

  Priority 1 — Settings panel (Cmd+,)
    - Font size, font family, tab size
    - Theme (system / light / dark)
    - Cargo path override
    - Backend: get_settings / save_settings commands, settings.json in App Support
    - Frontend: SettingsPanel component, slide-in or modal

  Priority 2 — Toolchain setup wizard
    - First-run detection when cargo is not found
    - Offer to run rustup-init or set manual path
    - Store cargo_path and wizard_completed in config.json

  Priority 3 — Live error checking
    - cargo check --message-format json in background (~500 ms debounce)
    - Push diagnostics to CodeMirror as squiggles
    - Separate --target-dir to avoid lock conflicts

  Priority 4 — New features
    - Dependency manager UI (add/remove crates without editing Cargo.toml manually)
    - Playground templates (select a starting point: async, CLI, data, web)
    - Output improvements (copy button, timestamps per line, ANSI colour support)


---

PARKED IDEAS
────────────

These are real, considered proposals that have been deliberately set aside.
Each has a rationale. Revisit when the time is right.

─────────────────────────────────────────────────────────────────────────────
IDEA: Native Swift + Monaco Hybrid (v2.0 rewrite candidate)
─────────────────────────────────────────────────────────────────────────────
Status: Parked — finish current Tauri version first
Logged: 2026-04-01

Background
  The current stack (Tauri + Svelte + WKWebView) has produced recurring classes
  of bugs that are hard to diagnose or fix without deep knowledge of multiple
  layers: DOM event propagation, WKWebView limitations, Tauri v1→v2 API changes,
  Svelte 4 vs 5 idiom drift. The bugs themselves are fixable but the debugging
  path is opaque — you'd need to hold Svelte → WKWebView → Tauri → Rust → macOS
  in your head simultaneously.

  Swift Playgrounds and Xcode both use fully custom native editors (built on
  NSTextView). Those are proprietary and unavailable. The best open-source
  native option is Runestone (Tree-sitter based), but it doesn't match Monaco's
  quality for a code editing tool.

Proposed Architecture
  Native SwiftUI app with one isolated WKWebView for the editor pane only.

  ┌─────────────────────────────────────────────────────────┐
  │  SwiftUI App                                            │
  │                                                         │
  │  ┌──────────────┐  ┌─────────────────┐  ┌───────────┐  │
  │  │ Sidebar      │  │ WKWebView       │  │ Output    │  │
  │  │ (native)     │  │ Monaco editor   │  │ (native)  │  │
  │  │              │  │ (editor only)   │  │           │  │
  │  │ ProjectList  │  │                 │  │ RunBlock  │  │
  │  │ FileList     │  │                 │  │ Console   │  │
  │  └──────────────┘  └─────────────────┘  └───────────┘  │
  │                                                         │
  │  Toolbar: native NSToolbar                              │
  │  Menus:   native NSMenu / commands                      │
  └─────────────────────────────────────────────────────────┘

  - Everything except the editor is native SwiftUI / AppKit
  - The WKWebView hosts only Monaco — no Svelte, no Tauri bridge
  - Swift ↔ Monaco communication via WKScriptMessageHandler (JS → Swift)
    and evaluateJavaScript (Swift → JS) — simple, well-documented, stable
  - Cmd+S, Cmd+R, Cmd+N handled at AppKit level (NSMenuItem / @FocusedValue)
    before they reach the WebView — no keyboard capture fights
  - Drag-drop, file dialogs, context menus all native — no workarounds needed
  - Rust runner stays as a plain Process (Foundation) — same logic, cleaner API
  - Storage: same ~/Library/Application Support layout as current v0.1.x

Why This Is Better
  - Menus, shortcuts, drag-drop, dialogs just work
  - Crashes and errors have clear Swift stack traces
  - Debugging path: Swift → Rust process, nothing else
  - The Monaco WKWebView is isolated — can't interfere with native UI events
  - Code is readable and fixable without knowing Svelte/Tauri internals

Why It's Parked
  - Current Tauri version works and is improving
  - Switching mid-stream loses momentum
  - Swift development is slightly slower (more API lookup needed)
  - Worth doing as a clean v2.0 rewrite once v0.1.x feature set is stable
    and we know exactly what we're rebuilding

Trigger Condition
  Revisit when v0.1.x hits a hard wall — a feature that genuinely can't be done
  cleanly in Tauri — or when the current version feels "done" and a rewrite
  makes sense as a quality investment.

References
  - Runestone: https://github.com/simonbs/Runestone
  - WKScriptMessageHandler docs: Apple Developer Documentation
  - Nova (Panic) uses a similar hybrid native+web-component approach
─────────────────────────────────────────────────────────────────────────────
