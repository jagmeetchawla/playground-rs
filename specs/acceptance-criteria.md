ACCEPTANCE CRITERIA

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
    Given: User has Rust installed (e.g. via rustup) at ~/.cargo/bin/cargo
    When:  App launches for the first time
    Then:  Wizard shows "Rust <version> found at <path>" with [Use this] and [Choose different]

[ ] Use detected toolchain
    Given: Wizard shows a detected toolchain
    When:  User clicks [Use this]
    Then:  Path is saved to app config, wizard dismissed, app loads normally

[ ] Custom path accepted
    Given: Wizard or settings panel is open
    When:  User picks a path via file picker and it contains a valid cargo binary
    Then:  Path saved, version shown, app uses that cargo for all invocations

[ ] Install via rustup succeeds
    Given: No Rust toolchain detected, user clicks [Install via rustup]
    When:  rustup-init download and install completes
    Then:  Progress streamed to wizard UI, toolchain path auto-detected on completion,
           app proceeds to main UI

[ ] Install progress is visible
    Given: Rustup install is in progress
    When:  rustup-init is running
    Then:  Wizard shows streaming output / progress indicator — not a frozen screen

[ ] Settings shows current toolchain
    Given: Toolchain is configured
    When:  User opens Settings (Cmd+,)
    Then:  Current cargo path and Rust version are displayed

[ ] Re-detect works
    Given: Settings panel is open
    When:  User clicks [Detect again]
    Then:  App re-scans for cargo, updates path and version if found

[ ] Invalid path rejected
    Given: User selects a path that does not contain a valid cargo binary
    When:  App attempts to validate it
    Then:  Error message shown, previous valid config preserved

---

LIVE ERROR CHECKING

[ ] Errors appear as squiggles
    Given: A playground is loaded with a type or syntax error
    When:  User stops typing for 500ms
    Then:  Red squiggle appears under the error in Monaco

[ ] Warnings appear as squiggles
    Given: A playground has a compiler warning (e.g. unused variable)
    When:  User stops typing for 500ms
    Then:  Amber squiggle appears under the warning in Monaco

[ ] Hover tooltip shows message
    Given: A squiggle is visible in the editor
    When:  User hovers over it
    Then:  Tooltip shows the cargo check error/warning message

[ ] Squiggles clear when error is fixed
    Given: An error squiggle is showing
    When:  User fixes the error and pauses typing
    Then:  Squiggle disappears

[ ] Live check does not execute code
    Given: A playground with a side effect (file write, network call)
    When:  cargo check runs in the background
    Then:  No side effect occurs — check only, no binary produced or run

[ ] Status shows checking
    Given: User has paused typing
    When:  cargo check is running in the background
    Then:  Status indicator shows "checking"

---

EDITOR

[ ] Select playground loads file
    Given: Sidebar shows playground list
    When:  User clicks a playground name
    Then:  Editor loads the file contents with Rust syntax highlighting

[ ] Unsaved changes indicator shown
    Given: A playground is loaded and clean
    When:  User edits the code
    Then:  A dot (●) appears next to the playground name in the sidebar

[ ] Unsaved indicator clears on save
    Given: A playground has unsaved changes (● visible)
    When:  User saves with Cmd+S or runs with Cmd+R
    Then:  The ● disappears

[ ] Edit persists on run
    Given: A playground is loaded in the editor
    When:  User edits code and clicks Run
    Then:  Changes are saved to src/bin/<name>.rs before execution

[ ] Cmd+S saves without running
    Given: A playground has unsaved changes
    When:  User presses Cmd+S
    Then:  File is written to disk, ● clears, no execution

---

PLAYGROUND MANAGEMENT

[ ] New playground created
    Given: User clicks [+ New] or presses Cmd+N
    When:  User enters a valid name and confirms
    Then:  src/bin/<name>.rs is created with fn main() template, loads in editor,
           appears in sidebar alphabetically

[ ] Playground renamed
    Given: User right-clicks a playground in the sidebar and selects Rename
    When:  User enters a new name and confirms
    Then:  src/bin/<old>.rs renamed to src/bin/<new>.rs, sidebar updates, editor reloads

[ ] Playground deleted
    Given: User right-clicks a playground and selects Delete
    When:  User confirms the deletion dialog
    Then:  src/bin/<name>.rs is removed, playground disappears from sidebar,
           editor clears if it was selected

[ ] Delete cancelled
    Given: Delete confirmation dialog is showing
    When:  User clicks Cancel
    Then:  Nothing changes

[ ] Playground duplicated
    Given: User right-clicks a playground and selects Duplicate
    When:  Action executes
    Then:  src/bin/<name>_copy.rs is created with same content, appears in sidebar,
           loads in editor

---

OUTPUT

[ ] stdout shown in output panel
    Given: A playground that prints to stdout
    When:  User runs it
    Then:  stdout lines appear in output panel in white, streaming line by line

[ ] stderr shown in output panel
    Given: A playground that writes to stderr
    When:  User runs it
    Then:  stderr lines appear in output panel in red

[ ] Compiler errors shown in output panel
    Given: A playground with a syntax or type error
    When:  User runs it
    Then:  Compiler error output appears in amber, run does not execute

[ ] Output streams live
    Given: A playground with a sleep or loop
    When:  User runs it
    Then:  Output appears line by line as produced, not buffered until completion

[ ] Clear output works
    Given: Output panel has content
    When:  User clicks [✕ Clear]
    Then:  Output panel is emptied

[ ] Previous output cleared on new run
    Given: Output panel has content from a previous run
    When:  User runs a playground
    Then:  Output panel clears before new output starts streaming

---

RUN / STOP

[ ] Run button disabled while running
    Given: A playground is currently running
    When:  User looks at the toolbar
    Then:  Run button is replaced by Stop button (■ Stop) — never both visible

[ ] Stop kills running playground
    Given: A playground is running
    When:  User clicks Stop or presses Cmd+.
    Then:  Process is killed, output panel shows "— stopped —", status returns to idle

[ ] Cmd+R triggers run
    Given: A playground is loaded and status is idle
    When:  User presses Cmd+R
    Then:  Playground runs, same as clicking Run

[ ] Cmd+. stops running playground
    Given: A playground is running
    When:  User presses Cmd+.
    Then:  Same as clicking Stop

---

STATUS

[ ] Status shows compiling
    Given: User clicks Run
    When:  cargo is compiling
    Then:  Status indicator shows "compiling"

[ ] Status shows running
    Given: Compilation succeeded
    When:  Binary is executing
    Then:  Status indicator shows "running"

[ ] Status shows error
    Given: Compilation failed or process exited non-zero
    When:  Run completes
    Then:  Status indicator shows "error" in red

[ ] Status returns to idle
    Given: Run completed successfully
    When:  Process exits with code 0
    Then:  Status indicator returns to "idle"

---

SETTINGS

[ ] Settings opens
    Given: App is running
    When:  User presses Cmd+, or clicks hamburger → Settings
    Then:  Settings panel opens showing toolchain, appearance sections

[ ] Theme changes apply immediately
    Given: Settings is open
    When:  User switches theme (dark / light / system)
    Then:  App re-renders in selected theme without restart

[ ] Font size change applies to editor
    Given: Settings is open
    When:  User changes editor font size
    Then:  Monaco editor updates font size immediately

[ ] Settings persist across restarts
    Given: User changed theme and font size
    When:  App is restarted
    Then:  Settings are restored to the values set in the previous session

---

UI / UX

[ ] Panels are resizable
    Given: App is open
    When:  User drags the divider between panels
    Then:  Panels resize, layout persists across app restarts

[ ] Sidebar is collapsible
    Given: App is open
    When:  User presses Cmd+\ or clicks the toggle
    Then:  Sidebar hides, editor expands; toggle again to restore

[ ] Dark mode follows system
    Given: macOS dark mode is enabled and theme is set to System
    When:  App launches
    Then:  App renders in dark mode

[ ] App ships as .app
    Given: cargo tauri build is run
    When:  Build completes
    Then:  A standalone .app is produced that runs without a terminal

---

Notes
- Output streaming is the most critical UX criterion — validate early.
- Stop (Cmd+.) is macOS convention — same as stopping a process in Terminal.
- Panel resize + window state persistence: use Tauri store plugin.
- Unsaved dot (●) follows VS Code / RustRover convention.
- Each criterion maps to a manual test step for v1.0; automated tests in v2.0.
