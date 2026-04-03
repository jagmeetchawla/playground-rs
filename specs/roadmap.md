ROADMAP
=======

This file tracks in-progress versions, future features, and parked ideas.
Active specs live in specifications.md. Completed specs live in archive/.

---

RELEASED
────────

v0.1.8.1 — Production Testing Bugfix Release
  Status: complete — released 2026-04-03
  Production tested by Jagmeet Chawla (3 rounds: clean+toolchain, clean+no-toolchain, upgrade)

  Shipped:
  1. Fix: toolchain detection uses absolute paths for rustup/rustc/rustfmt/clippy (app bundles get minimal PATH)
  2. Fix: serde_json template dep not added to Cargo.toml (quoted version string bug)
  3. Stop-and-run confirmation dialog — prompt before killing another playground's process
  4. Menu: Copy Code to Clipboard (Cmd+Shift+C) in Playground menu
  5. Menu: Export Project (Cmd+Shift+E) in Project menu
  6. Menu: Rename Playground in Playground menu
  7. Menu sync — Copy Code / Rename / Delete greyed out when no playground tab active
  8. Wizard: added rustup.rs link for install guidance
  9. Capability: shell:allow-open for external links

v0.1.8 — Final Feature Release (pre-distribution)
  Status: complete — released 2026-04-02
  See specs/archive/ for completed spec

  Shipped:
  1. Live error checking — cargo check squiggles in Monaco (300 ms debounce + one-in-flight queue)
  2. Autocomplete / LSP — SKIPPED (decided not to include)
  3. Themes — dark / light / system / rust toggle (Monaco + app chrome)
  4. Export / share — export as standalone CLI playground, copy to clipboard
  5. Rust Book examples polish — all 20 chapters compile with zero warnings
  6. Backend modularization — lib.rs split into 6 focused modules
  7. Test suite — 70 unit tests across 4 backend modules
  8. New app icon — illustrated rustic playground (cargo crate, gear, fn() sign)
  9. Rust theme — warm earthy palette (espresso bg, parchment text, Rust-red accents)
  10. Fix: stdin input now appears as soon as binary starts (detect cargo "Running" line)
  11. Fix: light theme stderr readability — use theme-aware --red variable (#d42020 in light)
  12. Fix: live theme preview in settings — switches instantly on click, reverts on cancel
  13. README: diagonal composite screenshot showing all three themes

v0.1.7 — Settings, Polish, and Deferred v0.1.6 Features
  Status: complete — released 2026-04-02
  See specs/archive/ for completed spec

  Shipped:
  1. Settings panel (Cmd+,) — font size, font family, tab size, cargo path
  2. Toolchain setup wizard — first-run detection (green/yellow/red), re-check
  3. Dependency manager UI — add/remove crates from Cargo.toml toolbar
  4. Playground templates — 11 starter templates with auto-deps
  5. Console improvements — copy button, ANSI color support, timestamps

v0.1.6.3 — Interactive Console (stdin support)
  Status: complete — released 2026-04-02

v0.1.6 — Editor Experience + App Polish
  Status: complete — released 2026-04-01
  See specs/archive/specs-v0.1.6.md

  Shipped (5 of 8):
  1. Stop button — actually kill the process (SIGTERM + SIGKILL fallback)
  5. Window state persistence — size, position, tabs, sidebar width
  6. Resizable panels — drag sidebar and output panel borders
  7. Hide Left Panel button — Cmd+Shift+L, matches Safari/Xcode
  8. Layout switch — toggle output panel bottom <> right

v0.1.5 — Multiple Projects + Unified Storage
  Status: complete — released 2026-04-01
  See specs/archive/specs-v0.1.5.md

v0.1.9 — Rename and Cleanup
  Status: complete — released 2026-04-03
  Renamed from playground-rs to rustic-playground. Cleaned up src/bin.

---

IN PROGRESS
───────────

v0.2 — Multi-Language Support (Native Projects)
  Status: specced — 2026-04-03
  See specs/specifications.md

  Overview:
  Adds a "native" project type alongside existing Rust (Cargo) projects.
  Native projects hold loose source files — C, C++, Zig, Rust (rustc) —
  compiled and run directly. No build system, no dependency management.
  Languages can be mixed freely in a single native project.

  Features:
  1. rustic.toml project manifest
     - Every project gets a rustic.toml at its root
     - [project] — type ("rust" | "native"), created_with version
     - [paths] — src directory, content directory (app reads these, never hardcodes)
     - [toolchain] — informational snapshot of compiler versions at creation time
     - Legacy projects without rustic.toml get one auto-generated on first load

  2. Project type selection
     - New project dialog offers Rust and Native type choices
     - Rust: full Cargo workspace with deps and live checking
     - Native: loose source files, compiler chosen by extension

  3. Native project playground CRUD
     - Source files in project root (paths.src = ".")
     - Files named with extension: hello.c, vectors.cpp, fizzbuzz.zig, ownership.rs
     - Stem validation: same [a-z][a-z0-9_]* rule as Rust playgrounds
     - list/new/load/save/rename/delete/duplicate — all path resolution via rustic.toml

  4. Compile and run (native)
     - .c    → clang <file> -o <out> && <out>
     - .cpp  → clang++ <file> -o <out> -std=c++17 && <out>
     - .zig  → zig run <file>
     - .rs   → rustc <file> -o <out> && <out>
     - Output binaries: target/runs/<stem>
     - Same process group kill, stdin, PLAYGROUND_CONTENT as Rust projects
     - Compiler paths: clang via xcrun, zig via PATH, rustc as cargo sibling

  5. New playground dialog (native)
     - Language picker: C, C++, Zig, Rust
     - Language-appropriate starter template per selection
     - Rust projects: no language picker (always .rs)

  6. Sidebar adaptation
     - Native projects: show filenames with extensions, no Cargo.toml entry, no deps
     - Rust projects: unchanged

  7. Monaco language detection
     - Set editor language by file extension: .c → "c", .cpp → "cpp", .zig → "zig", .rs → "rust"
     - Currently hardcoded to "rust" — change to detect from active tab

  8. Setup wizard updates
     - Detect clang (via xcrun --find clang)
     - Detect zig (via which zig)
     - Show status for all toolchains, link to ziglang.org
     - Does not block on missing Zig or clang

  9. Conditional menus
     - Rust-specific items (Add Dependency, Export as Cargo Project, Rust Book) hidden for native
     - Run/Stop/New/Copy work for both types
     - Menu rebuilds on project switch between types

  Not in scope for v0.2:
  - No dependency management for native projects
  - No live error checking for native projects
  - No LSP / autocomplete for C/C++/Zig
  - No build systems (CMake, Makefiles, zig build)
  - No multi-file compilation (each source file is standalone)
  - No header files for C/C++ (beyond system headers)
  - No templates beyond starter code per language
  - No book chapters for C/C++/Zig

---

NEXT
────

  After v0.2: website, DMG distribution, wiki, announcements.


---

PARKED IDEAS
────────────

These are real, considered proposals that have been deliberately set aside.
Each has a rationale. Revisit when the time is right.

─────────────────────────────────────────────────────────────────────────────
IDEA: Native Swift + Monaco Hybrid (v2.0 rewrite candidate)
─────────────────────────────────────────────────────────────────────────────
Status: Parked — finish current Tauri version first
Logged: 2026-04-01

Background
  The current stack (Tauri + Svelte + WKWebView) has produced recurring classes
  of bugs that are hard to diagnose or fix without deep knowledge of multiple
  layers: DOM event propagation, WKWebView limitations, Tauri v1→v2 API changes,
  Svelte 4 vs 5 idiom drift. The bugs themselves are fixable but the debugging
  path is opaque — you'd need to hold Svelte → WKWebView → Tauri → Rust → macOS
  in your head simultaneously.

  Swift Playgrounds and Xcode both use fully custom native editors (built on
  NSTextView). Those are proprietary and unavailable. The best open-source
  native option is Runestone (Tree-sitter based), but it doesn't match Monaco's
  quality for a code editing tool.

Proposed Architecture
  Native SwiftUI app with one isolated WKWebView for the editor pane only.

  ┌─────────────────────────────────────────────────────────┐
  │  SwiftUI App                                            │
  │                                                         │
  │  ┌──────────────┐  ┌─────────────────┐  ┌───────────┐  │
  │  │ Sidebar      │  │ WKWebView       │  │ Output    │  │
  │  │ (native)     │  │ Monaco editor   │  │ (native)  │  │
  │  │              │  │ (editor only)   │  │           │  │
  │  │ ProjectList  │  │                 │  │ RunBlock  │  │
  │  │ FileList     │  │                 │  │ Console   │  │
  │  └──────────────┘  └─────────────────┘  └───────────┘  │
  │                                                         │
  │  Toolbar: native NSToolbar                              │
  │  Menus:   native NSMenu / commands                      │
  └─────────────────────────────────────────────────────────┘

  - Everything except the editor is native SwiftUI / AppKit
  - The WKWebView hosts only Monaco — no Svelte, no Tauri bridge
  - Swift ↔ Monaco communication via WKScriptMessageHandler (JS → Swift)
    and evaluateJavaScript (Swift → JS) — simple, well-documented, stable
  - Cmd+S, Cmd+R, Cmd+N handled at AppKit level (NSMenuItem / @FocusedValue)
    before they reach the WebView — no keyboard capture fights
  - Drag-drop, file dialogs, context menus all native — no workarounds needed
  - Rust runner stays as a plain Process (Foundation) — same logic, cleaner API
  - Storage: same ~/Library/Application Support layout as current v0.1.x

Why This Is Better
  - Menus, shortcuts, drag-drop, dialogs just work
  - Crashes and errors have clear Swift stack traces
  - Debugging path: Swift → Rust process, nothing else
  - The Monaco WKWebView is isolated — can't interfere with native UI events
  - Code is readable and fixable without knowing Svelte/Tauri internals

Why It's Parked
  - Current Tauri version works and is improving
  - Switching mid-stream loses momentum
  - Swift development is slightly slower (more API lookup needed)
  - Worth doing as a clean v2.0 rewrite once v0.1.x feature set is stable
    and we know exactly what we're rebuilding

Trigger Condition
  Revisit when v0.1.x hits a hard wall — a feature that genuinely can't be done
  cleanly in Tauri — or when the current version feels "done" and a rewrite
  makes sense as a quality investment.

References
  - Runestone: https://github.com/simonbs/Runestone
  - WKScriptMessageHandler docs: Apple Developer Documentation
  - Nova (Panic) uses a similar hybrid native+web-component approach
─────────────────────────────────────────────────────────────────────────────
