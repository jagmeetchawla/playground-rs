SPECIFICATION

Status
- Version: v0.1.8
- Date: 2026-04-02
- Owner: Jagmeet Chawla

---

Product

What
  A native macOS desktop app — built with Tauri — that wraps the existing Rust playground
  runner in a Swift Playgrounds-inspired UI. v0.1.8 is the final feature release before
  shifting focus to distribution (website, DMG, wiki, announcements). It adds live compiler
  diagnostics, autocomplete via rust-analyzer, light/dark themes, playground export, and
  polishes the Rust Book examples.

Why
  The core editing and running experience is solid after v0.1.7. These five features close
  the gap between "playground" and "real editor": squiggles surface errors without running,
  autocomplete makes exploration faster, themes match user preference, and export lets users
  take their code elsewhere. After this, the app is ready for public release.

Note
  The editor is Monaco (not CodeMirror 6). The CLAUDE.md tech stack table is stale on this
  point — the actual codebase uses Monaco throughout.

---

Feature 1 — Live Error Checking (cargo check squiggles)
────────────────────────────────────────────────────────

Problem
  Users must press Run to discover compile errors. Typos and type mismatches should
  surface immediately as red/yellow squiggles in the editor.

Goal
  After the user stops typing for ~500 ms, run `cargo check` in the background and push
  compiler diagnostics to Monaco as squiggles with hover messages.

Backend (lib.rs)

  New state
    struct CheckProcess(Mutex<Option<u32>>)   // check child PID for cancellation

  New command
    #[tauri::command]
    async fn check_playground(
        name: String,
        on_diagnostics: Channel<serde_json::Value>,
        app: AppHandle,
    ) -> Result<(), String>

    - Saves the current code to disk first (same as run_playground does)
    - Runs: cargo check --bin <name> --message-format json
        --target-dir <workspace>/target/check-runs   (separate dir, no lock conflict)
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

  Cancellation
    If check_playground is called again while a previous check is running,
    kill the old process before starting the new one (same kill_pg pattern
    as run_playground).

Frontend (Editor.svelte)

  - After every code change, reset a 500 ms debounce timer
  - When the timer fires, call invoke('check_playground', { name, onDiagnostics })
  - Collect diagnostics; on { type: "done" }, call Monaco's
    editor.setModelMarkers() atomically (clear old markers, set new batch)
  - On tab switch, clear markers for the old tab and restore cached markers
    for the new tab (Map<string, IMarkerData[]>)

  Marker mapping
    severity "error"   -> MarkerSeverity.Error
    severity "warning" -> MarkerSeverity.Warning
    Lines/cols are 1-based in both cargo and Monaco — no conversion needed.

  Skip non-playground tabs
    Cargo.toml and content file tabs do not trigger cargo check.

  UX
    - No visible spinner for background checks — squiggles just appear/disappear
    - If the user presses Run while a check is in flight, the check is cancelled
      (run takes priority)

---

Feature 2 — Autocomplete / LSP Integration (rust-analyzer)
───────────────────────────────────────────────────────────

Problem
  Users have no code completion, go-to-definition, or inline documentation.
  Exploring Rust APIs requires switching to external docs.

Goal
  Connect Monaco to a rust-analyzer LSP server for completions, hover info,
  and inline signature help. This runs per-project (one rust-analyzer instance
  per open project).

Backend (lib.rs)

  New state
    struct LspProcess(Mutex<Option<tokio::process::Child>>)

  New commands
    #[tauri::command]
    async fn start_lsp(app: AppHandle) -> Result<u16, String>
    // Spawns rust-analyzer with stdio transport, returns nothing
    // (communication happens over stdin/stdout of the child process)

    #[tauri::command]
    async fn stop_lsp(app: AppHandle) -> Result<(), String>

  Approach — LSP proxy over Tauri IPC
    - Backend spawns `rust-analyzer` as a child process with stdio pipes
    - Frontend sends LSP JSON-RPC messages via a Tauri command:
        send_lsp_message(message: String) -> Result<(), String>
    - Backend forwards them to rust-analyzer's stdin
    - Backend reads rust-analyzer's stdout in a loop and emits events:
        app.emit("lsp:message", json_string)
    - Frontend listens for "lsp:message" and routes responses to Monaco

  Frontend (Editor.svelte / new lsp-client.ts)

    Monaco LSP integration via monaco-languageclient (or manual):
    - Register a CompletionItemProvider that sends textDocument/completion
    - Register a HoverProvider that sends textDocument/hover
    - Register a SignatureHelpProvider that sends textDocument/signatureHelp
    - Handle textDocument/publishDiagnostics from the server
      (these replace the cargo-check squiggles when LSP is active)

    Lifecycle:
    - On project load: invoke('start_lsp')
    - On project switch: invoke('stop_lsp') then invoke('start_lsp')
    - On app close: invoke('stop_lsp')
    - textDocument/didOpen sent when a tab is opened
    - textDocument/didChange sent on every edit (full sync mode)
    - textDocument/didClose sent when a tab is closed

  Fallback
    If rust-analyzer is not installed, Feature 1 (cargo check) remains the
    diagnostic source. Show a non-blocking toast: "Install rust-analyzer for
    autocomplete: rustup component add rust-analyzer"

  Capability permission
    None needed — this uses child process spawning (already permitted) and
    app.emit() (already permitted via core:event:default).

---

Feature 3 — Themes (Dark / Light / System)
───────────────────────────────────────────

Problem
  The app is dark-only. Users working in bright environments or preferring
  light themes have no option.

Goal
  Add a theme toggle: System (follows macOS appearance), Dark, Light.
  The theme applies to both the Monaco editor and the surrounding app chrome.

Settings change
  Add to Settings struct and config.json:
    theme: "system" | "dark" | "light"    // default: "system"

  Settings panel gets a new "Theme" segmented control.

Monaco themes
  - playground-dark (existing) — no changes needed
  - playground-light (new) — define in Editor.svelte:
      base: 'vs'
      Background: #ffffff
      Foreground: #1c1c1e
      Colors: matching macOS light appearance (system grays, blue accents)
      Token colors: Xcode-light-inspired (keywords pink, types teal,
        strings red-orange, comments gray, numbers amber)

App chrome (CSS)
  - Define CSS custom properties for both themes:
      --bg-primary, --bg-secondary, --bg-tertiary
      --text-primary, --text-secondary
      --border-color, --accent-color
      --sidebar-bg, --toolbar-bg, --output-bg
  - Apply via a .theme-dark / .theme-light class on <body>
  - "system" listens to prefers-color-scheme media query and sets the class
    accordingly, updating on OS appearance change

Frontend (App.svelte)
  - Pass theme to Editor as a prop
  - Editor $effect watches theme and calls monaco.editor.setTheme()
  - Sidebar, Output, TabBar, toolbar all use CSS vars — no code changes
    needed beyond defining the light-theme values

---

Feature 4 — Export / Share
──────────────────────────

Problem
  Users can't take their playground code outside the app. No way to share
  a working example or move to a real Cargo project.

Goal
  Two export options from a context menu or toolbar action on any playground:

  A) Export as standalone Cargo project
     Creates a self-contained directory with:
       my_playground/
       ├── Cargo.toml          (name = playground name, deps from project)
       ├── src/
       │   └── main.rs         (the playground code)
       └── content/            (if playground has content files)

     Uses a native save dialog to pick the destination folder.
     The exported project compiles and runs with `cargo run` independently.

  B) Copy to clipboard (for sharing)
     Copies the playground source code to the system clipboard.
     One-click from the context menu. No dialog needed.

Backend (lib.rs)

  New command
    #[tauri::command]
    async fn export_playground(
        name: String,
        dest: String,       // destination directory path
        app: AppHandle,
    ) -> Result<String, String>

    - Creates dest/<name>/ directory structure
    - Writes Cargo.toml with only the dependencies used in the playground
      (parse `use` statements to filter, or just copy all project deps —
       simpler and avoids false negatives)
    - Copies src/bin/<name>.rs → src/main.rs
    - Copies content files if they exist
    - Returns the path to the created directory

  Clipboard is handled entirely in the frontend (navigator.clipboard.writeText).

Frontend

  Toolbar: add an export/share icon button (right side, near layout toggle)
  - Click shows a small dropdown:
      "Export as Cargo Project..." → opens save dialog, then invoke('export_playground')
      "Copy Code to Clipboard"    → navigator.clipboard.writeText(currentCode)
  - Context menu on playground in sidebar: same two options
  - Toast confirmation after each action

  Capability permission
    dialog:default (save dialog) — check if already in capabilities, add if not.

---

Feature 5 — Rust Book Examples Polish
──────────────────────────────────────

Problem
  The Rust Book examples (20 chapters) were written in one pass. Some chapters
  may have gaps, unclear comments, or could better demonstrate the concept.

Goal
  Review and polish all 20 chapters in book_chapters.rs:
  - Ensure every playground compiles and runs without errors
  - Improve comments to be clearer and more educational
  - Add missing concepts where a chapter's key idea isn't demonstrated
  - Ensure consistent style across all chapters
  - Verify content files (attribution.md) are present and correct

  This is a quality pass, not a feature — no new backend/frontend code.

Approach
  - Read through each chapter's playgrounds in book_chapters.rs
  - For each: verify it compiles, check the comments are helpful, ensure the
    key Rust Book concept is actually demonstrated
  - Fix any issues found
  - Run cargo fmt + cargo clippy on the output

---

Acceptance Criteria

Feature 1 — Live Error Checking
  [ ] Red squiggles appear ~500 ms after typing stops on a syntax error
  [ ] Squiggle hover shows the compiler error message
  [ ] Squiggles clear when the error is fixed
  [ ] No squiggles on Cargo.toml or content file tabs
  [ ] Running cargo run while a check is in progress works (separate target dir)
  [ ] Starting a Run cancels any in-flight check

Feature 2 — Autocomplete / LSP
  [ ] Completions appear when typing (e.g. after `std::` or `.`)
  [ ] Hover over a symbol shows type info / docs
  [ ] Signature help shows parameter hints inside function calls
  [ ] LSP diagnostics appear as squiggles (replaces cargo-check when active)
  [ ] If rust-analyzer is not installed, falls back to cargo-check gracefully
  [ ] Toast shown suggesting rust-analyzer install if missing
  [ ] LSP process is cleaned up on project switch and app close

Feature 3 — Themes
  [ ] "System" theme follows macOS light/dark appearance automatically
  [ ] "Dark" theme matches current appearance (no regression)
  [ ] "Light" theme has readable contrast and consistent styling
  [ ] Theme change applies immediately to editor AND app chrome
  [ ] Theme preference persists across restarts
  [ ] Theme setting accessible from Settings panel (Cmd+,)

Feature 4 — Export / Share
  [ ] "Export as Cargo Project" creates a working standalone project
  [ ] Exported project compiles with `cargo run` independently
  [ ] Content files are included in the export if they exist
  [ ] "Copy to Clipboard" copies the full source code
  [ ] Both actions show a toast confirmation
  [ ] Export uses a native save dialog for destination

Feature 5 — Rust Book Polish
  [ ] All 20 chapters compile without errors
  [ ] Comments are clear and educational
  [ ] Key concepts per chapter are demonstrated
  [ ] Consistent code style across chapters
  [ ] attribution.md present in each chapter

---

Implementation Order (suggested)

  1. Feature 3  (Themes)           — CSS + Monaco theme, no backend logic, big visual impact
  2. Feature 1  (Live checking)    — backend command + frontend debounce, foundational for F2
  3. Feature 4  (Export/share)     — backend command + dialog, standalone feature
  4. Feature 5  (Book polish)      — review pass, no new architecture
  5. Feature 2  (LSP/autocomplete) — most complex: child process management, LSP protocol,
                                     Monaco providers. Build last since it depends on the
                                     project already being stable.
