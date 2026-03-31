SPECIFICATION

Status
- Version: v1.4 draft
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

┌─────────────────────────────────────────────────────────────────────────────────────┐
│  RS  Rust Playground    │  ⊙ cargo 1.x.x  │            [💾 Save]  [▶ Run]          │
├──────────────────┬──────────────────────────────┬──────────────────────────────────┤
│  Playgrounds     │  [tab] hello2.rs  [tab] …  × │  Console                  Clear  │
│  ─────────────── │  ─────────────────────────── │                                  │
│  🔍 Filter       │                              │  ▸ Run #1  cargo run…  15:32 ✓   │
│                  │   fn main() {                │  ▾ Run #2  cargo run…  15:34 ✓   │
│  RS hello2   ▾   │     let path = content_dir() │    COMPILER                      │
│  ├ 📄 data.txt   │       + "/data.txt";         │      Compiling…                  │
│  ├ 📄 config.json│     …                        │    OUTPUT                        │
│  └ [+ Add File]  │   }                          │      Hello from hello2!!         │
│                  │                              │                                  │
│  RS chapter3 ▸   │                              │                                  │
│  RS hello    ▸   │                              │                                  │
│  ...             │                              │                                  │
│  ─────────────── │                              │                                  │
│  > Cargo.toml    │                              │                                  │
└──────────────────┴──────────────────────────────┴──────────────────────────────────┘

---

Feature: Playground Content Folder (v1.4)

Motivation
Some playgrounds need supporting files to be useful — configuration, sample data, CSV
files, JSON fixtures, images. Swift Playgrounds solves this with an "assets" folder
per playground. We solve it the same way: each playground gets a `content/` subfolder
in the workspace. Files placed there are accessible to the running program at runtime.

---

Folder Structure

  workspace/                    ← ~/Library/Application Support/…/workspace/
    Cargo.toml
    src/
      bin/
        hello2.rs
        chapter3.rs
        …
    content/                    ← NEW: one subfolder per playground
      hello2/
        data.txt
        config.json
      chapter3/
        input.txt
      …

Rules:
- The `content/` folder at workspace root is created lazily (on first file add).
- Each playground's subfolder `content/<name>/` is created when the playground is
  created or on first file add (whichever comes first).
- Content folders are not deleted when a playground is deleted — user must clean
  up manually in v1.4. (Future: prompt on playground delete.)
- Renaming a playground renames its content subfolder atomically with the source rename.

---

Runtime Access — PLAYGROUND_CONTENT env var

When `run_playground` executes a binary, the runner injects:

  PLAYGROUND_CONTENT=/absolute/path/to/workspace/content/<name>

The binary then reads files portably:

  use std::env;

  fn main() {
      let dir = env::var("PLAYGROUND_CONTENT").unwrap_or_default();
      let data = std::fs::read_to_string(format!("{dir}/data.txt")).unwrap();
      println!("{data}");
  }

The app auto-generates a helper comment at the top of every new playground:

  // Content folder: use env::var("PLAYGROUND_CONTENT") to get the path.

This hint only appears in the fn main() template — it is not injected at runtime or
added to existing playgrounds.

---

Sidebar — Content Files Section

The sidebar playground list item expands to show its content files.

Collapsed state (default):
  RS hello2  ▸        ← chevron shows content is collapsed, badge shows file count if >0
                        e.g. RS hello2  ▸ [2]

Expanded state (active playground or manually expanded):
  RS hello2  ▾
  ├ 📄 data.txt
  ├ 🖼 photo.png
  └ [+ Add File]      ← always last item; opens add-file flow

File type icons (by extension):
  📄  .txt .md .csv .log .toml .yaml .yml .json .xml .html .rs (any text-ish)
  🖼  .png .jpg .jpeg .gif .webp .svg
  📦  everything else (binary / unknown)

Interactions:
- Click on a text file (📄)     → open as editor tab (language auto-detected by ext)
- Click on image file (🖼)      → open in a simple preview panel (v1.4 basic: just open
                                   with macOS default app via shell::open)
- Click on binary file (📦)     → reveal in Finder
- Right-click on any file       → context menu: Rename / Delete / Reveal in Finder
- Drag file from Finder         → accepted; copies file into content/<name>/ folder
- [+ Add File]                  → opens the add-file flow (see below)

Only the selected playground auto-expands its content section. Others stay collapsed.
Clicking the chevron on any playground toggles its content section independently.

---

Add File Flow

Two entry points both show the same inline UI in the sidebar:

  A) [+ Add File] in the content section
  B) "New file" option in future (v1.5+) — placeholder only in v1.4

Inline flow:
1. A text input appears below the last file, pre-focused.
2. Placeholder: "filename.txt"
3. On Enter: validate the name (see below), create the file, open as editor tab.
4. On Escape / blur with empty name: cancel silently.

File name validation:
- Must not be empty.
- Must not contain path separators (/ or \) or null bytes.
- No length limit beyond filesystem constraints (255 chars).
- No restriction on extension or case — anything is valid.
- If a file with that name already exists: show inline error "Already exists".

File creation:
- Creates an empty file at content/<name>/<filename>.
- For text-ish extensions (.txt .md .rs .json .csv .toml etc.): opens as editor tab.
- For others: just creates the file, shows it in the list.

Import existing file (drag-and-drop):
- User drags a file from Finder onto the playground item in the sidebar.
- File is copied (not moved) into content/<name>/.
- If a file with the same name exists: append _1, _2, … suffix (no silent overwrite).
- Tauri file drop events handle the drag target; the drop zone is the entire sidebar
  item row (or the content sub-list if already expanded).

---

Editor Integration

Opening a content text file in an editor tab:
- Tab label: just the filename (e.g. "data.txt") — no RS badge, use a 📄 badge instead.
- Language auto-detected from extension:
    .rs → rust,  .json → json,  .toml → ini (Monaco),  .md → markdown,
    .csv .txt .log → plaintext,  .yaml .yml → yaml,  all others → plaintext
- Saved via a new backend command `save_content_file(name, filename, content)`.
- Dirty state + Cmd+S both work exactly as for playground files.
- Tab shows dirty ● if unsaved, clears on save.

---

Backend Commands (new in v1.4)

list_content_files(name: &str) → Result<Vec<ContentFile>>
  ContentFile { filename: String, size_bytes: u64, is_text: bool }
  Returns files in content/<name>/ sorted alphabetically.
  Returns empty vec if folder doesn't exist (not an error).

create_content_file(name: &str, filename: &str) → Result<()>
  Creates an empty file at content/<name>/<filename>.
  Creates the content/<name>/ dir if missing.
  Returns Err if filename already exists.

save_content_file(name: &str, filename: &str, content: &str) → Result<()>
  Overwrites content/<name>/<filename> with new text content.

read_content_file(name: &str, filename: &str) → Result<String>
  Reads content/<name>/<filename> as UTF-8 text.

delete_content_file(name: &str, filename: &str) → Result<()>
  Deletes the file. Returns Err if not found.

rename_content_file(name: &str, old: &str, new: &str) → Result<()>
  Renames within same content folder. Returns Err if new name already exists.

import_content_file(name: &str, src_path: &str) → Result<String>
  Copies file from src_path into content/<name>/. Returns final filename
  (may differ from original if a name collision was resolved with _1 suffix).

Security: all filenames pass through the same path-traversal guard as playground names:
  - Reject names containing / \ or null bytes.
  - Canonicalize destination path and verify it stays inside content/<name>/.

---

run_playground change

In the existing run_playground command, before spawning the cargo process, set:

  cmd.env("PLAYGROUND_CONTENT", workspace_dir.join("content").join(name))

No other change to the run flow.

---

Acceptance Criteria (v1.4)

FOLDER STRUCTURE
[ ] content/ folder created at workspace root on first file add
[ ] content/<name>/ subfolder created per playground
[ ] Renaming a playground renames its content subfolder atomically
[ ] Deleting a playground does NOT delete its content folder (v1.4 limitation, noted)

RUNTIME
[ ] PLAYGROUND_CONTENT env var is set when running any playground
[ ] Value is the absolute path to content/<name>/
[ ] Playground can read files via env::var("PLAYGROUND_CONTENT")
[ ] New playground template includes the content-dir hint comment

SIDEBAR — CONTENT SECTION
[ ] Active playground auto-expands to show its content files
[ ] Collapsed playground shows file count badge if it has content files
[ ] 📄 🖼 📦 icons shown by file type
[ ] [+ Add File] button at bottom of content list
[ ] Right-click on file: Rename, Delete, Reveal in Finder

ADD FILE FLOW
[ ] [+ Add File] opens inline name input in sidebar
[ ] Enter creates file and opens as editor tab (if text type)
[ ] Escape cancels silently
[ ] Duplicate filename shows inline error
[ ] Filename with path separators (/) rejected

DRAG AND DROP
[ ] Dragging a file from Finder onto a playground item copies it to content/<name>/
[ ] Name collision resolved with _1 suffix (no silent overwrite)

EDITOR TAB FOR CONTENT FILES
[ ] Text content files open as editor tabs
[ ] Tab shows 📄 badge instead of RS badge
[ ] Language detected from file extension
[ ] Dirty state and Cmd+S save work correctly
[ ] save_content_file invoked (not save_playground)

IMAGE / BINARY FILES
[ ] Clicking image file opens with macOS default app (shell::open)
[ ] Clicking binary file reveals in Finder

BACKEND SECURITY
[ ] All content filenames validated against path-traversal attack
[ ] Canonicalized paths verified to stay within content/<name>/ boundary

---

Exclusions (v1.4)
- No inline image preview in the app itself — delegate to macOS
- No drag-and-drop reordering of content files
- No content folder size limits or quotas
- No deletion of content folder on playground delete (warn only)
- No shared/global content folder — content is per-playground only
- No binary file editing in Monaco

---

Notes
- The PLAYGROUND_CONTENT env var pattern matches how many tools expose context to
  subprocesses (e.g. CARGO_MANIFEST_DIR). It's simple, portable, and requires no
  magic from the runner.
- Drag-and-drop in Tauri uses the file-drop feature of the webview — must be enabled
  in tauri.conf.json: `"fileDropEnabled": true` under windows config.
- Monaco language IDs: "rust", "json", "ini" (for TOML), "markdown", "yaml",
  "plaintext". All are built into Monaco, no extra plugins needed.
- Content files are not tracked in Cargo.toml or any build manifest — they are
  purely runtime data accessed by the binary.
