SPECIFICATION

Status
- Version: v0.1.6
- Date: 2026-04-01
- Owner: Jagmeet Chawla

---

Visual References

  specs/assets/swift-playgrounds-reference.png
  specs/assets/v1.2-annotated-feedback.png
  specs/assets/v1.3-annotated-feedback.png

---

Product

What
  A native macOS desktop app — built with Tauri — that wraps the existing Rust playground
  runner in a Swift Playgrounds-inspired UI. v0.1.6 focuses on the editor experience
  and app polish: actually killing running processes, live compiler feedback, first-run
  setup, persistent settings, resizable panels, and improved layout control.

Why
  The core loop (write → run → see output) works. v0.1.6 makes it feel like a real tool:
  Stop does what it says, squiggles surface errors without running, the app remembers
  your layout, and panels resize without reloading.

---

Feature 1 — Stop Button: Actually Kill the Process
───────────────────────────────────────────────────

Problem
  The current Stop button only resets the status label in the frontend.
  The `cargo run` process continues running — output keeps arriving and the binary
  is not terminated.

Goal
  Clicking Stop (or Cmd+.) kills the active child process immediately.

Backend (lib.rs)

  State
    struct RunningProcess(Mutex<Option<u32>>)   // child PID
    // Managed via app.manage() alongside ActiveProject

  New command
    #[tauri::command]
    fn kill_playground(app: AppHandle) -> Result<(), String>

    - Reads the PID from RunningProcess state
    - Sends SIGTERM; waits up to 500 ms for exit
    - If still alive, sends SIGKILL
    - Clears the stored PID

  run_playground changes
    - After spawn(), store child.id() in RunningProcess state
    - On process exit (either complete or killed), clear the stored PID
    - Use tokio::process::Child.kill() for async kill support

Frontend (App.svelte)

  stop() change
    - await invoke('kill_playground')  instead of just resetting status
    - Backend kill triggers the channel's "complete" event (exit code -1 / 130),
      which already updates the RunBlock status — no duplicate state management needed

  Edge cases
    - If no process is running, kill_playground returns Ok(()) silently
    - Cmd+. and Stop button both call the same stop() function

---

Feature 2 — Live Error Checking (cargo check squiggles)
────────────────────────────────────────────────────────

Goal
  After the user stops typing for ~500 ms, run `cargo check` silently in the background
  and push compiler diagnostics to Monaco as red/yellow squiggles with hover messages.
  This mirrors what rust-analyzer does but uses the project's own toolchain directly.

Backend

  New command
    #[tauri::command]
    async fn check_playground(
        name: String,
        on_diagnostics: Channel<serde_json::Value>,
        app: AppHandle,
    ) -> Result<(), String>

    - Runs: cargo check --bin <name> --message-format json
        --target-dir  <workspace>/target/check-runs   ← separate dir, no lock conflict
    - Parses JSON output lines; for each "compiler-message" with spans:
        {
          "type": "diagnostic",
          "file": "src/bin/<name>.rs",
          "line": 5,          // 1-based
          "col": 8,           // 1-based
          "end_line": 5,
          "end_col": 12,
          "severity": "error" | "warning",
          "message": "cannot borrow `x` as mutable..."
        }
    - Sends a final { "type": "done" } message when complete

  Separate target-dir
    check-runs/ is used exclusively by check_playground.
    run_playground continues to use playground-runs/.
    No Cargo.lock contention between live check and explicit Run.

  Cancellation
    If check_playground is called again while a previous check is running,
    the old process is killed before the new one starts.
    Store the check child PID in CheckProcess(Mutex<Option<u32>>) app state.

Frontend

  Editor.svelte changes
    - After every code change, reset a 500 ms debounce timer
    - When the timer fires, call invoke('check_playground', { name, onDiagnostics })
    - On each { type: "diagnostic" } message, call Monaco's
      editor.setModelMarkers() with the converted IMarkerData
    - On { type: "done" }, replace markers atomically (clear old, set new)
    - On tab switch, clear markers for the old tab and restore saved markers
      for the new tab (store per-tab in a Map<tabId, IMarkerData[]>)

  Marker mapping
    severity "error"   → monaco.MarkerSeverity.Error
    severity "warning" → monaco.MarkerSeverity.Warning
    Lines/cols are 1-based in both cargo and Monaco — no conversion needed

  No check for non-playground tabs
    Cargo.toml and content files don't run cargo check.

---

Feature 3 — Toolchain Setup Wizard
───────────────────────────────────

Goal
  On first launch (or when cargo is not found), show a one-time setup screen that
  detects rustup/cargo and offers to install or locate them. Never blocks on a
  broken toolchain silently.

Detection (startup, lib.rs)

  fn detect_toolchain() -> ToolchainStatus
    enum ToolchainStatus {
        Found { cargo_path: String, version: String },
        NotFound,
    }
  - Checks ~/.cargo/bin/cargo, then PATH via `which cargo`
  - Returns NotFound if neither exists or if `cargo --version` fails

  emit("toolchain:status", status) during setup()
  Frontend listens and conditionally shows the wizard screen.

Wizard screen (SetupWizard.svelte)

  States:
    detecting → found | not_found | installing | done | error

  "Not found" view
    - Message: "Rust is not installed."
    - Primary button: "Install rustup" → runs rustup-init with -y via shell
    - Secondary link: "I already have Cargo — set path manually"
    - Manual path input: file picker → validate → save to config.json

  "Install" flow
    - Streams install output via a Channel (same pattern as run_playground)
    - On success: re-detect, update toolchainInfo state, dismiss wizard
    - On error: show error, offer retry or manual path

  Config persistence
    - cargo_path stored in config.json alongside active_project
    - cargo_path() in lib.rs checks config first, then falls back to auto-detect

  Shown only once
    - wizard_completed: bool in config.json
    - Set to true when a working cargo is confirmed
    - Can be re-triggered from Settings → "Re-run setup wizard"

---

Feature 4 — Settings Panel
───────────────────────────

Goal
  A native-feeling settings panel (not a modal, a slide-in sheet or dedicated panel)
  for editor and app preferences.

Settings stored in config.json (extends existing)

  {
    "active_project": "default",
    "cargo_path": "",               // empty = auto-detect
    "wizard_completed": true,
    "editor": {
      "font_size": 13,
      "font_family": "JetBrains Mono",
      "tab_size": 4,
      "theme": "system"             // "system" | "dark" | "light"
    },
    "preferred_edition": "2021"     // used when creating new projects
  }

Settings.svelte

  Access: toolbar button (gear icon, far right) or Cmd+,
  Layout: slide-in panel from the right, 280 px wide, overlays the editor

  Sections:
    Editor
      Font Size         — number stepper, 10–24, default 13
      Font Family       — text input (any monospace), default "JetBrains Mono"
      Tab Size          — segmented control: 2 | 4
      Theme             — segmented control: System | Light | Dark

    Toolchain
      Cargo Path        — text input + "Browse…" button
                          Empty = auto-detect (shows resolved path as placeholder)
      Rust Edition      — segmented control: 2018 | 2021 | 2024
      Re-run Setup Wizard  — button

  Live preview
    Font size and family changes apply to Monaco immediately (no save needed).
    Theme changes apply immediately.
    Tab size takes effect on the next new playground.

  Persistence
    On blur / change: invoke('save_settings', { settings })  immediately.
    On open: invoke('get_settings') to hydrate panel.

New commands
    get_settings() -> Settings
    save_settings(settings: Settings) -> Result<(), String>

App.svelte integration
  - Editor receives fontsize, fontFamily, tabSize, theme as props
  - Monaco editor.updateOptions() called reactively when they change

---

Feature 5 — Window State Persistence
─────────────────────────────────────

Goal
  Reopen the app in the same state as it was left: same window size, position,
  sidebar width, and open tabs.

What is saved (window-state.json in App Support)

  {
    "window": {
      "x": 200, "y": 150,
      "width": 1280, "height": 800
    },
    "sidebar_width": 220,
    "output_height": 240,         // used in bottom layout
    "output_width": 320,          // used in right layout
    "open_tabs": ["hello", "main"],
    "active_tab": "hello",
    "layout": "bottom"            // "bottom" | "right"
  }

Backend
  New commands:
    get_window_state() -> WindowState
    save_window_state(state: WindowState) -> Result<(), String>

  Window geometry saved via Tauri's window.outer_position() and .outer_size()
  on the "close-requested" event.

Frontend
  - On mount: load window state, restore tab list (re-invoke load_playground per tab),
    sidebar/output dimensions via CSS variables / inline styles
  - On layout change: save immediately
  - On tab open/close: save immediately
  - On window resize: debounced 1s save (don't hammer disk on live resize)

  Tabs restore order but unsaved changes are discarded (no dirty state serialisation).
  If a saved tab no longer exists in the project, it is silently skipped.

---

Feature 6 — Resizable Panels
─────────────────────────────

Goal
  Drag the border between the sidebar and the editor to resize the sidebar.
  Drag the border between the editor and the output panel to resize the output.
  The editor takes all remaining space.

Implementation
  - CSS resize handles: 4 px wide/tall transparent divs with `cursor: col-resize` /
    `cursor: row-resize` placed on the panel borders
  - Mouse drag: pointerdown → pointermove → pointerup on the resize handle
    Updates a CSS variable (--sidebar-width, --output-height / --output-width)
  - CSS grid / flex: the main layout uses the CSS variable as the fixed dimension;
    editor region uses `flex: 1` / `minmax(0, 1fr)` to fill the rest
  - Min/max clamps:
      Sidebar:  min 160 px, max 380 px
      Output:   min 100 px, max 60% of the editor area
  - State is saved via window-state persistence (Feature 5) on drag end

No external drag libraries — pure pointer events, ~50 lines.

---

Feature 7 — Hide Left Panel Button
────────────────────────────────────

Goal
  A toggle button collapses the sidebar entirely, giving the editor full width.
  Matches the pattern in Safari ("Show/Hide Sidebar"), Xcode, and Claude.

Behaviour
  - A small button at the top-left of the toolbar (left of the project switcher)
    shows the sidebar-hide icon (three horizontal lines or the standard macOS
    sidebar toggle icon)
  - Click toggles sidebarVisible: boolean state
  - When hidden: sidebar has width 0, no border, no transition flash
    (use display:none or a CSS class — avoid animating width through 0 as Monaco
    may not resize cleanly)
  - The button icon/tooltip flips: "Show Sidebar" / "Hide Sidebar"
  - Keyboard shortcut: Cmd+Shift+L (matches Xcode)
  - State persisted via window-state (Feature 5): sidebar_hidden: boolean

  When hidden:
    - Toolbar button is the only way to restore (no accidental re-show on resize)
    - The editor area expands to full width immediately

---

Feature 8 — Layout Switch
──────────────────────────

Goal
  Toggle between two output panel positions:
    Bottom  — output panel below the editor (current / default)
    Right   — output panel to the right of the editor (side-by-side)

The layout switch button
  - Lives in the far-right of the toolbar, rightmost element
  - Shows the icon/label for what you'd switch TO (not current state):
      Currently Bottom → button shows "⊡ Side-by-side"  (or a right-panel icon)
      Currently Right  → button shows "⊟ Stacked"       (or a bottom-panel icon)
  - Tooltip: "Switch to stacked layout" / "Switch to side-by-side layout"

Layout implementation
  - layoutMode: 'bottom' | 'right'  state in App.svelte
  - Bottom: main area is a column flex (editor on top, output below)
    Output height = --output-height CSS var (resizable via Feature 6)
  - Right: main area is a row flex (sidebar | editor | output)
    Output width = --output-width CSS var (resizable via Feature 6)
  - CSS classes on .main: layout-bottom / layout-right
  - Monaco needs editor.layout() called after the container resizes —
    do this in a ResizeObserver on the editor wrapper div (already needed for
    general resize handling)
  - State persisted via window-state (Feature 5)

---

Acceptance Criteria

Feature 1 — Stop
  [ ] Clicking Stop during a running process kills it (verified: no more output lines)
  [ ] Cmd+. has the same effect
  [ ] RunBlock shows exit code -1 (or 130) and status "error" / "stopped"
  [ ] Stop when nothing is running does nothing (no error)

Feature 2 — Live Check
  [ ] Red squiggles appear ~500 ms after stopping typing on a syntax error
  [ ] Squiggle hover shows the compiler message text
  [ ] Squiggles clear when the error is fixed
  [ ] Squiggles do not appear on Cargo.toml or content tabs
  [ ] Running `cargo run` while a check is in progress works (no file lock error)

Feature 3 — Setup Wizard
  [ ] On a machine without cargo, the wizard screen is shown on first launch
  [ ] "Install rustup" button streams install output and dismisses on success
  [ ] Manual cargo path is validated (cargo --version) before saving
  [ ] Wizard is not shown again once wizard_completed = true
  [ ] Re-run option available from Settings

Feature 4 — Settings
  [ ] Font size change applies to Monaco immediately (no save/reload needed)
  [ ] Theme change switches Monaco theme immediately
  [ ] Settings survive app restart
  [ ] Cargo path override is used by run_playground and check_playground
  [ ] Cmd+, opens settings panel; Escape closes it

Feature 5 — Window State
  [ ] Window reopens at the same position and size
  [ ] Open tabs are restored with the same active tab
  [ ] If a saved tab no longer exists, it is skipped silently
  [ ] Sidebar width is restored

Feature 6 — Resizable Panels
  [ ] Sidebar can be dragged between 160 px and 380 px
  [ ] Output panel can be dragged in both layout modes
  [ ] Editor resizes cleanly (Monaco re-layouts, no blank area)
  [ ] Resize state persists across restarts

Feature 7 — Hide Sidebar
  [ ] Cmd+Shift+L toggles sidebar
  [ ] Button icon/tooltip updates correctly
  [ ] Editor expands to full width when sidebar is hidden
  [ ] Hidden state persists across restarts

Feature 8 — Layout Switch
  [ ] Button shows correct "switch to" label for both states
  [ ] Switching to Right moves output panel to the right of the editor
  [ ] Switching to Bottom moves output panel below the editor
  [ ] Monaco re-layouts cleanly in both modes
  [ ] Layout preference persists across restarts

---

Implementation Order (suggested)

  1. Feature 1  (Stop kill)      — small backend change, high user impact
  2. Feature 7  (Hide sidebar)   — pure frontend, quick win, big feel improvement
  3. Feature 8  (Layout switch)  — frontend only, builds on sidebar toggle pattern
  4. Feature 6  (Resize panels)  — pointer events + CSS vars, no backend
  5. Feature 5  (Window state)   — backend + frontend wiring
  6. Feature 4  (Settings)       — new panel + backend commands
  7. Feature 3  (Setup wizard)   — conditional first-run flow
  8. Feature 2  (Live check)     — most complex: async debounce + Monaco markers

  The first four are frontend-only and can be done in a single session.
  The last four require backend commands and config schema changes.
