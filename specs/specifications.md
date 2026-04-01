SPECIFICATION

Status
- Version: v0.1.5
- Date: 2026-03-31
- Owner: Jagmeet Chawla

---

Visual References

Swift Playgrounds reference (UI target)
  specs/assets/swift-playgrounds-reference.png

Annotated feedback screenshots
  specs/assets/v1.2-annotated-feedback.png
  specs/assets/v1.3-annotated-feedback.png

---

Product

What
A native macOS desktop app — built with Tauri — that wraps the existing Rust playground
runner in a Swift Playgrounds-inspired UI. Multiple independent Projects let users
maintain separate Rust environments with different dependencies. Write Rust, see errors
live, hit Run, see output.

Why
Single-workspace was a good starting point but limits real use. Different problems need
different dependency sets — async code needs tokio, data work needs polars, web needs
axum. Projects make the tool genuinely useful across domains without dependency conflict.

---

Core Concept Change: Projects (v0.1.5)

A Project is an independent Rust package with:
  - Its own Cargo.toml  (and thus its own dependency set)
  - Its own set of Playgrounds  (src/bin/*.rs)
  - Its own Content folder  (content/)

"Workspace" is not used as a term — it collides with Cargo workspaces (multi-crate
monorepos). "Project" maps to how Xcode, VS Code, and IntelliJ name this concept.

Hierarchy:
  Projects  >  Playgrounds  >  Content files

---

Storage — Unified (dev = release)

ALL storage uses macOS Application Support regardless of build mode.
The dev-mode shortcut (repo root as workspace) is removed. Dev and release
behave identically.

  ~/Library/Application Support/com.playground-rs.app/
    config.json              ← { "active_project": "default" }
    projects/
      default/
        Cargo.toml
        src/bin/
          hello.rs
          my_playground.rs
        content/
          data.csv
      async_experiments/
        Cargo.toml           ← [dependencies] tokio = { ... }
        src/bin/
          server.rs
        content/
      data_science/
        Cargo.toml           ← [dependencies] polars = { ... }
        src/bin/
          analysis.rs
        content/

Rules:
  - Each project's content/ is isolated to that project.
  - Switching projects closes all open tabs and loads the new project.
  - The active project name is persisted to config.json so it survives restarts.
  - On first launch (no projects exist), a "default" project is created and seeded
    with a hello.rs playground.

---

UI Layout (v0.1.5)

┌────────────────────────────────────────────────────────────────────────────────────┐
│  RS  [ default ▾ ]          │  ⊙ cargo 1.x.x  │         [💾 Save]  [▶ Run]        │
├─────────────────────┬───────────────────────────────┬──────────────────────────────┤
│  Playgrounds│Content │  [tab] hello.rs  [tab] …  ×  │  Console               Clear │
│  ───────────┴─────── │  ──────────────────────────── │                              │
│  🔍 Filter           │   fn main() {                 │  ▸ Run #1  cargo run  15:32 ✓ │
│  RS hello.rs  ●      │     println!("Hello!");       │  ▾ Run #2  cargo run  15:34 ✓ │
│  RS server.rs        │   }                           │    COMPILER                  │
│  ...                 │                               │      Compiling…              │
│  ─────────────────── │                               │    OUTPUT                    │
│  > Cargo.toml        │                               │      Hello!                  │
└─────────────────────┴───────────────────────────────┴──────────────────────────────┘

---

Project Switcher (toolbar)

The app name "Rust Playground" is replaced by a project name pill with a dropdown arrow:

  [ default ▾ ]

Clicking it opens a popover anchored to the pill:

  ┌──────────────────────┐
  │ ● default            │  ← active project (checkmark or filled dot)
  │   async_experiments  │
  │   data_science       │
  │ ────────────────────  │
  │   New Project…       │
  │ ────────────────────  │
  │   Rename Project…    │
  │   Delete Project…    │
  └──────────────────────┘

Interactions:
  Click a project name  → switch to that project (close tabs, reload sidebar)
  New Project…          → opens inline name input in the popover
  Rename Project…       → opens inline rename input for the active project
  Delete Project…       → confirmation alert, then delete; switch to first remaining
                          project or create "default" if none left

Project name rules (same as playground names):
  - Lowercase letters, digits, underscores only: [a-z][a-z0-9_]*
  - Max 64 characters
  - Must be unique

---

Sidebar — unchanged from v1.4

The two-tab sidebar (Playgrounds | Content) is unchanged.
Content tab now shows files for the active project's content/ folder.
No per-playground subfolder — content/ is shared across all playgrounds
within the same project.

---

Backend — New Commands

list_projects() → Result<Vec<String>>
  Lists project directory names under projects/, alphabetically sorted.

get_active_project() → Result<String>
  Returns the active project name from config.json.
  Returns "default" if config.json missing or malformed.

new_project(name: String) → Result<()>
  Creates projects/<name>/ with:
    - Cargo.toml  (fresh package with just the package section + empty [dependencies])
    - src/bin/hello.rs  (hello world template)
    - content/  directory
  Errors if name already exists.

switch_project(name: String) → Result<()>
  Updates the in-memory active project state.
  Persists to config.json.
  Frontend is responsible for closing tabs and reloading state after this call.

rename_project(old_name: String, new_name: String) → Result<()>
  Renames the directory.
  If old_name is the active project, updates config.json to new_name.

delete_project(name: String) → Result<()>
  Deletes projects/<name>/ recursively.
  Does not switch automatically — frontend switches first, then calls delete.

duplicate_project(name: String) → Result<String>
  Copies projects/<name>/ to projects/<name>_copy (or _copy2, etc.).
  Returns the new project name.

---

Backend — Modified Behaviour

workspace_dir(app) is now project-aware:
  projects_dir(app).join(active_project_name)

projects_dir(app):
  app_data_dir().join("projects")   ← same path in dev and release

Active project is held in Tauri app state:
  struct ActiveProject(Mutex<String>)

Registered in setup():
  app.manage(ActiveProject(Mutex::new(loaded_project_name)))

All existing commands (list_playgrounds, new_playground, run_playground,
list_content_files, etc.) remain unchanged in signature — they just resolve
paths through the updated workspace_dir() which is now project-aware.

config.json schema:
  { "active_project": "default" }

ensure_workspace() is renamed ensure_project() and called:
  - On app startup (for the active project)
  - After new_project() (for the new project)

Migration note:
  Dev-mode users who had playgrounds in src/bin/ (repo root) will not see them
  after upgrading to v0.1.5 — those files are in a different location. Users
  should manually copy src/bin/*.rs files into the new project's folder if needed.
  v0.1.5 creates a fresh "default" project on first launch.

---

Frontend — Changes

App.svelte:
  - Add activeProject: string state, loaded from get_active_project() in onMount
  - Add projects: string[] list, loaded from list_projects() in onMount
  - Replace toolbar app name with <ProjectSwitcher> component
  - switchProject(name): call switch_project(name), clear all tabs, reload
    playgrounds/cargoToml/toolchainInfo for the new project

New component: ProjectSwitcher.svelte
  Props: projects: string[], active: string
  Events: switch(name), new(name), rename({old, new}), delete(name)
  Renders the pill button + dropdown popover.
  Inline name input for new/rename within the popover.

No changes to:
  - Sidebar.svelte  (already project-agnostic after v1.4 simplification)
  - Editor.svelte
  - TabBar.svelte
  - Output.svelte

---

Native macOS Menu Bar (v0.1.5 revision)

The menu bar must reflect the two-level hierarchy: Projects and Playgrounds.
The old single "File" menu is replaced by two menus: "Project" and "Playground".

Full menu structure:

  Rust Playground          ← app menu (unchanged)
    About Rust Playground
    ─────
    Hide Rust Playground   ⌘H
    Hide Others            ⌥⌘H
    Show All
    ─────
    Quit                   ⌘Q

  Project                  ← new menu (was part of File)
    New Project…           ⌘⇧N
    ─────
    <project list>         ← dynamic; active project has a checkmark (✓)
    ─────
    Rename Project…
    Delete Project…

  Playground               ← renamed from File, playground-scoped actions
    New Playground         ⌘N
    ─────
    Save                   ⌘S
    ─────
    Close Tab              ⌘W

  Run                      ← unchanged
    Run                    ⌘R
    Stop                   ⌘.

  Edit                     ← unchanged
    Undo  Redo  Cut  Copy  Paste  Select All

Dynamic project list:
  - Rebuilt whenever a project is created, renamed, or deleted.
  - Each item has id "switch_project:<name>" so on_menu_event can parse it.
  - Active project gets a checkmark via MenuItem::set_checked(true) or
    by rebuilding the menu with the checked item.
  - Clicking any project name emits "menu:switch-project" with the name
    as payload to the frontend.

Keyboard accelerators:
  ⌘⇧N  → "menu:new-project"     (open new-project flow in ProjectSwitcher)
  ⌘N   → "menu:new"             (new playground in active project)
  ⌘S   → "menu:save"
  ⌘R   → "menu:run"
  ⌘.   → "menu:stop"
  ⌘W   → "menu:close-tab"

New Tauri events emitted by on_menu_event:
  "menu:new-project"     → frontend opens ProjectSwitcher in 'new' mode
  "menu:rename-project"  → frontend opens ProjectSwitcher in 'rename' mode
  "menu:delete-project"  → frontend opens ProjectSwitcher in 'delete-confirm' mode
  "menu:switch-project"  → payload: project name string; frontend calls switchProject()
  (existing events unchanged: menu:new, menu:save, menu:run, menu:stop, menu:close-tab)

Menu rebuild command:
  rebuild_projects_menu(projects: Vec<String>, active: String, app: AppHandle)
  Called from frontend after any project list change (new/rename/delete/switch).
  Rebuilds only the Project submenu, leaves other menus untouched.

Frontend additions:
  - App.svelte listens to menu:new-project, menu:rename-project,
    menu:delete-project — sets ProjectSwitcher mode accordingly via a
    new openSwitcher(mode) binding
  - App.svelte listens to menu:switch-project — calls switchProject(payload)
  - After every project mutation (new/rename/delete/switch), calls
    invoke('rebuild_projects_menu', { projects, active: activeProject })
  - ProjectSwitcher: expose openWithMode(mode) so App can drive it from
    menu events (open popover + set mode in one call)

---

Acceptance Criteria (v0.1.5)

STORAGE
[ ] App Support used in both dev and release mode
[ ] projects/ directory created on first launch
[ ] config.json created/updated on project switch
[ ] Active project survives app restart

PROJECT SWITCHER
[ ] Toolbar shows active project name with ▾ arrow
[ ] Clicking opens popover with project list
[ ] Active project is visually indicated (dot or checkmark)
[ ] Clicking another project switches to it
[ ] All open tabs are closed on switch
[ ] Sidebar reloads with new project's playgrounds and content

NEW PROJECT
[ ] "New Project…" opens inline name input
[ ] Validation: lowercase, digits, underscores; max 64 chars
[ ] Duplicate name shows error
[ ] New project seeded with hello.rs playground
[ ] Switches to new project after creation

RENAME PROJECT
[ ] "Rename Project…" pre-fills current name
[ ] Validation rules same as new
[ ] Toolbar pill updates immediately
[ ] config.json updated if renaming active project

DELETE PROJECT
[ ] Confirmation shown before delete
[ ] Project directory removed
[ ] Switches to first remaining project (or creates "default" if none)
[ ] Deleted project no longer appears in list

NATIVE MENU BAR
[ ] "Project" menu present (replaces old File menu project actions)
[ ] "Playground" menu present with New / Save / Close Tab
[ ] Project list in Project menu is dynamic — updates on any project change
[ ] Active project shown with checkmark ✓ in project list
[ ] Clicking a project name in the menu switches to that project
[ ] ⌘⇧N triggers new-project flow in ProjectSwitcher popover
[ ] Rename Project… and Delete Project… in menu open correct popover modes
[ ] All existing accelerators still work (⌘N, ⌘S, ⌘R, ⌘., ⌘W)

EXISTING FUNCTIONALITY
[ ] All v1.4 features work within a project (playgrounds, content, run, save)
[ ] Each project's content/ is independent
[ ] PLAYGROUND_CONTENT points to the active project's content/ folder

---

Exclusions (v0.1.5)
- No project import/export
- No project templates beyond the default hello world seed
- No drag to reorder projects in the list
- No project-level metadata (description, icon, colour)
- No multi-window (one project open at a time)
