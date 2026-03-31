SPECIFICATION

Status
- Version: v1.2 draft
- Date: 2026-03-31
- Owner: Jagmeet Chawla

---

Visual References

Swift Playgrounds reference (UI target)
  specs/assets/swift-playgrounds-reference.png
  Shared 2026-03-30. Drove the v1.1 macOS dark colour system, blue pill sidebar,
  file tabs, RS badge, and 'playground-dark' Monaco theme.

Annotated feedback screenshot (v1.2 requirements source)
  specs/assets/v1.2-annotated-feedback.png
  Shared 2026-03-31. Shows the running app with six red-box annotations that define
  all v1.2 work. Each section below maps directly to one annotation.

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

┌─────────────────────────────────────────────────────────────────────────┐
│  RS  Rust Playground  │  [💾 Save]  │  /path/to/cargo ▾  │  [▶ Run]    │
├──────────────┬──────────────────────────────┬──────────────────────────┤
│  Playgrounds │  [tab] hello  [tab] chapter3 │  Console                 │
│  ───────── │  ──────────────────────────  │                           │
│  🔍 Filter  │                              │  ┌─ Run #3 ──────────── ▾┐│
│             │   fn main() {                │  │ ▶ cargo run hello      ││
│  RS hello   │     println!("hi");          │  │ hi                     ││
│  RS chapter3│   }                          │  └───────────────────────┘│
│  RS chapter4│                              │                           │
│  ...        │                              │  ┌─ Run #2 ──────────── ▸┐│
│             │                              │  │ (collapsed)            ││
│  ─────────  │                              │  └───────────────────────┘│
│  Cargo.toml │                              │                           │
└──────────────┴──────────────────────────────┴──────────────────────────┘

---

Toolbar (v1.2)

Left:   RS badge + "Rust Playground" app name
Centre: [💾 Save] button — explicit save, always visible, disabled when clean
        Cargo/Rust path display — shows active toolchain path, click to pick different
        distribution or toolchain (stable / beta / nightly / custom path)
Right:  [▶ Run] / [■ Stop] — never both visible

Save button behaviour:
- Always visible in toolbar (not hidden behind Cmd+S only)
- Disabled (greyed) when no unsaved changes in active tab
- Enabled (white) when active tab is dirty
- Cmd+S remains as keyboard shortcut — both routes do the same thing

Toolchain picker (click on path in toolbar):
- Popover lists: detected toolchains (stable, beta, nightly if installed via rustup)
  plus any custom paths previously saved
- [Browse…] option opens file picker for a custom cargo binary
- Selected toolchain stored in app config, used for all cargo invocations
- Shows Rust version next to each entry (e.g. "stable  1.78.0")

---

Sidebar (v1.2)

Top section — playground list (unchanged from v1.1):
- Search/filter bar
- Playground items with RS badge, blue pill selection
- Right-click context menu: Rename, Duplicate, Delete

Bottom section — Cargo.toml viewer:
- Pinned at the bottom of the sidebar, always visible
- Shows the workspace Cargo.toml (the one that governs all playgrounds)
- Read-only view by default — syntax highlighted, scrollable
- Double-click or [Edit] button to open in editor tab for editing
- Useful for: checking/adding dependencies without leaving the app

Divider between the two sections is a fixed separator — not draggable in v1.2.

---

New Playground flow (fix)

Current behaviour (broken): the "+ Add playground from the sidebar" empty-state
hint text does not work — clicking it or the + button does not reliably open a
new playground.

Required behaviour:
- Sidebar + button (SVG icon, top-right of sidebar header) opens an inline name
  input in the sidebar list
- Cmd+N does the same
- Input validated on Enter: must match [a-z][a-z0-9_]*, max 64 chars
- On confirm: creates src/bin/<name>.rs with fn main() template, opens as new tab
- On Escape or blur with empty input: cancels silently
- Empty state "create a new one" link in editor area triggers the same flow

---

Keyboard shortcuts (verify all work)

The empty state shortcut grid shows ⌘N / ⌘R / ⌘S. All three must work:

Cmd+N   New playground — opens inline name input in sidebar
Cmd+R   Run active playground (save first, then cargo run)
Cmd+S   Save active playground — same as clicking Save button
Cmd+.   Stop running playground
Cmd+W   Close active tab (prompt if dirty)
Cmd+,   Open settings
Cmd+\   Toggle sidebar

Each shortcut must work when focus is in the editor, sidebar, or output panel.
Monaco intercepts keys by default — Cmd+R and Cmd+S are already overridden;
verify Cmd+N is not consumed by Monaco when editor is focused.

---

Console — block-based interface (v1.2)

Current behaviour: a flat scrolling list of lines, appended per run with a
divider. Hard to distinguish compiler output from program output; older runs
are equally prominent as new ones.

New behaviour: each run is a self-contained collapsible block.

Block structure (one block per run):

  ┌─ Run #3  ▶ cargo run hello  2026-03-31 14:41  ─────────────────── ▾ ┐
  │                                                                       │
  │  ┌─ Compiler ──────────────────────────────────────────────────── ▾ ┐│
  │  │  Compiling hello v0.1.0                                          ││
  │  │  Finished in 0.4s                                                ││
  │  └──────────────────────────────────────────────────────────────────┘│
  │                                                                       │
  │  ┌─ Output ────────────────────────────────────────────────────── ▾ ┐│
  │  │  Hello, world!                                                   ││
  │  └──────────────────────────────────────────────────────────────────┘│
  │                                                                       │
  └───────────────────────────────────────────────────────────────────────┘

Collapsing rules:
- Latest run: expanded by default (both Compiler and Output sub-blocks open)
- Previous runs: outer block collapses to header only on new run start
- User can manually expand/collapse any block at any time
- Collapsed header shows: run number, command, timestamp, exit status (✓ or ✗)

Sub-blocks within a run:
- Compiler — cargo compile output (stdout/stderr from cargo itself before the
  binary runs): "Compiling…", "Finished", "error[E…]" lines
- Output — stdout and stderr from the running binary itself
- If compilation fails: Compiler block shows errors in red, Output block absent

Error/warning styling within Compiler block:
- error[E…] lines: red
- warning: lines: amber
- note: lines: dim white
- Finished / Compiling lines: dim (tertiary text colour)

Clear button:
- Clears all run blocks for the active tab
- Confirmation not required (history is not persisted to disk anyway)

---

Acceptance Criteria (v1.2 additions)

TOOLBAR — SAVE BUTTON
[ ] Save button visible in toolbar at all times when a tab is open
[ ] Save button disabled (greyed) when active tab has no unsaved changes
[ ] Save button enabled when active tab is dirty
[ ] Clicking Save button saves file and clears dirty state — same as Cmd+S
[ ] Cmd+S still works as keyboard shortcut

TOOLBAR — TOOLCHAIN PICKER
[ ] Toolbar shows current cargo path (truncated if long)
[ ] Clicking path opens a popover listing available toolchains
[ ] Each entry shows channel name and Rust version
[ ] Selecting an entry switches the active toolchain for future runs
[ ] [Browse…] opens file picker for custom cargo binary
[ ] Invalid path rejected with error message

NEW PLAYGROUND (fix)
[ ] Clicking + in sidebar header opens inline name input in sidebar list
[ ] Cmd+N opens the same inline input
[ ] Input validates on Enter: [a-z][a-z0-9_]*, max 64 chars
[ ] Invalid name shows inline error message
[ ] Escape cancels without creating anything
[ ] Created playground opens as a new tab immediately
[ ] "create a new one" link in editor empty state triggers the same flow

KEYBOARD SHORTCUTS
[ ] Cmd+N creates new playground when editor is focused
[ ] Cmd+R runs active playground when editor is focused
[ ] Cmd+S saves active playground when editor is focused
[ ] Cmd+. stops running playground
[ ] Cmd+W closes active tab (prompts if dirty)
[ ] All shortcuts work when sidebar or output panel is focused too

CONSOLE — BLOCK INTERFACE
[ ] Each run appears as a distinct collapsible block with header
[ ] Block header shows: run number, command, timestamp, exit status
[ ] Latest run block is expanded by default
[ ] Previous run blocks auto-collapse when a new run starts
[ ] User can manually expand/collapse any block
[ ] Compiler sub-block and Output sub-block are separate within each run
[ ] Compiler errors shown in red within Compiler block
[ ] Warnings shown in amber within Compiler block
[ ] Output sub-block absent when compilation fails
[ ] Clear button removes all blocks for active tab

SIDEBAR — CARGO.TOML
[ ] Cargo.toml section pinned at bottom of sidebar
[ ] Shows workspace Cargo.toml content with syntax highlighting
[ ] Scrollable independently of playground list
[ ] Double-click or [Edit] opens Cargo.toml as an editor tab

---

Exclusions (v1.2)
- No Cargo.toml editing directly inline in the sidebar panel (open as tab instead)
- No per-playground Cargo.toml (workspace model unchanged)
- No toolchain installation from the picker — picker selects from already-installed only
- No drag to reorder run blocks

---

Notes
- Toolchain picker is display + selection only in v1.2; installation via rustup wizard
  remains a v1.x backlog item
- Block-based console is the biggest UX change — implement incrementally: outer run
  blocks first, then Compiler/Output sub-blocks
- Cargo.toml at sidebar bottom is read-mostly; full editing via tab open is sufficient
- Verify Monaco key intercept for Cmd+N specifically — not previously overridden
