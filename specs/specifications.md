SPECIFICATION

Status
- Version: v0.2
- Date: 2026-04-03
- Owner: Jagmeet Chawla

---

Product

What
  Rustic Playground v0.2 — multi-language support. Add a **native project** type
  alongside the existing Rust (Cargo) project type. Native projects hold loose
  source files — `.c`, `.cpp`, `.zig`, `.rs` — compiled and run directly via
  `clang`, `clang++`, `zig run`, and `rustc`. No build system, no dependency
  management. Languages can be mixed freely in a single native project.

  Rust projects remain unchanged — full Cargo workspace experience with
  dependencies, live checking, book chapters, and export.

Why
  Panache before public release. C/C++ come free on macOS (clang ships with
  Xcode Command Line Tools). Zig is a rising language with a dead-simple
  `zig run` model. Supporting multiple languages differentiates Rustic Playground
  from single-language tools and makes it useful for broader experimentation.

Design Principle
  Project-level typing. The project type determines the experience:
  - **Rust project** — Cargo workspace. Full feature set.
  - **Native project** — folder of source files. Compiler chosen by extension.
    No build system. No deps. Mix languages freely.

---

Data Model Changes

rustic.toml — project manifest
  Every project has a `rustic.toml` at its root. This is the project's own
  manifest — metadata, structure, and toolchain info.

  Rust project:
    [project]
    type = "rust"
    created_with = "0.2"

    [paths]
    src = "src/bin"
    content = "content"

    [toolchain]
    rustc = "1.82.0"
    cargo = "1.82.0"

  Native project:
    [project]
    type = "native"
    created_with = "0.2"

    [paths]
    src = "."
    content = "content"

    [toolchain]
    clang = "Apple clang 16.0.0"
    zig = "0.14.0"

  The [toolchain] section is informational — populated at creation time with
  detected versions. Not used for dispatch.

  The [paths] section tells the app where to find source files and content.
  The app reads these paths instead of hardcoding src/bin/ or project root.
  In a future release, users could point content elsewhere or organize source
  files into subfolders.

Project type detection
  Primary: read rustic.toml → project.type
  Fallback (legacy projects without rustic.toml): if Cargo.toml exists → "rust",
  otherwise infer from file extensions present.
  The app writes rustic.toml on project creation. Legacy projects get one
  auto-generated on first load.

Rust project structure
  projects/<name>/
  ├── rustic.toml
  ├── Cargo.toml
  ├── src/bin/
  │   └── <playground>.rs
  └── content/

Native project structure
  projects/<name>/
  ├── rustic.toml
  ├── <playground>.c
  ├── <playground>.cpp
  ├── <playground>.zig
  ├── <playground>.rs
  └── content/

  No src/bin/ nesting. No Cargo.toml. Source files live directly in the
  project root (as declared by paths.src = "."). The content/ folder path
  comes from paths.content.

Supported extensions and compilers
  .c    → clang <file> -o <out> && <out>
  .cpp  → clang++ <file> -o <out> -std=c++17 && <out>
  .zig  → zig run <file>
  .rs   → rustc <file> -o <out> && <out>

  Output binaries go to: projects/<name>/target/runs/<stem>
  (target/ is already gitignored and skipped by duplicate_project)

Playground naming for native projects
  Names include the extension: `hello.c`, `vectors.cpp`, `fizzbuzz.zig`.
  Validation: `<stem>` follows the same `[a-z][a-z0-9_]*` rule. Extension must
  be one of the supported set.

---

Feature 1 — Project Type Selection

New project dialog
  When creating a new project, the user picks a type:
  - **Rust** — "Full Cargo workspace with dependencies and live checking"
  - **Native** — "Loose source files — C, C++, Zig, Rust (rustc). No build system."

  The choice is permanent for the project (no conversion between types).

Backend
  new_project gains an optional `project_type` parameter (default: "rust").

  For native projects:
  - Creates rustic.toml with type = "native", paths.src = ".", paths.content = "content"
  - Populates [toolchain] with detected clang/zig versions
  - Creates content/ directory
  - Seeds a hello.c starter file:
      #include <stdio.h>
      int main() {
          printf("Hello from hello!\n");
          return 0;
      }
  - Does NOT create Cargo.toml or src/bin/

  For rust projects:
  - Creates rustic.toml with type = "rust", paths.src = "src/bin", paths.content = "content"
  - Populates [toolchain] with detected rustc/cargo versions
  - Existing behavior (Cargo.toml + src/bin/hello.rs + content/)

  New command: get_project_type(name) → "rust" | "native"
  - Reads rustic.toml → project.type
  - Fallback: Cargo.toml present → "rust", otherwise infer from files

  New command: get_project_manifest(name) → RusticManifest struct
  - Returns the full parsed rustic.toml (type, paths, toolchain)
  - The frontend uses paths.src and paths.content for all file operations

  Legacy project migration:
  - On first load of a project without rustic.toml, auto-generate one
    based on detected structure (Cargo.toml → rust, otherwise native)

Frontend
  NewProjectModal (or inline in sidebar) shows the type picker. Once selected,
  the project is created with the appropriate type.

---

Feature 2 — Native Project: Playground CRUD

list_playgrounds (native)
  Reads project root directory. Returns files matching supported extensions
  (.c, .cpp, .zig, .rs). Sorted alphabetically.

  The existing list_playgrounds reads from src/bin/ and strips .rs — the native
  variant reads from the project root and preserves the full filename (stem +
  extension), since extension matters.

new_playground (native)
  Takes a name WITH extension (e.g., "hello.c"). Validates stem, validates
  extension is supported. Writes a language-appropriate starter template:

  .c:
    #include <stdio.h>
    int main() {
        printf("Hello from <name>!\n");
        return 0;
    }

  .cpp:
    #include <iostream>
    int main() {
        std::cout << "Hello from <name>!" << std::endl;
        return 0;
    }

  .zig:
    const std = @import("std");
    pub fn main() !void {
        const stdout = std.io.getStdWriter();
        try stdout.print("Hello from <name>!\n", .{});
    }

  .rs:
    fn main() {
        println!("Hello from <name>!");
    }

load_playground / save_playground (native)
  Same as Rust, but path is project_root/<name_with_ext> instead of
  src/bin/<name>.rs.

rename_playground / delete_playground / duplicate_playground (native)
  Same semantics, different base path. Rename must preserve or allow changing
  the extension.

Implementation approach
  All playground commands resolve paths through rustic.toml's [paths] section:

  - Read paths.src from the manifest to find the source directory
  - Rust: workspace/<paths.src>/<name>.rs  (e.g., workspace/src/bin/hello.rs)
  - Native: workspace/<paths.src>/<name>   (e.g., workspace/hello.c)
  - Content: workspace/<paths.content>/    (e.g., workspace/content/)

  This means the backend never hardcodes src/bin/ or assumes project root.
  The path comes from the manifest. Existing safe_playground_path is updated
  to read the manifest and resolve accordingly.

---

Feature 3 — Native Project: Compile and Run

run_playground (native)
  Detects language from file extension. Builds and runs:

  For compiled languages (C, C++, Rust):
    1. Compile: clang/clang++/rustc <source> -o <target_dir>/<stem>
    2. If compilation fails, stream stderr and send { stream: "complete", code: 1 }
    3. If compilation succeeds, run the binary and stream stdout/stderr
    4. Send { stream: "complete", code: <exit_code> }

  For Zig:
    1. Run: zig run <source>
    2. Stream stdout/stderr
    3. Send { stream: "complete", code: <exit_code> }

  Compiler output (errors, warnings) goes to stderr stream — the frontend
  already displays stderr in red, so compilation errors show naturally.

  The output channel sends the same JSON shape as today:
    { "stream": "stdout" | "stderr" | "complete", "line": "...", "code": N }

  Process management:
  - Same process_group(0) + SIGTERM/SIGKILL pattern
  - Same RunningProcess state for kill_playground
  - Same StdinHandle for send_stdin (interactive input works)
  - PLAYGROUND_CONTENT env var set the same way

  Compiler paths:
  - clang / clang++: use `xcrun --find clang` to resolve (works even without
    full Xcode install, just Command Line Tools). Fall back to /usr/bin/clang.
  - zig: check ~/.local/bin/zig, then PATH via `which zig`
  - rustc: sibling of cargo — derive from settings cargo_path
    (e.g., ~/.cargo/bin/cargo → ~/.cargo/bin/rustc)

kill_playground
  Unchanged — works for any child process.

check_playground (native)
  Not supported for native projects. The frontend should not trigger live
  checking for native project tabs. (Future: clang -fsyntax-only, zig check,
  rustc --edition 2021 -- but not in v0.2.)

---

Feature 4 — New Playground Dialog (Native)

When creating a playground in a native project, the dialog must include a
language picker:

  Name: [____________]
  Language: [C] [C++] [Zig] [Rust]

  The language selection determines the file extension and starter template.
  Default language: C (since it's the most accessible).

For Rust projects, the dialog remains unchanged (no language picker — always .rs).

---

Feature 5 — Sidebar Adaptation

Rust projects (unchanged)
  Sidebar shows:
  - Cargo.toml (always at top)
  - Playgrounds (src/bin/*.rs, displayed without .rs extension)
  - Content files

Native projects
  Sidebar shows:
  - Playgrounds grouped or sorted by extension, displayed WITH extension:
      hello.c
      vectors.cpp
      fizzbuzz.zig
      ownership.rs
  - Content files

  No Cargo.toml entry. No "Dependencies" section.

  Context menu on playground: Run, Rename, Duplicate, Delete (same as Rust
  minus dependency-related items).

---

Feature 6 — Monaco Editor Language Detection

Set Monaco language mode based on file extension:
  .c    → "c"
  .cpp  → "cpp"
  .zig  → "zig"          (Monaco has basic Zig support via monarch tokenizer)
  .rs   → "rust"

Currently hardcoded to "rust" — change to detect from the active tab's
filename/extension.

For Rust projects, all playground tabs are still "rust". Cargo.toml is "toml".
For native projects, each tab gets its language from its extension.

---

Feature 7 — Setup Wizard Updates

The setup wizard currently checks for Rust toolchain only. For v0.2:

Toolchain detection
  - Rust: cargo, rustc, rustfmt, clippy (existing)
  - C/C++: clang (check via `xcrun --find clang` or `which clang`)
  - Zig: zig (check via `which zig`)

Display in wizard
  Rust tools:    ✅ cargo, rustc, rustfmt, clippy
  C/C++ tools:   ✅ clang (via Xcode Command Line Tools)
  Zig:           ❌ not found — install from ziglang.org

  Rust is required for Rust projects. C/C++ and Zig are optional — the user
  can still create native projects; they'll just get an error if they try to
  run a language whose compiler isn't installed.

  The wizard does NOT block on missing Zig. It shows the status and a link
  to ziglang.org for installation.

---

Feature 8 — Menu and Keyboard Shortcuts

Existing Rust-specific menu items should be aware of project type:

  - "Add Dependency..." — only shown for Rust projects
  - "Export as Cargo Project..." — only shown for Rust projects
  - "Load Rust Book Examples..." — only shown for Rust projects
  - Run (⌘R), Stop (⌘.), New Playground (⌘N) — work for both types
  - Copy Code (⌘⇧C) — works for both types

rebuild_menu receives the project type and conditionally includes/excludes
Rust-specific items.

---

Feature 9 — Backwards Compatibility

Existing projects
  Legacy projects have no rustic.toml. On first load, the app auto-generates
  one by detecting structure:
  - Cargo.toml present → type = "rust", paths.src = "src/bin", paths.content = "content"
  - Otherwise → type = "native", paths.src = ".", paths.content = "content"
  Toolchain versions populated from currently installed tools.

Config
  No changes to config.json structure. The active_project field works the
  same way regardless of project type.

Export
  Export is Rust-only (produces a Cargo project). For native projects,
  "Copy Code to Clipboard" is available; full export is not applicable
  (single files don't need a project wrapper).

---

Scope Boundaries — What v0.2 Does NOT Include

  - No dependency management for native projects
  - No live error checking for native projects (future: clang -fsyntax-only)
  - No LSP / autocomplete for C/C++/Zig
  - No build systems (CMake, Makefiles, zig build)
  - No multi-file compilation (each source file is standalone)
  - No header files for C/C++ (beyond system headers)
  - No Zig auto-installer in the wizard (just detection + link)
  - No conversion between project types
  - No templates for native projects (just starter code per language)
  - No book chapters for C/C++/Zig

---

Acceptance Criteria

Feature 1 — Project Type Selection
  [ ] New project dialog offers Rust and Native type choices
  [ ] Rust project creates rustic.toml + Cargo.toml + src/bin/ structure
  [ ] Native project creates rustic.toml + content/ + hello.c starter
  [ ] rustic.toml contains correct [project], [paths], and [toolchain] sections
  [ ] get_project_type returns correct type for new and existing projects
  [ ] Legacy projects without rustic.toml get one auto-generated on first load

Feature 2 — Native Playground CRUD
  [ ] list_playgrounds reads from paths.src in rustic.toml
  [ ] list_playgrounds returns .c, .cpp, .zig, .rs files for native projects
  [ ] new_playground creates file with language-appropriate template
  [ ] load/save/rename/delete/duplicate work for native playgrounds
  [ ] All path resolution uses rustic.toml [paths], never hardcoded
  [ ] Stem validation enforced (same naming rules as Rust playgrounds)

Feature 3 — Compile and Run
  [ ] .c files compile and run via clang
  [ ] .cpp files compile and run via clang++ with -std=c++17
  [ ] .zig files run via zig run
  [ ] .rs files compile and run via rustc
  [ ] Compilation errors stream to stderr (shown in red in output panel)
  [ ] Process kill works (SIGTERM + SIGKILL on process group)
  [ ] stdin input works for native playgrounds
  [ ] PLAYGROUND_CONTENT env var is set

Feature 4 — New Playground Dialog
  [ ] Native projects show a language picker (C, C++, Zig, Rust)
  [ ] Rust projects show no language picker (always .rs)
  [ ] Selected language determines file extension and starter template

Feature 5 — Sidebar
  [ ] Native projects show files with extensions (hello.c, not hello)
  [ ] No Cargo.toml entry for native projects
  [ ] Context menus work correctly for native playgrounds

Feature 6 — Monaco Language Detection
  [ ] .c files get C syntax highlighting
  [ ] .cpp files get C++ syntax highlighting
  [ ] .zig files get Zig syntax highlighting
  [ ] .rs files get Rust syntax highlighting (both project types)
  [ ] Cargo.toml still gets TOML highlighting

Feature 7 — Setup Wizard
  [ ] Shows clang status (found / not found)
  [ ] Shows zig status (found / not found)
  [ ] Links to ziglang.org for Zig installation
  [ ] Does not block on missing Zig or clang

Feature 8 — Menus
  [ ] Rust-specific menu items hidden for native projects
  [ ] Run/Stop/New/Copy work for both project types
  [ ] Menu rebuilds correctly on project switch between types

Feature 9 — Backwards Compatibility
  [ ] Legacy projects without rustic.toml get one auto-generated on first load
  [ ] Auto-detection correctly identifies Rust projects (Cargo.toml present)
  [ ] All existing Rust projects work without manual changes

---

Implementation Order (suggested)

  1. Data model     — rustic.toml, get_project_type, new_project with type
  2. Playground CRUD — native path helpers, list/new/load/save/rename/delete
  3. Compile & run  — native run_playground dispatch, compiler resolution
  4. Frontend       — sidebar, new playground dialog, language picker
  5. Monaco         — language detection by extension
  6. Wizard         — clang/zig detection
  7. Menus          — conditional items based on project type
  8. Polish         — test all language combinations, edge cases
