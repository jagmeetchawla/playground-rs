SPECIFICATION

Status
- Version: v0.4
- Date: 2026-07-10
- Owner: Jagmeet Chawla

---

Product

What
  Rustic Playground v0.4 — Multi-Version Toolchain Picker + Rust 1.9 support.

  Turn the toolchain pill in the toolbar into a first-class version picker.
  Users can see all installed Rust toolchains, switch between them per-project,
  and install new ones from within the app. Per-project pinning uses the
  standard `rust-toolchain.toml` file so projects mirror how real Rust
  projects manage toolchains.

  Bundled with active testing/support for Rust 1.9x and a small set of
  templates that showcase modern Rust features (let-chains, let-else, and
  similar). MIN_RUST floor stays at 1.85 (edition 2024 requirement); this
  release does not raise the minimum.

Why
  Since v0.3.5 the toolchain pill has been a passive health indicator:
  green/yellow/red, no interaction. Every real Rust workflow eventually
  needs multi-version awareness — testing against nightly, verifying MSRV,
  or working on a project that pins an older toolchain. Sending users
  back to the terminal for `rustup toolchain install` breaks the
  "no terminal required" promise.

  Also, Rust has moved on — the current stable is well past 1.85, and our
  docs and UI still reference 1.85 as if it were current. The version bump
  copy work is small and pairs cleanly with the picker.

Design Principle
  rust-toolchain.toml is the source of truth for a project's toolchain.
  The app tracks a session-level "active toolchain" that seeds new projects
  and provides a fallback when a project's pinned toolchain isn't installed.
  We never invent state that rustup already tracks.

  Resolution order for run/check:
    1. If project has rust-toolchain.toml AND that toolchain is installed
       → use it (`rustup run <name> cargo …`)
    2. Otherwise → use app's active toolchain
    3. If active toolchain isn't installed → fall back to rustup default

  Backward compat: existing projects (no rust-toolchain.toml) work exactly
  as they do today — bare `cargo` invocation, which respects rustup's default.

---

Scope

In scope
  - Multi-version toolchain picker (backend + frontend)
  - Per-project toolchain pinning via `rust-toolchain.toml`
  - New-project default → current active toolchain
  - Rust 1.9 support pass (testing, doc/copy updates)
  - 2-3 new templates showcasing modern Rust syntax
  - Cargo.lock housekeeping (v0.3.6 stale bump, already committed)

Out of scope
  - Multi-version support for Zig/Clang/Swift (frozen per project strategy)
  - Automatic toolchain install when a project pins something missing
    (v0.4.x follow-up; v0.4 just falls back to active)
  - Raising MIN_RUST above 1.85
  - Any editor changes (notebook mode, inline prose blocks) — separate release
  - Any port work (Linux, Windows, TUI) — still parked

---

Architecture — Backend

New commands (all in `src-tauri/src/cargo_commands.rs` or a new module)

  list_rust_toolchains() → Result<Vec<ToolchainInfo>, String>
    Runs `rustup toolchain list --verbose` (verbose gives paths + version).
    Parses each line into:
      ToolchainInfo {
        name: String,            // e.g. "stable-aarch64-apple-darwin"
        short_name: String,      // e.g. "stable" (channel) or "1.90.0"
        version: Option<String>, // e.g. "1.90.0" (rustc version)
        is_rustup_default: bool, // marked with "(default)" in rustup list
        is_active: bool,         // matches app's tracked active
      }

  set_active_toolchain(name: String) → Result<(), String>
    Writes to app config.json (new field: active_toolchain).
    Fires an event so frontend can re-render pill + downstream state.

  install_toolchain(name: String, on_progress: Channel<String>) → Result<(), String>
    Spawns `rustup toolchain install <name>`, streams stdout/stderr line-by-line
    via Channel (reuse infrastructure from v0.3.5 rustup update).
    Handles common errors: network, invalid name, already installed (noop).

  get_project_toolchain(project_path: String) → Result<Option<String>, String>
    Reads rust-toolchain.toml if present. Returns Some(name) or None.

  set_project_toolchain(project_path: String, name: String) → Result<(), String>
    Writes rust-toolchain.toml with the specified toolchain.

Config schema changes (`config.json`)
  Add field: active_toolchain: Option<String>
  On first launch (or when None): populate from `rustup show active-toolchain`.

Run/check pipeline changes
  Modify run_playground and check_playground:
    let toolchain = resolve_toolchain(project_path, app_config);
    let (program, base_args) = match toolchain {
      Some(name) => ("rustup", vec!["run", &name, "cargo"]),
      None       => ("cargo", vec![]),  // shouldn't happen with active fallback
    };

  resolve_toolchain(project_path, config) → Option<String>:
    1. Read rust-toolchain.toml → parse toolchain.channel field
    2. Check if that toolchain is installed (query list_rust_toolchains)
    3. If installed → return that name
    4. Else → return config.active_toolchain
    5. Else → return None (bare cargo, which will use rustup default)

New-project scaffold
  Modify create_project (Rust variant only — other langs frozen):
    After scaffolding Cargo.toml + src/bin/, write rust-toolchain.toml:
      [toolchain]
      channel = "<app's active_toolchain>"

  If active_toolchain is None (edge case), don't write the file.

rust-toolchain.toml format
  Standard rustup format:
    [toolchain]
    channel = "stable"           # or "1.90.0", "nightly", etc.

  We only touch the `channel` field. `components`, `targets`, `profile`
  are respected if present but not written by us.

---

Architecture — Frontend

Toolchain pill dropdown
  Current: static badge showing rustc version + health color
  New: clickable → dropdown menu

  Dropdown content:
    ┌─────────────────────────────────┐
    │ Active toolchain                │
    │ ├─ ✓ stable (1.90.0)            │
    │ ├─   nightly (2026-07-01)       │
    │ ├─   1.85.0                     │
    │ ├─ ─────────────────            │
    │ ├─   Install Toolchain…         │
    │ └─   Manage in Settings…        │
    └─────────────────────────────────┘

  Click a toolchain → invoke set_project_toolchain (if project loaded)
    AND set_active_toolchain (updates session default for new projects).
    Pill re-renders with new active.

  Click "Install Toolchain…" → opens InstallToolchainDialog.

  Soft "install newer stable?" hint (v0.4 addition):
    Prepended to the dropdown ONLY when the user has no stable-channel or
    pinned-semver toolchain at or above LATEST_KNOWN_STABLE (a backend
    constant, e.g. "1.96.0", bumped each Rustic Playground release).

    Semantics:
      - Compare installed versions, not just the active toolchain
        (someone deliberately using 1.85 for MSRV testing while having
        1.96 installed elsewhere → NO hint)
      - Skip beta/nightly when computing "have latest stable"
      - Hint appears above the toolchain list; clicking it opens the
        Install Toolchain dialog pre-filled with the recommended version
      - Never surfaced outside the dropdown — no unsolicited nag

    Backend commands used:
      - get_latest_known_stable() → "1.96.0"
      - list_rust_toolchains() → each ToolchainInfo has .version populated
      - frontend does the max-version comparison

Install Toolchain dialog
  Modal with:
    - Radio: stable / beta / nightly / specific version
    - Text field: version (e.g. "1.90.0"), enabled when "specific version" selected
    - Install button → invokes install_toolchain, opens a streaming console
      panel showing rustup output live
    - Cancel button
    - On success: closes dialog, refreshes toolchain list, pill updates

  Reuse Channel-based streaming console from v0.3.4 install flow.

State updates
  Add to App.svelte $state:
    let availableToolchains: ToolchainInfo[] = $state([])
    let activeToolchainName: string | null = $state(null)

  Load on app boot + on installer completion.

  Menu integration: no menu changes (pill dropdown handles everything;
  no "Toolchains" menu needed for v0.4).

Copy updates
  Files to touch for Rust 1.9 positioning:
    - ui/src/lib/ToolchainWizard.svelte — messaging referencing "Rust 1.85"
      or "current stable" should read appropriately for 1.9x-current era
    - ui/src/lib/SettingsModal.svelte — same
    - ui/src/lib/HelpModal.svelte — Rust section
    - docs/index.html (rusticplayground.dev) — separate repo, deferred

---

Templates (Rust 1.9 showcase)

Add to `ui/src/lib/templates.ts`:

  1. "let-chains" (stable in 1.88)
     - Demonstrates: `if let Some(x) = foo && x > 0 { ... }`
     - Use case: teaching modern conditional binding
     - Cargo.toml: rust-toolchain.toml set to stable (needs 1.88+)

  2. "let-else patterns" (stable in 1.65 but underused)
     - Demonstrates: `let Ok(value) = parse() else { return; };`
     - Use case: early-return refactoring pattern

  3. "Async closures" (if stable at target 1.9x version)
     - Confirm stability status before including
     - If not stable → skip and add third template as "std API tour"

  Templates should live at:
    ui/src/lib/templates.ts entries under a "Modern Rust" section
    (or just interspersed with existing — decide during implementation)

---

Implementation Phases

Phase 1 — Backend commands (2-3 hours)
  - list_rust_toolchains
  - set_active_toolchain
  - install_toolchain (Channel-streamed)
  - get_project_toolchain / set_project_toolchain
  - Config schema addition + first-launch bootstrap
  Checkpoint: `cargo test`, manual invoke via developer tools works.

Phase 2 — Run/check integration (1-2 hours)
  - resolve_toolchain helper
  - Modify run_playground + check_playground to prepend `rustup run <name>`
  - Preserve existing behavior when no toolchain resolved
  Checkpoint: run a project with rust-toolchain.toml → verify correct toolchain
  used (e.g., pin to nightly, `println!("{}", cfg!(rustc_version_bigger_than_1_50)))`).

Phase 3 — Frontend pill + dialog (2-3 hours)
  - Convert toolchain pill to dropdown
  - Wire commands to UI
  - Build InstallToolchainDialog with streaming console
  - Reactive state updates on install completion / switch
  Checkpoint: click pill, switch toolchain, install a new one — all work.

Phase 4 — New-project default (1 hour)
  - Wire active toolchain into create_project scaffold
  - Verify rust-toolchain.toml appears in new projects
  Checkpoint: create new project → rust-toolchain.toml present with correct
  channel; run project → uses that channel.

Phase 5 — Rust 1.9 support pass + templates (1-2 hours)
  - Sanity check parse_rust_version against 1.9x variants
  - Copy sweep across UI (ToolchainWizard, Settings, Help)
  - Add 2-3 new templates
  Checkpoint: fresh install with 1.9x rustc shows appropriate messaging;
  new templates compile and run.

Phase 6 — Testing + release (1 session)
  - Vanilla VM first-launch scenarios
  - Multi-toolchain scenarios (2+ installed, switching between them)
  - Backward-compat scenarios (existing projects with no rust-toolchain.toml)
  - Missing-toolchain scenario (pin to 1.42.0, don't install → fall back)
  - Version bump, sync-version, build, notarize, tag, release

---

Scope Boundaries — What v0.4 Does NOT Include

  - No automatic install when project pins a missing toolchain
    (falls back to active; v0.4.x follow-up: prompt to install)
  - No components/targets management (only `channel` field of rust-toolchain.toml)
  - No global "add toolchain" workflow outside project context
    (Install Toolchain dialog is available from pill regardless of project,
    but there's no separate "Manage toolchains" full screen)
  - No changes to Zig/Clang/Swift toolchain handling
  - No editor changes (notebook, inline prose)
  - No port work
  - No raise of MIN_RUST above 1.85

---

Migration & Backward Compatibility

Existing projects
  - No rust-toolchain.toml → nothing changes. Continue to work with bare cargo.
  - User can opt in per-project by switching toolchain via the pill.

Existing user config (config.json)
  - Missing `active_toolchain` → on first v0.4 launch, populate from
    `rustup show active-toolchain`. Serde default handles absence.

Downgrade path
  - If user downgrades to v0.3.x after using v0.4, the rust-toolchain.toml
    files stay. Older versions will invoke bare cargo, which itself respects
    rust-toolchain.toml (via rustup shim behavior). So projects still work
    with pinned toolchains — the app just doesn't show a picker UI.
