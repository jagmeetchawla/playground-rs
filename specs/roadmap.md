ROADMAP
=======

This file tracks in-progress versions, future features, and parked ideas.
Active specs live in specifications.md. Completed specs live in archive/.

---

RELEASED
────────

v0.3 — Language Module Architecture + Zig & Swift Support
  Status: complete — released 2026-04-03

  Shipped:
  1. Lang enum with exhaustive match dispatch (languages/mod.rs)
  2. Per-language modules: rust.rs, clang.rs, zig.rs, swift.rs
  3. RunConfig enum (Direct vs CompileThenRun) with generic runners
  4. Shared FileLanguage helpers for flat-directory languages
  5. Zig project type: zig run, custom Monaco tokenizer, 6 templates
  6. Swift project type: swiftc compile+run with SDK resolution, 6 templates
  7. Frontend language registry (languages.ts) replacing if/else branching
  8. Zig marked experimental with badge in ProjectSwitcher
  9. Manifest expansion: zigflags, swiftflags, zig/swiftc toolchain info
  10. Toolchain wizard: Zig + Swift tabs with install instructions
  11. Settings: Zig + Swift toolchain version display
  12. Book system modularization: generic seed_book dispatch via Lang enum
  13. Apple's Swift Programming Language book examples (8 chapters, 14 playgrounds)
  14. Dynamic Help menu: book items generated from language modules

  After v0.3: website (rustic-playground.app), DMG distribution, wiki, announcements.
  Website/FAQ/wiki content source: HelpModal.svelte (Apple-style user guide).
  When building the website: extract HelpModal content into shared markdown files that
  both the Svelte app and the static site generator consume. Until then, HelpModal.svelte
  is the single source of truth — edit inline.
  Must document: Zig support targets 0.15.x — breaking API changes between versions.

---

IN PROGRESS
───────────

v0.3.2 — Welcome Wizard + Language Gating
  Status: in progress — 2026-04-04

  Overview:
  Multi-step Welcome Wizard replaces the single-screen toolchain check. Users
  choose which languages to enable, verify toolchains, set theme/font, and
  optionally load book examples — all in a guided 5-step flow. Only enabled
  languages appear in project switcher, Learn menu, books section, and Settings.
  Languages can be added/removed later from Settings. At least one language must
  be selected.

  Completed:
  1. Config: enabled_languages field with serde default (backward compatible)
  2. Backend commands: get_enabled_languages, set_enabled_languages
  3. complete_wizard: accepts and persists enabled_languages
  4. Menu: Learn menu filters by enabled languages
  5. languages.ts: enabledLanguageConfigs helper
  6. Welcome Wizard: 5-step (Languages, Toolchains, Appearance, Books, Finish)
  7. App.svelte: enabledLangs state, wizard/settings handlers, menu rebuild
  8. ProjectSwitcher: project types and books filtered by enabled languages
  9. Dual-mode ToolchainWizard: wizard (first launch) + settings panel (⌘,)
  10. Empty-state book buttons filtered by enabled languages
  11. Apply button in settings mode (persist without closing)
  12. Book management via explicit checkboxes (load/remove, not auto)
  13. Toolchain pill: language support status (red=not enabled, yellow=partial, green=ok)
  14. Per-language hello projects on wizard completion (hello_rust, hello_c, etc.)
  15. Rename native→clang throughout codebase (backend, frontend, docs)
  16. CLG badge for project-level, C/C++ badges for file-level
  17. Apple HIG styling: ghost/tinted toolbar buttons, 600-weight titles, standardized font sizes
  18. Toast stacking with independent auto-dismiss timers
  19. Books section in dropdown only shows loaded books (no "Load" option)
  20. Project selector pill matches popover width (240–320px)
  21. Stdout streaming fix: byte-level pipe reading for prompts without newline
  22. Synthetic "Running" marker for non-cargo direct runners (zig run)
  23. CLI Input templates for Zig (0.15 API) and Swift
  24. Zig version pinning: version_ok check for 0.15.x, yellow warning for other versions
  25. Zig label includes version: "Zig (v0.15)" throughout UI
  26. HelpModal rewrite: Apple User Guide style with sidebar navigation, 9 sections
      (source of truth for website/wiki/FAQ content)

---

PLANNED
───────

v0.3.3 — Edition Builds (Rust Edition, C Edition, Power Edition)
  Status: in progress — 2026-04-04

  Overview:
  Ship multiple editions of the app as separate DMGs, each tailored to a single
  language or the full multi-language experience. Same codebase, different configs.
  Tauri --config overrides handle app name/identifier/icon; VITE_EDITION env var
  controls frontend behavior via editions.ts registry. Single-language editions
  feel native — no language pickers, no irrelevant badges/themes/books.

  Completed:
  1. Edition config registry (ui/src/lib/editions.ts) — EditionConfig type,
     5 editions (power, rust, clang, zig, swift), currentEdition() helper
  2. Tauri config overrides (editions/*.json) — per-edition productName,
     identifier, window title for all 5 editions
  3. App.svelte: edition language enforcement on mount (locks enabledLangs)
  4. ToolchainWizard: edition-aware wizard steps (skip Languages for single-lang),
     edition-aware settings tabs, filtered theme dropdown, edition display name
  5. ProjectSwitcher: hide language badges and type selector for single-lang editions
  6. HelpModal: dynamic sections (skip Languages for single-lang), filtered
     language cards and book items by enabledLanguages, edition display name
  7. AboutModal: edition display name and tagline
  8. menu.rs: dynamic product name from tauri.conf (merged config)
  9. Build script (scripts/build-editions.sh) — multi-edition build pipeline

  Remaining:
  10. Per-edition icons (art task)
  11. CI matrix for automated multi-edition builds
  12. End-to-end testing of each edition

v0.3.4 — Linux Port (GTK4 / Vala)
  Status: planned

  Overview:
  Native Linux desktop app using GTK4 and Vala, packaged as .deb and .rpm.
  Not a Tauri port — a purpose-built native Linux app that shares the same
  project storage format and playground model but uses GTK4 for the UI layer.

  Why GTK4/Vala instead of porting Tauri:
  - Tauri on Linux uses WebKitGTK which has quality/performance issues
  - GTK4 is the native Linux toolkit — proper theming, keyboard handling,
    system integration (file dialogs, notifications, dark mode)
  - Vala compiles to C with GObject, giving native performance with a
    high-level syntax similar to C#
  - GTK4's GtkSourceView provides syntax highlighting without a WebView
  - .deb and .rpm are the expected distribution formats on Linux

  Architecture:
  - GTK4 + libadwaita for UI (sidebar, output panel, toolbar, dialogs)
  - GtkSourceView 5 for the code editor (syntax highlighting, line numbers)
  - Vala for application code (compiles to C via valac)
  - Subprocess spawning for toolchain invocation (cargo, clang, zig, swiftc)
  - Same ~/. local/share/rustic-playground/ storage layout as macOS version
  - Same rustic.toml manifest format — projects are portable between platforms

  Scope:
  - Core playground loop: edit, run (⌘R / Ctrl+R), streaming output
  - Multi-language support (Rust, C/C++, Zig, Swift)
  - Project and playground CRUD
  - Content files
  - Book examples (same chapter data, different UI)
  - Edition builds (reuse edition config concept from v0.3.3)
  - Theming via libadwaita (light/dark/accent, no custom Monaco themes)

  Not in scope (initially):
  - Live error checking (no cargo check integration — add later)
  - Monaco editor (GtkSourceView is the Linux equivalent)
  - Exact feature parity with macOS — Linux version ships core features first

  Packaging:
  - .deb for Debian/Ubuntu (apt install)
  - .rpm for Fedora/RHEL (dnf install)
  - Flatpak as a stretch goal
  - Build via Meson (standard GTK4/Vala build system)

  Items:
  1. GTK4 + Vala project scaffold with Meson build
  2. Application window: sidebar + editor + output panel layout
  3. GtkSourceView editor integration (Rust, C, Zig, Swift highlighting)
  4. Subprocess runner: spawn toolchain, stream stdout/stderr
  5. Project/playground CRUD (same storage format as macOS)
  6. rustic.toml manifest support
  7. Content files support
  8. Settings dialog (font, theme, toolchain paths)
  9. Toolchain detection (cargo, clang, zig, swiftc)
  10. Book examples (port chapter data)
  11. Edition config support (single-language vs power)
  12. .deb packaging (debian/ directory)
  13. .rpm packaging (spec file)
  14. CI: build and package for Ubuntu + Fedora

---

v0.3.1 — Read-Only Book Projects, Copy to Project, Learn Menu
  Status: complete — 2026-04-03

  Overview:
  Book projects (The Rust Book, The K&R C Book, The Swift Book) become read-only
  reference material. Editor non-editable, save/rename/delete disabled. "Copy to
  Project…" lets users copy any book playground into their own project. Books are
  surfaced prominently via Learn menu, project switcher, and empty state.

  Completed:
  1. Read-only editor mode for book projects (Monaco readOnly + domReadOnly)
  2. Save/rename/delete disabled for book playgrounds and book projects
  3. Menu items dynamically disabled for book projects (is_book_project param)
  4. Custom Cut/Paste menu items (disabled for read-only, replacing predefined)
  5. Sidebar context menu: hides Rename/Delete, shows "Copy to Project…" for books
  6. ProjectSwitcher: hides rename/delete for book projects
  7. CopyToProjectModal — pick target user project + playground name
  8. Backend copy_playground_to_project command (writes to target project directly)
  9. Project source metadata in rustic.toml (source field in ProjectInfo)
  10. Per-playground locking: locked Vec<String> in manifest, toggle per playground
  11. Lock toggle button in toolbar (red=locked, green=unlocked, disabled for books)
  12. Read-only pill indicator in toolbar for book projects
  13. Duplicate Project: clears source + readonly so copy becomes user project
  14. About modal: version derived from tauri.conf.json (getVersion())
  15. Shortcut key badges: system UI font for proper ⌘/⇧ rendering
  16. Editor onChange: Svelte 5 callback prop (fixed silent failure with old pattern)
  17. Toolbar button restyle: Save=green, Share=yellow, Run=orange (all solid)
  18. "Learn" top-level menu: book submenus moved from Help for discoverability
  19. Books section in ProjectSwitcher: macOS-style flyout submenus for loaded books,
      "Load" tag for unloaded books, always visible at bottom of dropdown
  20. Empty state: "Learn from examples" buttons for unloaded books
  21. Consistent book naming: "The Rust Book", "The K&R C Book", "The Swift Book"
  22. Grouped project list with search filter in ProjectSwitcher
  23. Remove book: backend command + Help/Learn menu integration
  24. Help menu restructured: books in Learn, Help has only Help + About
  25. Zig theme: warm amber palette (#f7a41d accent) — CSS + Monaco editor
  26. Swift theme: warm coral palette (#f05138 accent) — CSS + Monaco editor
  27. "Auto (match language)" theme: switches theme based on active project type
  28. Theme selector: dropdown replacing segmented control (7 themes + auto)
  29. SVG gear icon in toolbar replacing Unicode character (proper centering)

  30. Sidebar lock icons next to locked/read-only playgrounds
  31. Lock button colors (red/green) verified across all 7 themes

---

v0.2 — Clang (C/C++) Projects
  Status: complete — released 2026-04-03

  Shipped:
  1. rustic.toml project manifest (type, paths, build flags, toolchain)
  2. Project type selection: Rust vs Clang (C/C++) in new project dialog
  3. Clang playground CRUD (list/new/load/save/rename/delete/duplicate)
  4. Compile and run: clang/clang++ dispatch by extension, stdin, process kill
  5. Compiler flags UI: sidebar panel with C/C++ flag inputs, saved to rustic.toml
  6. New playground dialog: language tabs (C/C++), 8 templates each
  7. Sidebar: file badges (C/C++ in sea green), compiler flags panel
  8. Monaco language detection (.c → "c", .cpp → "cpp")
  9. Conditional menus: K&R book for Clang, Rust book for Rust
  10. Toolchain wizard: tabbed Rust-first layout with C/C++ status tab
  11. Clang export: POSIX shell runner, Makefile, flags.sh, README
  12. K&R C Book examples (8 chapters, 16 playgrounds)
  13. Sea green theme (Monaco + app chrome)
  14. Settings: clang version display with install instructions
  15. Rust export: added README for consistency

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
