SPECIFICATION

Status
- Version: v1.4 draft (revised)
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

Annotated feedback screenshot (v1.3 requirements source)
  specs/assets/v1.3-annotated-feedback.png
  Three bug fixes: tab close, false-dirty state, Save button position.

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

UI Layout (v1.4)

┌────────────────────────────────────────────────────────────────────────────────────┐
│  RS  Rust Playground    │  ⊙ cargo 1.x.x  │           [💾 Save]  [▶ Run]          │
├─────────────────────┬───────────────────────────────┬──────────────────────────────┤
│  Playgrounds│Content │  [tab] hello2.rs  [tab] …  × │  Console               Clear │
│  ───────────┴─────── │  ─────────────────────────── │                              │
│                      │                              │  ▸ Run #1  cargo run  15:32 ✓ │
│  (Playgrounds tab)   │   fn main() {                │  ▾ Run #2  cargo run  15:34 ✓ │
│  🔍 Filter           │     let dir = env::var(      │    COMPILER                  │
│  RS hello2  ●        │       "PLAYGROUND_CONTENT"); │      Compiling…              │
│  RS chapter3         │     …                        │    OUTPUT                    │
│  RS hello            │   }                          │      Hello!                  │
│  ...                 │                              │                              │
│  ─────────────────── │                              │                              │
│  > Cargo.toml        │                              │                              │
│                      │                              │                              │
│  (Content tab)       │                              │                              │
│  📄 data.txt         │                              │                              │
│  📄 config.json      │                              │                              │
│  🖼 photo.png        │                              │                              │
│  [+ New File]        │                              │                              │
│  [drop files here]   │                              │                              │
└─────────────────────┴───────────────────────────────┴──────────────────────────────┘

---

Sidebar — Two-Tab Design (v1.4)

The left sidebar has two tabs across the top:

  [ Playgrounds ]  [ Content ]

These are the only two views of the sidebar. The tab strip is always visible.
Switching between them never changes the active playground — context is preserved.

──────────────────────────────────────────
Tab 1: Playgrounds  (unchanged from v1.3)
──────────────────────────────────────────

Same as before:
- Search/filter bar
- Playground items: RS badge, blue pill selection, dirty dot ●
- Right-click context menu: Rename, Duplicate, Delete
- + button in header: opens inline name input to create new playground
- Cargo.toml section pinned at bottom (collapsible, Edit button)

No per-playground content file expansion in this tab.
The playground list stays clean — filenames do not appear here.

──────────────────────────────────────────
Tab 2: Content
──────────────────────────────────────────

Shows the content folder for the currently selected playground.
If no playground is selected, shows an empty state: "Select a playground to view
its content files."

Header:
  Content — hello2        ← playground name in subtitle, updates on selection change

File list:
  📄 data.txt
  📄 config.json
  🖼 photo.png
  📦 archive.zip

File type icons (by extension):
  📄  any text-ish: .txt .md .csv .log .toml .yaml .yml .json .xml .html .rs
  🖼  .png .jpg .jpeg .gif .webp .svg
  📦  everything else (binary / unknown)

File interactions:
  Click text file (📄)    → open as editor tab in the main editor area
  Click image file (🖼)   → open with macOS default app (shell::open)
  Click binary file (📦)  → reveal in Finder
  Right-click any file    → context menu: Rename / Delete / Reveal in Finder

New file button:
  [+ New File] pinned at the bottom of the list.
  Opens an inline name input directly in the file list (same pattern as new playground):
    - Text input appears at the bottom of the list, auto-focused.
    - Placeholder: "filename.txt"
    - Enter: validate → create empty file → open as editor tab if text type.
    - Escape / blur with empty input: cancel silently.
    - Duplicate name: inline error "Already exists".
    - Name with / \ or null bytes: inline error "Invalid name".

Drag and drop:
  The entire Content tab pane is a drop zone.
  User drags one or more files from Finder and drops onto the Content tab.
  Each file is copied (not moved) into content/<playground>/.
  Name collision: appended _1, _2… suffix — no silent overwrite.
  A subtle dashed border appears on the pane while files are dragged over it.

Empty content folder state:
  When no files exist yet:

    Drop files here
    or [+ New File]

  Instructional, not just blank.

Auto-switch to Content tab:
  When a user opens a content file from any path (e.g. future file picker),
  the sidebar switches to the Content tab automatically so the file is visible
  in context. The playground tab is not auto-switched.

---

Folder Structure

  workspace/
    Cargo.toml
    src/bin/
      hello2.rs
      chapter3.rs
    content/              ← ONE shared folder, all playgrounds read from here
      data.csv
      config.json
      photo.png

Rules:
- content/ is a single flat directory shared by all playgrounds.
- Created lazily on first file add.
- Renaming or deleting a playground has no effect on content/.
- No per-playground subfolders — keep it simple.

---

Runtime Access — PLAYGROUND_CONTENT env var

When run_playground runs a binary it injects:

  PLAYGROUND_CONTENT=/absolute/path/to/workspace/content

The playground reads files portably:

  use std::{env, fs};

  fn main() {
      let dir = env::var("PLAYGROUND_CONTENT").unwrap_or_default();
      let data = fs::read_to_string(format!("{dir}/data.csv")).unwrap();
      println!("{data}");
  }

New playground template includes a hint comment:
  // Files in your content folder are available via:
  // let dir = std::env::var("PLAYGROUND_CONTENT").unwrap_or_default();

---

Editor Integration (content text files)

Opening a content text file:
- Tab badge: 📄 (not the RS badge used for Rust source files)
- Tab label: just the filename, e.g. "data.txt"
- Language auto-detected by extension:
    .rs → rust   .json → json   .toml → ini   .md → markdown
    .yaml / .yml → yaml          .csv .txt .log and all others → plaintext
- Dirty state and Cmd+S work identically to playground source tabs.
- Save invokes save_content_file(playground_name, filename, content) — not save_playground.
- Closing a dirty content tab: same behaviour as closing a dirty playground tab
  (close immediately, no confirm — file on disk is safe).

---

Backend Commands (new in v1.4)

list_content_files(name: &str) → Result<Vec<ContentFile>>
  ContentFile { filename: String, size_bytes: u64, is_text: bool }
  Sorted alphabetically. Returns empty vec (not error) if folder missing.

create_content_file(name: &str, filename: &str) → Result<()>
  Creates empty file at content/<name>/<filename>.
  Creates content/<name>/ dir if missing.
  Returns Err if filename already exists.

save_content_file(name: &str, filename: &str, content: &str) → Result<()>
  Overwrites with new text content.

read_content_file(name: &str, filename: &str) → Result<String>
  Reads as UTF-8 text.

delete_content_file(name: &str, filename: &str) → Result<()>
  Deletes file. Returns Err if not found.

rename_content_file(name: &str, old: &str, new_name: &str) → Result<()>
  Renames within the same content folder.
  Returns Err if new name already exists.

import_content_file(name: &str, src_path: &str) → Result<String>
  Copies from src_path into content/<name>/.
  Returns final filename (may have _1 suffix if collision resolved).

Security: same path-traversal guard as playground names on all filename parameters.
  - Reject names containing / \ or null bytes.
  - Canonicalize and verify destination stays inside content/<name>/.

run_playground change:
  cmd.env("PLAYGROUND_CONTENT", workspace_dir.join("content").join(name));

---

Acceptance Criteria (v1.4)

SIDEBAR TABS
[ ] Two tabs visible at top of sidebar: "Playgrounds" and "Content"
[ ] Switching tabs does not change the active playground
[ ] Playgrounds tab is unchanged from v1.3
[ ] Content tab header shows "Content — <playground name>"
[ ] Content tab shows empty state when no playground is selected
[ ] Content tab shows empty drop-zone state when folder is empty

CONTENT FILE LIST
[ ] Files listed with correct 📄 🖼 📦 icons
[ ] Clicking text file opens as editor tab
[ ] Clicking image file opens with macOS default app
[ ] Clicking binary file reveals in Finder
[ ] Right-click: Rename / Delete / Reveal in Finder

NEW FILE
[ ] [+ New File] opens inline input at bottom of file list
[ ] Enter validates name and creates file
[ ] Text files open as editor tab immediately on creation
[ ] Escape cancels silently
[ ] Duplicate name shows inline error "Already exists"
[ ] Name with / or \ shows inline error "Invalid name"

DRAG AND DROP
[ ] Entire Content tab pane is a drop zone
[ ] Dashed highlight border appears on drag-over
[ ] Files dropped are copied into content/<playground>/
[ ] Name collision resolved with _1 suffix (no overwrite)
[ ] Multiple files can be dropped at once

EDITOR TABS FOR CONTENT FILES
[ ] Tab shows 📄 badge (not RS)
[ ] Language auto-detected from extension
[ ] Dirty state and Cmd+S work correctly
[ ] save_content_file invoked on save (not save_playground)

RUNTIME
[ ] PLAYGROUND_CONTENT env var injected on run
[ ] Points to correct absolute path for the active playground
[ ] New playground template includes the hint comment

FOLDER STRUCTURE
[ ] content/ created lazily on first file add
[ ] content/ is shared — renaming/deleting a playground does not touch it

BACKEND SECURITY
[ ] Path-traversal guard on all content filenames
[ ] Canonicalized paths verified to stay within content/<name>/

---

Exclusions (v1.4)
- No inline image preview inside the app — delegate to macOS
- No drag to reorder content files
- No content folder deletion on playground delete (v1.4 limitation)
- No shared/global content folder — content is per-playground only
- No binary file editing in Monaco

---

Notes
- The two-tab sidebar design keeps the playground list uncluttered — file browsing
  has its own dedicated space rather than expanding inline under each playground.
  This is the same pattern used by many IDEs (VS Code Explorer vs Search tabs).
- The Content tab always reflects the currently selected playground — it follows
  selection, it does not drive it.
- Drag-and-drop in Tauri requires fileDropEnabled: true in tauri.conf.json under
  the window configuration.
- Monaco language IDs used: "rust", "json", "ini" (TOML), "markdown", "yaml",
  "plaintext". All built-in, no extra plugins needed.
- The PLAYGROUND_CONTENT env var pattern is inspired by CARGO_MANIFEST_DIR — a
  well-known convention for giving subprocesses their own path context.
