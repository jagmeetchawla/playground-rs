SPECIFICATION

Status
- Version: v1.3 draft
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
  Shared 2026-03-31. Six red-box annotations that defined all v1.2 work.

Annotated feedback screenshot (v1.3 requirements source)
  specs/assets/v1.3-annotated-feedback.png
  Shared 2026-03-31. Three red-arrow annotations on the running v1.2 app.
  All three are bug fixes or small UX adjustments.

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

┌─────────────────────────────────────────────────────────────────────────────────┐
│  RS  Rust Playground    │  ⊙ cargo 1.x.x  │              [💾 Save]  [▶ Run]    │
├──────────────┬──────────────────────────────┬──────────────────────────────────┤
│  Playgrounds │  [tab] hello  [tab] hello2 × │  Console                  Clear  │
│  ─────────── │  ─────────────────────────── │                                  │
│  🔍 Filter   │                              │  ▸ Run #1  cargo run…  15:32 ✓   │
│              │   fn main() {                │  ▾ Run #2  cargo run…  15:34 ✓   │
│  RS hello    │     println!("hi");          │    COMPILER                      │
│  RS hello2   │   }                          │      Compiling…                  │
│  RS chapter3 │                              │      Finished…                   │
│  ...         │                              │    OUTPUT                        │
│              │                              │      Hello from hello2!!         │
│  ─────────── │                              │                                  │
│  Cargo.toml  │                              │                                  │
└──────────────┴──────────────────────────────┴──────────────────────────────────┘

---

Toolbar (v1.3)

Left:   RS badge + "Rust Playground" app name
Centre: Cargo/Rust toolchain pill — shows detected cargo version (read-only display
        for v1.3; toolchain picker popover deferred to v1.4)
Right:  [💾 Save]  [▶ Run] / [■ Stop]
        Save and Run are always on the right side together, visually grouped.

Change from v1.2:
  Save button moved from toolbar centre to toolbar right, placed immediately
  left of the Run/Stop button. This keeps all action buttons in one zone and
  frees the centre for the toolchain info display only.

Save button behaviour (unchanged from v1.2):
- Always visible when a tab is open
- Disabled (greyed) when active tab has no unsaved changes
- Enabled when active tab is dirty
- Cmd+S still works as a keyboard shortcut

---

Bug Fixes (v1.3)

1. TAB CLOSE BUTTON DOES NOT WORK

  Root cause: window.confirm() is not rendered by Tauri's WKWebView. When a tab
  is dirty and the user clicks ×, the confirmation dialog is called but returns
  false silently — so the early-return guard always fires and the tab never closes.
  Compounded by the false-dirty bug (see #2) which makes every tab appear dirty.

  Fix:
  - Remove window.confirm() from closeTab().
  - Just close the tab immediately. The source file is not deleted — it stays on
    disk. The user loses only in-editor unsaved edits, which is acceptable for a
    playground. The dirty indicator (●) in the tab bar provides enough warning.
  - (Future: add a native Tauri confirm via @tauri-apps/plugin-dialog if the
    team decides the confirmation is valuable.)

2. SAVE INDICATOR (●) SHOWS UP EVEN WHEN NO CHANGES ARE MADE

  Root cause: Monaco's editor.getModel().setValue(code) — called when the user
  switches tabs to sync the editor value — fires onDidChangeModelContent. This
  triggers the dispatch('change', ...) in Editor.svelte, which calls onCodeChange()
  in App.svelte, which pushes the tab name into dirtyTabs. Result: every tab is
  immediately marked dirty on first load, before the user has typed anything.

  Fix:
  - Add an ignoreNextChange flag in Editor.svelte.
  - Set it to true immediately before calling setValue().
  - In the onDidChangeModelContent listener, check the flag: if true, clear it
    and return without dispatching. The next real user keystroke clears the flag
    and dispatches normally.

3. SAVE BUTTON POSITION (UX)

  Described above in Toolbar section. Save moves to right, next to Run.

---

Acceptance Criteria (v1.3)

TAB CLOSE
[ ] Clicking × on any tab closes it immediately, regardless of dirty state
[ ] No confirm() dialog required
[ ] Closing the active tab switches focus to adjacent tab (left or right)
[ ] Closing the last tab shows the empty state

DIRTY STATE
[ ] Opening a playground tab does NOT mark it dirty
[ ] Switching between tabs does NOT mark either tab dirty
[ ] Typing in the editor marks the tab dirty (● appears)
[ ] Saving clears the dirty state
[ ] Dirty ● appears in: tab bar, sidebar playground item

TOOLBAR — SAVE POSITION
[ ] Save button is on the RIGHT side of the toolbar, left of Run/Stop
[ ] Toolbar centre shows only the toolchain pill (no Save button there)
[ ] Save and Run are visually grouped together on the right

---

Exclusions (v1.3)
- No native confirmation dialog (deferred until plugin-dialog is added)
- No toolchain picker popover (toolchain pill is display-only, deferred to v1.4)
- No drag-to-reorder tabs

---

Notes
- The window.confirm() / window.prompt() family are unreliable in Tauri's WKWebView
  without explicit webview dialog configuration. Avoid using them; prefer inline UI
  (input fields, Svelte-rendered modals) or the @tauri-apps/plugin-dialog Tauri API.
- The false-dirty bug is subtle but high-impact: it breaks tab close, pollutes the
  dirty indicator, and could cause unnecessary save prompts. Fix it by guarding the
  Monaco change listener.
- After v1.3 all three annotations from the screenshot should be resolved.
