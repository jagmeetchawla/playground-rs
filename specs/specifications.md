SPECIFICATION

Status
- Version: v0.3
- Date: 2026-04-03
- Owner: Jagmeet Chawla

---

Product

What
  Rustic Playground v0.3 — Language Module Architecture + Zig & Swift Support.

  Refactor the backend and frontend into per-language modules so each language
  is self-contained and higher-level code dispatches via enum match. Then add
  Zig and Swift as new project types alongside Rust and Clang (C/C++).

  Rust remains the first-class citizen — no compromise on the Rust experience.

Why
  v0.2 added Clang C/C++ with ~61 hardcoded if/else branch points. Adding
  more languages would be unmaintainable. A proper module architecture makes
  each language isolated (can't break another) and adding a new language is
  mechanical — implement the module, add an enum arm, compiler tells you
  every place that needs updating.

  Zig and Swift round out the macOS developer experience — Zig is a rising
  systems language, Swift ships free with Xcode CLI tools.

Design Principle
  Language modules. Each language is a Rust module that implements all
  language-specific behavior. A central Lang enum dispatches to the right
  module. Shared helpers (flat-directory listing, shell export) serve
  file-based languages (clang, zig, swift). Rust is the outlier with its
  Cargo package structure.

---

Architecture — Backend

Lang enum (`src-tauri/src/languages/mod.rs`)
  Enum with exhaustive match dispatch — compiler forces every arm to be
  handled when a new language is added.

  enum Lang { Rust, Clang, Zig, Swift }

  Each variant dispatches to its module's functions:
  - project_type() → &str
  - extensions() → &[&str]
  - src_dir() → &str
  - scaffold_project(path, name) → Result
  - new_manifest() → RusticManifest
  - starter_template(name, ext) → String
  - list_playgrounds(dir) → Vec<String>
  - validate_name(name) → Result<(stem, ext)>
  - playground_path(name, dir) → Result<PathBuf>
  - build_run_command(...) → Result<RunConfig>
  - supports_live_check() → bool
  - detect_toolchain() → Vec<ToolInfo>
  - export_project(...) → Result<PathBuf>

RunConfig enum
  Two run strategies:
  - Direct { program, args, env, cwd } — cargo run, zig run
  - CompileThenRun { compiler, args, binary_path, env, cwd } — clang, swiftc

  Generic runners in playground_commands.rs handle both variants with shared
  output streaming (stream_child_output).

Shared FileLanguage helpers
  Clang, Zig, and Swift share flat-directory pattern. Helpers avoid
  code duplication:
  - file_list_playgrounds(dir, extensions)
  - file_validate_name(name, extensions)
  - file_playground_path(name, dir)
  - file_export_shell(workspace, name, dest, ...)

  Only Rust is different (src/bin/, no extension, Cargo.toml, clap export).

Module structure
  src-tauri/src/languages/
  ├── mod.rs       — Lang enum, RunConfig, ToolInfo, shared helpers
  ├── rust.rs      — Cargo package, cargo run, live check, clap export
  ├── clang.rs     — C/C++ with clang, compile+run, Makefile export
  ├── zig.rs       — zig run (direct), zig flags
  └── swift.rs     — swiftc compile+run, swift flags

---

Architecture — Frontend

Language registry (`ui/src/lib/languages.ts`)
  TypeScript config objects per language. Components read capabilities
  instead of checking project type strings.

  type ProjectType = 'rust' | 'clang' | 'zig' | 'swift'

  interface LanguageConfig {
    type, label, badge, badgeClass, color, extensions,
    hasCargoToml, hasBuildFlags, buildFlagLabels,
    supportsLiveCheck, runCommandDisplay, toolchainName,
    needsExtension, subLanguages?
  }

  Components use: getLang(projectType).hasBuildFlags instead of isNative.

---

Language Specifications

Rust (unchanged from v0.2)
  - Project type: "rust"
  - Extensions: [".rs"]
  - Source dir: src/bin/
  - Run: cargo run --bin <name>
  - Live check: cargo check (300ms debounce)
  - Export: clap CLI runner + merged Cargo.toml
  - Manifest: [build] cflags/cxxflags empty (unused)
  - Toolchain: cargo, rustc via ~/.cargo/bin/

Clang C/C++ (unchanged from v0.2)
  - Project type: "clang"
  - Extensions: [".c", ".cpp"]
  - Source dir: . (project root)
  - Run: clang/clang++ compile → run binary
  - Live check: none
  - Export: POSIX shell runner + Makefile
  - Manifest: [build] cflags, cxxflags
  - Toolchain: clang via xcrun

Zig (new)
  - Project type: "zig"
  - Extensions: [".zig"]
  - Source dir: . (project root)
  - Run: zig run <file>.zig (Direct — single step)
  - Live check: none (future: zig ast-check)
  - Export: POSIX shell runner with zig run commands
  - Manifest: [build] zigflags (e.g. -O ReleaseSafe)
  - Toolchain: zig version
  - Starter: const std = @import("std"); pub fn main() !void { ... }

Swift (new)
  - Project type: "swift"
  - Extensions: [".swift"]
  - Source dir: . (project root)
  - Run: swiftc compile → run binary (CompileThenRun)
  - Live check: none (future: swiftc -typecheck)
  - Export: POSIX shell runner with swiftc compile + run
  - Manifest: [build] swiftflags (e.g. -O)
  - Toolchain: swiftc --version (ships with Xcode CLI tools)
  - Starter: print("Hello from \(name)!")

---

Manifest Evolution (rustic.toml)

  [build] section gains new fields with #[serde(default)]:
    zigflags = []           # zig compiler flags
    swiftflags = []         # swift compiler flags

  [toolchain] section gains new optional fields:
    zig = "0.13.0"          # zig version
    swiftc = "6.0"          # swiftc version

  Backward compatible — existing manifests parse without error.
  Empty arrays/None values are harmless.

  detect_project_type heuristic fallback chain:
    rustic.toml → Cargo.toml → .zig files → .swift files → "clang"

---

Implementation Phases

Phase 1 — Extract Rust module (no behavior change)
  Create languages/ directory with Lang enum, RunConfig, shared helpers.
  Extract Rust logic into languages/rust.rs.
  Checkpoint: all tests pass, app works identically.

Phase 2 — Extract Clang module (no behavior change)
  Extract C/C++ logic into languages/clang.rs.
  Replace all if/else in dispatchers with Lang::from_str() match.
  Extract generic run_direct / run_compile_then_execute.
  Checkpoint: all tests pass, app works identically.

Phase 3 — Zig backend
  Expand manifest structs. Implement languages/zig.rs fully.
  Add Lang::Zig arm to all match blocks.
  Checkpoint: cargo test passes.

Phase 4 — Swift backend
  Expand manifest structs. Implement languages/swift.rs fully.
  Add Lang::Swift arm to all match blocks.
  Checkpoint: cargo test passes.

Phase 5 — Frontend registry + Zig/Swift UI
  Create languages.ts registry. Create zig/swift templates.
  Refactor all components to use registry.
  Add Monaco themes + app themes for Zig and Swift.
  Checkpoint: create/run/export all 4 project types.

---

Scope Boundaries — What v0.3 Does NOT Include

  - No live error checking for Zig/Swift/Clang (future)
  - No LSP / autocomplete for any non-Rust language
  - No package managers (Swift PM, Zig build.zig.zon)
  - No multi-file compilation
  - No book chapters for Zig/Swift (templates are enough)
  - No conversion between project types
