SPECIFICATION

Status
- Version: v0.2
- Date: 2026-04-03
- Owner: Jagmeet Chawla

---

Product

What
  Rustic Playground v0.2 — Native C/C++ projects. Add a **native project** type
  alongside the existing Rust (Cargo) project type. Native projects hold loose
  C and C++ source files compiled directly with clang/clang++. Compiler flags
  are configurable per project via rustic.toml. Default flags include -lsqlite3
  (macOS ships sqlite3) and -std=c++17 for C++.

  Rust projects remain unchanged — full Cargo workspace experience with
  dependencies, live checking, book chapters, and export.

Why
  Panache before public release. C/C++ come free on macOS (clang ships with
  Xcode Command Line Tools). Supporting native languages differentiates Rustic
  Playground from single-language tools. Zig support deferred to v0.3 to keep
  this release tight and well-polished.

Design Principle
  Project-level typing. The project type determines the experience:
  - **Rust project** — Cargo workspace. Full feature set.
  - **Native (C/C++) project** — folder of .c/.cpp files. Compiled with
    clang/clang++. Configurable compiler flags. No build system.

---

Data Model Changes

rustic.toml — project manifest
  Every project has a `rustic.toml` at its root. This is the project's own
  manifest — metadata, structure, build flags, and toolchain info.

  Rust project:
    [project]
    type = "rust"
    created_with = "0.2"

    [paths]
    src = "src/bin"
    content = "content"

    [build]
    cflags = []
    cxxflags = []

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

    [build]
    cflags = ["-lsqlite3"]
    cxxflags = ["-std=c++17", "-lsqlite3"]

    [toolchain]
    clang = "Apple clang 16.0.0"

  The [build] section holds compiler flags passed to clang/clang++ during
  compilation. These are user-editable via the sidebar UI and saved to
  rustic.toml. Standard libraries like stdio.h need no flags. For
  homebrew-installed libraries, the user adds flags like:
    -I/opt/homebrew/include -L/opt/homebrew/lib -lfoo

  The [toolchain] section is informational — populated at creation time.
  The [paths] section tells the app where to find source and content files.

Project type detection
  Primary: read rustic.toml → project.type
  Fallback (legacy): Cargo.toml exists → "rust", otherwise → "native"
  Legacy projects get rustic.toml auto-generated on first load.

Rust project structure (unchanged)
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
  ├���─ <playground>.cpp
  └── content/

  Source files live directly in the project root (paths.src = ".").

Supported extensions and compilers
  .c   → clang <file> -o <out> [cflags] && <out>
  .cpp → clang++ <file> -o <out> [cxxflags] && <out>

  Output binaries: projects/<name>/target/runs/<stem>

Playground naming for native projects
  Names include the extension: `hello.c`, `vectors.cpp`.
  Validation: stem follows `[a-z][a-z0-9_]*` rule. Extension must be .c or .cpp.

---

Feature 1 — Project Type Selection

New project dialog
  - **Rust** — "Cargo workspace with deps and live checking"
  - **Native (C/C++)** — "C/C++ with clang — compiler flags in rustic.toml"

  The choice is permanent for the project.

Backend
  new_project gains optional `project_type` parameter (default: "rust").

  For native: creates rustic.toml (type=native, default build flags), content/,
  and hello.c starter.
  For rust: unchanged (rustic.toml + Cargo.toml + src/bin/hello.rs).

  Commands: get_project_type, get_project_manifest

---

Feature 2 — Native Project: Playground CRUD

  list_playgrounds reads .c/.cpp files from paths.src directory.
  new_playground takes name WITH extension (e.g., "hello.c").
  Starter templates: stdio.h for C, iostream for C++.
  All CRUD operations resolve paths via rustic.toml [paths].

---

Feature 3 — Compile and Run

  1. Read [build] flags from rustic.toml (cflags for .c, cxxflags for .cpp)
  2. Compile: clang/clang++ <source> -o target/runs/<stem> [flags]
  3. If compile fails, stream stderr and complete with error code
  4. If compile succeeds, run the binary with stdin/stdout/stderr streaming

  Compiler path: clang via `xcrun --find clang`, fallback /usr/bin/clang.
  clang++ derived as sibling of clang.

  Process management: same process_group + SIGTERM/SIGKILL pattern.
  PLAYGROUND_CONTENT env var set the same way.

---

Feature 4 — New Playground Dialog (Native)

  Language picker: C, C++ (2 buttons).
  Selected language determines file extension and starter template.
  Default: C.

  Rust projects: no language picker (always .rs, unchanged).

---

Feature 5 — Compiler Flags UI

  Sidebar panel visible for native projects (replaces Cargo.toml section):
  - C flags text input (space-separated)
  - C++ flags text input (space-separated)
  - "Saved to rustic.toml" hint

  Changes save immediately to [build] in rustic.toml via save_build_flags.
  Loaded on project switch via get_build_flags.

  Default flags: -lsqlite3 (C), -std=c++17 -lsqlite3 (C++).

---

Feature 6 — Sidebar Adaptation

  Native projects: show filenames with extensions, compiler flags panel,
  no Cargo.toml entry, no deps.
  Rust projects: unchanged.

---

Feature 7 — Monaco Language Detection

  Editor language by file extension: .c → "c", .cpp → "cpp".
  Rust projects unchanged. Cargo.toml still gets TOML highlighting.

---

Feature 8 — Conditional Menus

  - "Export Project" disabled for native projects
  - "Load Rust Book Examples" disabled for native projects
  - Run/Stop/New/Copy work for both types
  - rebuild_menu receives project_type parameter

---

Feature 9 — Backwards Compatibility

  Legacy projects without rustic.toml get one auto-generated on first load.
  Cargo.toml present → rust. Otherwise → native.
  All existing Rust projects work without manual changes.

---

Scope Boundaries — What v0.2 Does NOT Include

  - No Zig support (deferred to v0.3)
  - No live error checking for native projects (future: clang -fsyntax-only)
  - No LSP / autocomplete for C/C++
  - No build systems (CMake, Makefiles)
  - No multi-file compilation (each source file is standalone)
  - No header files beyond system headers
  - No conversion between project types
  - No book chapters for C/C++

---

Acceptance Criteria

Feature 1 — Project Type Selection
  [ ] New project dialog offers Rust and Native (C/C++) type choices
  [ ] Rust project creates rustic.toml + Cargo.toml + src/bin/
  [ ] Native project creates rustic.toml + content/ + hello.c starter
  [ ] rustic.toml contains [project], [paths], [build], [toolchain] sections
  [ ] get_project_type returns correct type for new and existing projects
  [ ] Legacy projects get rustic.toml auto-generated on first load

Feature 2 — Native Playground CRUD
  [ ] list_playgrounds reads .c/.cpp files from paths.src
  [ ] new_playground creates file with language-appropriate template
  [ ] load/save/rename/delete/duplicate work for native playgrounds
  [ ] All path resolution uses rustic.toml [paths]
  [ ] Stem validation enforced

Feature 3 — Compile and Run
  [ ] .c files compile and run via clang with cflags
  [ ] .cpp files compile and run via clang++ with cxxflags
  [ ] Compilation errors stream to stderr
  [ ] Process kill works (SIGTERM + SIGKILL on process group)
  [ ] stdin input works for native playgrounds
  [ ] PLAYGROUND_CONTENT env var is set

Feature 4 — New Playground Dialog
  [ ] Native projects show language picker (C, C++)
  [ ] Rust projects show no language picker

Feature 5 — Compiler Flags
  [ ] Sidebar shows cflags and cxxflags inputs for native projects
  [ ] Changes saved to rustic.toml [build] section
  [ ] Default flags: -lsqlite3 (C), -std=c++17 -lsqlite3 (C++)
  [ ] Flags loaded on project switch

Feature 6 — Sidebar
  [ ] Native: filenames with extensions, compiler flags panel, no Cargo.toml
  [ ] Rust: unchanged

Feature 7 — Monaco
  [ ] .c files get C syntax highlighting
  [ ] .cpp files get C++ syntax highlighting

Feature 8 — Menus
  [ ] Export and Rust Book disabled for native projects
  [ ] Run/Stop/New/Copy work for both types
  [ ] Menu rebuilds on project switch

Feature 9 — Backwards Compatibility
  [ ] Legacy projects get rustic.toml auto-generated
  [ ] All existing Rust projects work without changes
