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

NEXT UP
───────

v0.3.5 — Rust Toolchain Version Gate + In-App Update
  Status: SHIPPED — 2026-04-15
  See v0.3.5 entry under RELEASED (continued) below for the shipped summary.

  (Historical plan preserved for reference.)
  Priority: ASAP — shipped before broader distribution push.

  Problem:
  We display the rustc/cargo version in the toolchain pill and wizard, but
  we never enforce a minimum. All generated Cargo.toml files use
  `edition = "2024"`, which requires rustc ≥ 1.85 (Feb 2025). A user on
  an older toolchain sees green status everywhere, then their first
  playground fails to compile with a confusing edition error. Zig already
  has this gate (`version_ok` on 0.15.x); Rust does not.

  Plan (mirrors Zig's pattern + v0.3.4 toolchain installer infra):

  Backend — src-tauri/src/cargo_commands.rs
    - Add MIN_RUST = (1, 85, 0) constant (edition 2024 floor)
    - Parse rustc --version → semver tuple, compare to MIN_RUST
    - Add `version_ok: bool` to the rust block in check_toolchain JSON
    - New Tauri command: update_rust → spawns `rustup update stable`,
      streams stdout/stderr to a console block (reuse the streaming
      infra from v0.3.4's install_toolchain command)
    - Edge: if rustup is missing but rustc is outdated, return
      version_ok: false with a hint to install rustup first
    - Edge: if version parsing fails, default version_ok: true (don't
      block users on parser bugs)

  Frontend
    - App.svelte: add rustInfo.version_ok mirroring zigInfo; thread into
      projectStatus() so the toolchain pill renders YELLOW (not green,
      not red) when Rust is installed but outdated
    - ToolchainWizard.svelte: yellow row + "Update Rust" button when
      !version_ok, with the same UX as the existing repair flow
    - SettingsModal.svelte: same "Update Rust" affordance under the
      Rust section
    - Tooltip / message copy: "Rust 1.85+ required for edition 2024.
      Click to run `rustup update stable`."

  Test plan
    - Vanilla VM with old rustup-installed toolchain: pin to 1.74,
      verify yellow pill, click Update Rust, verify success and
      re-check turns green
    - Vanilla VM with no rustup: verify version_ok: false plus the
      "install rustup first" hint
    - Dev machine (current toolchain): verify nothing visually
      changes — green stays green

  Why ASAP: as soon as we publicize the website/DMG, users on stale
  Rust toolchains will hit confusing edition-2024 errors on their first
  playground. This needs to land before the launch announcements go out.

---

Distribution & Launch (post v0.3.4)
  Status: not started — 2026-04-06

  The code freeze for v0.3.4 is in. Next phase is shipping it to users.
  See specs/release-plan.md for the full 16-step launch checklist.

  Tasks:
  - DMG visual identity — DEFERRED. We attempted custom background +
    custom installer file icon during 2026-04-07 testing. The .dmg file
    icon worked, but the mounted volume icon stayed as the app icon
    (Tauri stamps it via .VolumeIcon.icns and there's no clean way to
    override without a heavier post-build hdiutil mount/inject/recompress
    pipeline). Decision: ship with stock Tauri DMG behavior for v0.3.4
    Phase 1 — accept that the .dmg file looks similar to the app. Revisit
    when there's appetite for the full mount/inject/recompress pipeline,
    or when DMG identity becomes a real user complaint post-launch.
  - Per-edition icons (art task — can ship with same icon initially)
  - Build DMGs via scripts/build-editions.sh (Rust Edition + Power Edition)
  - Code-sign + notarize DMGs (macOS Gatekeeper requirement)
  - GitHub Releases: upload DMGs as v0.3.4 release assets
  - Website launch: rustic-playground.app on GitHub Pages
  - Website videos (2026-04-12): 5 Focusee screen recordings (30-45s each)
    for the "How it works" section. One video per step:
      1. Create a project
      2. Write a playground
      3. Press ⌘R
      4. Iterate
      5. Make it yours (settings: theme, font, editor config)
    Format: short, focused, no narration (silent with captions or just
    the app in action). Embed inline under each step or as a
    click-to-play thumbnail. Tool: Focusee (macOS screen recorder).
  - Wiki / FAQ pages (Zig 0.15 pinning, edition differences, install guide)
  - Staggered community announcements over 5 days

  Phase 1: Rust Edition first (sharpest pitch — one language, one audience)
  Phase 2: Power Edition a few weeks later

  - FEATURE: Share Playground — generate permanent sharing links.
    User request (2026-04-12). "Generate a Permanent Sharing Link" → "Perfect
    for Discord, GitHub, or StackOverflow" → "Collaborate with a Click."
    Goal: one-click share of a playground's code as a permanent, clickable URL.
    Recipients can view the code, copy it, or import it into their own Rustic
    Playground.

    Requires a cloud backend — playground code needs to live somewhere
    addressable by URL. Options:
      (a) GitHub Gist integration — authenticate via GitHub OAuth, create a
          gist per share, return the gist URL. Pros: permanent, well-known,
          syntax highlighting built in, no server to maintain. Cons: requires
          GitHub account + OAuth flow, gists are public or require paid GitHub.
      (b) Custom share service — lightweight API (e.g. Cloudflare Workers +
          KV store) that accepts code payloads, returns a short URL like
          rustic-playground.app/s/<id>. Pros: branded, no GitHub dependency,
          can add features (expiry, edit, fork). Cons: we run a service, need
          abuse controls, storage costs.
      (c) iCloud-backed sharing — store shared snippets in the user's iCloud
          container, generate a CloudKit share link. Pros: no third-party
          accounts, Apple-native. Cons: recipient needs iCloud, complex API,
          CloudKit sharing is designed for collaboration not public links.
      (d) URL-encoded (no backend) — compress code with gzip, base64-encode,
          embed in URL fragment: rustic-playground.app/share#<payload>. Website
          decodes and renders. Pros: zero infrastructure, truly permanent (it's
          in the URL). Cons: URL length limits (~2000 chars safe, ~8000 max in
          modern browsers), won't work for large playgrounds, ugly URLs.

    Recommended approach: Start with (d) for small playgrounds (under ~4KB of
    source), fall back to (a) GitHub Gist for larger ones. Evaluate (b) custom
    service if we want a branded experience. Discuss before implementing.

    Share flow (frontend):
      1. User clicks Share button (toolbar or menu) or presses shortcut
      2. Current playground code is captured from the editor
      3. Code is compressed + encoded into a share URL (or uploaded to backend)
      4. URL is copied to clipboard with a success toast
      5. Optional: show a modal with the URL, a "Copy" button, and preview of
         how it will look to recipients

    Receive flow (website):
      • rustic-playground.app/share#<payload> or /s/<id>
      • Website renders the code with syntax highlighting (Monaco or Prism.js)
      • "Open in Rustic Playground" button (deep link via custom URL scheme)
      • "Copy Code" button for users without the app
      • Language badge (Rust/C/C++/Zig/Swift) and metadata

    App-side receive (deep link):
      • Register rustic-playground:// URL scheme in Tauri config
      • Handle rustic-playground://share?code=<payload> — opens app, creates
        a new playground from the shared code, or opens it in a scratch buffer
      • Alternatively: just copy to clipboard and let user paste — simpler v1

    Menu / toolbar integration:
      • New menu item: Playground → "Share Playground…" (⌘⇧S or ⌘⇧A)
      • Toolbar: add to existing export dropdown, or new share button
      • Disabled for empty/unsaved playgrounds

    Metadata to include in share:
      • Code (required)
      • Language / project type (rust/clang/zig/swift)
      • Playground name (optional — for display)
      • App version that created the share (for compatibility)
      • Timestamp

    iCloud angle:
      • User mentioned iCloud as a possible backend. iCloud sync (already
        spec'd above as a separate feature) handles project sync across
        the user's own devices. Sharing is different — it's about sending
        code to OTHER people. iCloud's CloudKit sharing could theoretically
        do this but it's designed for collaboration (inviting specific Apple
        IDs), not public link sharing. Keep iCloud for device sync, use a
        different mechanism for public sharing.

    Collaboration (future):
      • v1 is one-way sharing (read-only links). Real-time collaboration
        (two users editing the same playground) is a much larger feature
        that would need operational transform / CRDTs, presence indicators,
        cursor sharing, etc. Park for post-v1. Could build on CloudKit or
        a WebSocket service.

    Priority: Medium. Strong "show and tell" value for learners sharing code
    on Discord/Reddit/StackOverflow. The URL-encoded approach (d) could ship
    quickly as a v1 with no backend dependency.

  - FEATURE: Cloud Projects — full shared projects hosted in the cloud.
    User request (2026-04-12). Natural evolution of the Share Playground
    feature. Instead of sharing a single playground as a read-only snippet,
    host an entire project (all playgrounds, Cargo.toml, content files) in
    the cloud so multiple users can access, fork, and collaborate on it.

    Why this matters:
      • Share Playground (above) covers the "show a snippet" use case. But
        real learning happens in projects with multiple playgrounds, shared
        dependencies, and content files. A workshop instructor wants to share
        a 10-playground project with students — not 10 individual links.
      • "Open this project in Rustic Playground" becomes a one-click onboard
        for tutorials, courses, blog posts, and conference workshops.
      • Combines the distribution model of GitHub repos with the instant-run
        experience of the app — no git clone, no cargo build, just open.

    Cloud backend requirements:
      • Project storage — full project tree (Cargo.toml, rustic.toml,
        src/bin/*.rs, content/*) stored server-side. NOT build artifacts.
      • User accounts — at minimum, identify who owns a project. Could
        piggyback on GitHub OAuth (same as Gist sharing) or Apple Sign In.
      • Project URLs — rustic-playground.app/p/<owner>/<project> or
        rustic-playground.app/p/<short-id>
      • Access control — public (anyone can view/fork), unlisted (link-only),
        private (owner only). Start with public + unlisted.
      • Forking — "Fork to My Projects" downloads the cloud project into
        the user's local app as a new editable project. One-way copy, not
        a live sync (that's collaboration, below).
      • Versioning — at minimum, snapshots (user manually publishes a new
        version). Git-style history is overkill for v1.

    Relationship to other features:
      • Share Playground (above) = lightweight, no account, single file.
        Cloud Projects = heavier, requires account, full project.
        They coexist — Share for quick snippets, Cloud for full projects.
      • iCloud Sync (above) = private, across the user's OWN devices.
        Cloud Projects = public/shared, across DIFFERENT users.
        Different use cases, different backends, different auth models.

    Backend options:
      (a) GitHub-backed — each cloud project is a GitHub repo (or gist)
          under the user's account. Pros: free hosting, version control
          built in, familiar. Cons: requires GitHub account, repo clutter,
          can't control the UX of the hosted page.
      (b) Custom service — API server (Cloudflare Workers / Fly.io /
          Railway) + object storage (R2 / S3) + database (D1 / Turso).
          Pros: full control, branded URLs, custom features. Cons: we
          run infrastructure, costs scale with users, abuse controls needed.
      (c) CloudKit (Apple) — store projects in a public CloudKit database.
          Pros: free tier generous (10 GB assets, 100K records), Apple-
          native, no separate account for Apple users. Cons: Apple-only
          (non-Apple recipients can't access), CloudKit API is complex,
          web access requires CloudKit JS (limited).
      (d) Hybrid — GitHub for storage + our service for metadata/discovery.
          User authenticates with GitHub once, we create repos on their
          behalf, our API indexes them for search/browse. Pros: combines
          GitHub's reliability with our UX. Cons: complexity of two systems.

    Recommended approach: Evaluate (b) custom service if we want full
    control over the experience, or (d) hybrid if we want to leverage
    GitHub without running heavy storage. Discuss before committing —
    running a service is a different kind of commitment than shipping
    a desktop app.

    UX flow — publishing:
      1. User opens a project in the app
      2. Project → "Publish to Cloud…" (or Share → "Publish Project…")
      3. First time: OAuth sign-in (GitHub or Apple)
      4. Modal: project name, description, visibility (public/unlisted)
      5. Upload: all project files serialized and pushed to cloud backend
      6. Success: shareable URL copied to clipboard + toast
      7. Updates: "Update Published Project" pushes latest local state

    UX flow — consuming:
      1. Recipient clicks rustic-playground.app/p/<owner>/<project>
      2. Website renders: project overview, playground list, code preview
      3. "Open in Rustic Playground" button (deep link → app imports project)
      4. "Fork to My Projects" button (downloads as new local project)
      5. Without the app: browse code on the website, copy individual files

    App-side import:
      • Deep link: rustic-playground://project?url=<cloud-url>
      • App downloads all project files, creates a new local project
      • Marks it with source = "cloud:<url>" in rustic.toml for update tracking
      • User can pull updates from the published version (like git pull)

    Workshop / classroom use case:
      • Instructor publishes a project with 10 playgrounds + exercises
      • Students click one link → project appears in their app, ready to run
      • Students work locally — no live sync, no conflict resolution needed
      • Instructor can publish updates; students see "Update available" badge

    Content beyond code:
      • Cloud projects include content/ files (data files, images, text)
      • Cargo.toml with dependencies — recipients get the same deps
      • rustic.toml metadata — project type, build flags carry over

    What this does NOT include (v1):
      • Real-time collaboration (two users editing simultaneously)
      • Live sync (changes propagate automatically between users)
      • Comments, issues, or discussion threads on projects
      • Pull requests or merge workflows
      • Project discovery / marketplace / browse page (v2 maybe)
      • Billing or paid tiers

    Dependencies:
      • Requires Share Playground to be designed first (shared infra:
        OAuth, URL scheme, website share pages)
      • Requires cloud backend decision — can't be URL-encoded like
        single-playground shares (projects are too large)
      • Requires user account system (GitHub OAuth or Apple Sign In)

    Priority: LOW for now. This is a significant infrastructure commitment
    (running a service, user accounts, abuse controls, storage costs).
    Share Playground covers 80% of the sharing need with much less effort.
    Revisit after Share Playground ships and we see demand for full-project
    sharing — especially from workshop/classroom users.

  - FEATURE: Crate Manager — search and add dependencies from crates.io.
    User request (2026-04-12). "Search 100k+ Crates via crates.io" → "Add
    Dependencies Instantly" → "No Manual Cargo.toml Edits Required."
    Goal: in-app crate discovery and dependency management. Users search
    crates.io, pick a crate, and it's added to their project's Cargo.toml
    automatically — no terminal, no manual TOML editing.

    Current state:
      • The app already has a basic "Add Dependency" button in the toolbar
        (cargo_commands.rs: add_dependency / remove_dependency commands).
      • It takes a crate name + version string and writes to Cargo.toml.
      • Known issue (v0.3.3): button doesn't focus the input, no search,
        no validation against crates.io, no version picker.

    Proposed UX:
      • New "Manage Dependencies" modal (⌘⇧D or via toolbar button)
      • Search bar at top — live search against crates.io API as user types
        (debounced, ~300ms, like the editor's live check)
      • Search results: crate name, description, latest version, download
        count, last updated. Paginated or virtual-scrolled for large results.
      • Click a result → expands to show: full description, recent versions
        dropdown, feature flags checkboxes, "Add to Project" button
      • Current dependencies listed below search (or in a separate tab):
        shows all [dependencies] from Cargo.toml with version, "Remove"
        button, "Update" button (if newer version exists on crates.io)
      • Version picker: dropdown showing recent versions (latest 5-10),
        with "Latest" as default. Shows semver compatibility hint.
      • Feature flags: checkboxes for optional features (crates.io API
        exposes these). Common pattern: serde with "derive" feature.

    crates.io API:
      • Search: GET https://crates.io/api/v1/crates?q=<query>&per_page=20
      • Crate detail: GET https://crates.io/api/v1/crates/<name>
      • Versions: GET https://crates.io/api/v1/crates/<name>/versions
      • No auth required for read-only access
      • Rate limit: 1 req/sec (respect via frontend debounce + backend
        rate limiting). User-Agent header required.
      • All requests from Rust backend (Tauri command), not frontend JS —
        avoids CORS issues and keeps network access in the backend layer.

    Backend commands:
      • search_crates(query: String, page: u32) -> CrateSearchResult
        Hits crates.io search API, returns Vec<CrateSummary>
      • get_crate_details(name: String) -> CrateDetail
        Hits crates.io crate detail API, returns full info + versions
      • add_dependency(project, crate_name, version, features) — enhanced
        version of existing command, now with features support
      • remove_dependency(project, crate_name) — already exists
      • update_dependency(project, crate_name, new_version) — new
      • list_dependencies(project) -> Vec<Dependency> — parse Cargo.toml

    Cargo.toml writing:
      • Use toml_edit crate (preserves formatting, comments, ordering) —
        NOT string manipulation or serde round-trip (which loses formatting).
      • Handle both simple (`serde = "1.0"`) and table (`serde = { version
        = "1.0", features = ["derive"] }`) dependency formats.
      • After writing, trigger cargo check in background to verify the
        dependency resolves (surface errors immediately, don't wait for
        next ⌘R).

    Offline behavior:
      • Search requires network. Show clear "No internet connection" state
        instead of silent failure.
      • Manual add (type crate name + version directly) should still work
        offline — it writes to Cargo.toml without validation. Add a warning
        toast: "Added without verification — will validate on next build."

    Book projects:
      • Disable entirely for book projects (read-only).

    Language scope:
      • Rust only for v1. Zig (build.zig.zon), Swift (Package.swift), and
        C/C++ (no package manager) are out of scope.
      • The modal title/UI should make it clear this is Rust/Cargo specific.
        "Cargo Dependencies" or "Crate Manager" — not generic "Dependencies."

    Power Edition considerations:
      • Only show the crate manager when active project is Rust type.
      • Menu item and shortcut disabled for non-Rust projects.

    Priority: HIGH. This is one of the most-requested beginner friction
    points — "how do I add rand to my playground?" Currently requires
    knowing TOML syntax and crate version numbers. A searchable UI with
    one-click add removes that barrier entirely.

---

RELEASED (continued)
───────────────────

v0.3.4 — In-App Toolchain Installer & Repair
  Status: complete — released 2026-04-06

  Overview:
  Users no longer need a terminal to get a working Rust toolchain. The app
  detects what's broken (no rustup, no default toolchain, missing components),
  shows a clear status card, and runs the right fix in-app — streaming the
  installer output live in a modal. The toolbar pill becomes a one-click
  entry point: ● green when healthy, ◐ yellow when components are missing,
  ○ red when nothing is installed.

  Completed:
  1. ToolchainFixWizard.svelte — self-contained modal with status card,
     fix actions, and live-streaming output panel (auto-scroll, fixed-height
     log, hides detail grid during fix to free vertical space)
  2. Backend run_toolchain_fix command with InstallRustup / SetDefaultStable /
     AddComponent actions, streamed via Tauri Channel
  3. rust_state cascade in check_toolchain: not_installed → no_default →
     missing_components → healthy (mutually exclusive, ordered by severity)
  4. Toolbar pill is now a button — opens fix wizard for Rust, opens Settings
     → Toolchains for other languages. Status dot (●/◐/○) reflects full
     rust_state, so missing rustfmt/clippy correctly shows yellow
  5. "Install rustfmt & clippy" combined shortcut button when both are
     missing — runs fixes sequentially, streams all output into one log
  6. Per-language menu restructure: replaces old "Learn" menu. Single-language
     editions get a top-level menu named after the language (e.g. "Rust");
     Power Edition gets a "Languages" menu with per-language submenus. Each
     language menu contains "Rust Toolchain…" (Rust only) plus a book
     sub-submenu (Load / Remove / Read Online) for languages with books
  7. Settings/Wizard "Repair Toolchain…" button reuses the same fix wizard,
     z-indexed above Settings so it layers correctly. Settings auto-refreshes
     after the fix completes via a refreshKey-driven silent re-check
  8. RUSTUP_AUTO_INSTALL=0 on read-only probes (check_toolchain,
     get_toolchain_info, check_playground) — rustup ≥1.28 silently
     auto-installs the default toolchain when proxies are invoked, which
     would mask broken state. The env var disables that behavior so users
     see (and can fix) the actual state. Run command intentionally left
     auto-install-enabled: explicit user action, implicit fix is welcome
  9. Synced terminology across Settings, Wizard, and FixWizard:
     "● Rust toolchain is healthy / Everything is installed and ready to use"

  Tested branches: healthy, missing_components (rustfmt and/or clippy),
  no_default (rustup default none), not_installed (rustup self uninstall),
  clt_missing (Xcode Command Line Tools absent — added 2026-04-07).
  All five verified end-to-end on a vanilla macOS VM.

  Known Issues (discovered during 2026-04-08 testing):
  - Power Edition: when CLT is missing and the user selects ONLY C/C++
    (not Rust), the wizard offers no way to install/repair CLT in-app.
    The Install Rust Toolchain… button only appears when Rust is selected
    because the FixWizard is wired to rust_state, not to a global CLT
    prerequisite. C/C++ users see "clang not found" with a static
    `xcode-select --install` Terminal command but can't trigger the
    repair from inside the app the way Rust users can.
    Scope: Power Edition only (Rust Edition forces Rust selection so this
    can't happen). Does NOT block v0.3.4 Rust Edition Phase 1.
    Workaround for users today: copy the `xcode-select --install` command
    from the wizard's clang section, run in Terminal, follow Apple's GUI
    installer, return to the app and click Re-check.
    Proper fix: aligns with the "Promote Xcode CLT to a global
    prerequisite" refactor below. Once CLT is a top-of-cascade prereq
    rendered ABOVE the per-language status, the Install Xcode CLT button
    becomes language-agnostic and shows for any selected language that
    needs it (Rust, C/C++, Swift). Same FixWizard, same auto-polling,
    same UX — independent of which language(s) the user enabled.
    Resolve as part of the CLT refactor. Discovered: 2026-04-08.

  - Power Edition fails to detect zig (and likely non-standard clang/swiftc
    installs) even when they're on the user's PATH and resolvable via
    `which zig` in Terminal.
    Root cause: macOS app bundles launch with a minimal PATH that does NOT
    include /opt/homebrew/bin, /usr/local/bin, ~/.local/bin, or any other
    shell-init PATH additions. We correctly work around this for Rust by
    resolving cargo/rustc via absolute paths in ~/.cargo/bin (see the
    tool_path() helper in cargo_commands.rs). For zig, the current code
    uses bare `zig version` and `which zig`, which fail in the bundled app
    even though they succeed in Terminal.
    Scope: Power Edition only. Rust Edition ships with only Rust enabled
    and the zig detection code is never hit. This does NOT block the
    Rust Edition Phase 1 release.
    Fix options (for Power Edition Phase 2):
      1. Probe common zig install locations in order: /opt/homebrew/bin/zig,
         /usr/local/bin/zig, ~/.local/bin/zig, /usr/bin/zig. Covers ~95% of
         Homebrew and manual installs. Simplest.
      2. Load the user's shell environment via `zsh -ilc 'echo $PATH'`
         (login interactive shell) and search the resulting PATH. Most
         robust but adds a shell invocation to every check_toolchain call.
      3. Add a user-configurable zig_path setting in Settings → Languages
         → Zig, same pattern as cargo_path. Most flexible but requires
         manual setup.
      4. Combine 1 + 3: probe common paths automatically, allow manual
         override via settings when the auto-detect misses.
    Same root cause likely affects:
      • Homebrew-installed LLVM clang (/opt/homebrew/opt/llvm/bin/clang)
      • Swift toolchain downloads (/Library/Developer/Toolchains/*.xctoolchain/usr/bin/swift)
      • Zig version managers (zvm, etc.) with non-standard install paths
    Aligns with: "Promote Xcode CLT to a global prerequisite" refactor
    and the broader subprocess discovery cleanup needed for Power Edition.
    Belongs in the same refactor sprint — resolve together.

  Post v0.3.4 Backlog:
  - DISCUSS: Switch CLT install from `xcode-select --install` to the
    Homebrew "softwareupdate placeholder" technique for fully in-CLI install.
    User raised (2026-04-08): Homebrew, rustup-init, and most macOS
    provisioning tools install Xcode Command Line Tools entirely from CLI
    without the GUI dialog popping up. They use an undocumented but stable
    Apple workaround. Worth understanding the technique and discussing
    whether to adopt it for Rustic Playground v0.4+. **Do NOT implement
    without discussion** — there are real tradeoffs.
    The technique:
      ```sh
      # Step 1: signal "user is requesting CLT install"
      sudo touch /tmp/.com.apple.dt.CommandLineTools.installondemand.in-progress
      # Step 2: softwareupdate now lists CLT as available
      PROD=$(softwareupdate -l | grep -E '\*.*Command Line Tools' \
        | sed -E 's/^[^C]+(Command Line Tools.*)/\1/' | sort -V \
        | tail -n 1 | tr -d '\n')
      # Step 3: install non-interactively (no GUI)
      sudo softwareupdate -i "$PROD" --verbose
      # Step 4: clean up
      sudo rm -f /tmp/.com.apple.dt.CommandLineTools.installondemand.in-progress
      ```
    Why it works:
      • macOS's softwareupdate daemon checks for the placeholder file to
        decide whether to expose CLT as an installable package. Without
        the file, `softwareupdate -l` hides CLT and treats it as a
        "needs the GUI flow" item. With the file, CLT shows up as a
        regular softwareupdate package and installs via the standard
        non-interactive softwareupdate -i path.
      • Apple's own GUI dialog uses this same internal IPC contract — the
        placeholder file is the "the user has consented, proceed" signal
        they pass to softwareupdate from the GUI shim. Third-party tools
        bypass the GUI and create the file directly.
      • Stable since macOS 10.10 (~2014). Works through Tahoe (26).
        Apple has never documented or removed it despite knowing tools
        rely on it — implicit understanding.
    Pros vs current xcode-select --install + polling:
      + No "find Apple's dialog hidden behind our app" UX problem
      + Real-time progress streamed to console (like a normal cargo build)
      + Synchronous: command returns when CLT is actually installed —
        no auto-polling loop, no 10–15 minute wait dance
      + FixWizard CLT branch becomes much simpler — drop the polling,
        drop the "find the dialog" callout, drop the elapsed timer
      + Matches Homebrew/rustup mental model — power users will recognize
        the pattern and trust it
    Cons:
      - Requires SUDO. Tauri apps can't directly elevate. Standard
        workaround is `osascript -e 'do shell script "..." with
        administrator privileges'` which pops macOS's native admin
        password prompt with our app's name. Cleaner than expected
        but it's another modal that some users find scarier than
        Apple's branded "Install Command Line Tools?" dialog.
      - "Rustic Playground wants to make changes, enter password" feels
        more invasive than Apple's blue-bordered system dialog, even
        though they're mechanically equivalent.
      - Sudo prompts can't be programmatically satisfied — must wrap
        multiple steps in one `do shell script` call to avoid re-prompts.
      - Depends on undocumented behavior. If macOS 27 or 28 changes the
        placeholder mechanism, our installer breaks and we ship a fix
        urgently. Homebrew can absorb this risk; we're smaller.
      - Privilege escalation triggers user trust scrutiny. Need a clear
        UI explanation: "we need admin to install Apple's developer
        tools, same way Homebrew does." Plus a Help section.
    What to discuss before deciding:
      1. Are users actually complaining about the hidden Apple dialog,
         or is it a paper cut we've already mitigated with the auto-poll?
      2. Are we comfortable depending on undocumented Apple behavior in a
         tool that targets beginners (who can't easily diagnose if it
         breaks)?
      3. Do we want a "headless / unattended" install mode for CI users
         and provisioning scripts? That's the strongest argument for
         this technique — `xcode-select --install` cannot be made
         unattended at all.
      4. Could this be a Power-Edition-only feature — Rust Edition stays
         on the safe Apple-blessed flow, Power Edition gets the
         power-user-friendly fast path?
      5. What's the regression risk vs the UX improvement?
    Sketch of how it would integrate with FixWizard:
      • Replace the InstallXcodeCLT FixAction handler in lib.rs:
         - Current: spawns `xcode-select --install`, returns in seconds
         - New: spawns `osascript -e 'do shell script "<all 4 steps>"
           with administrator privileges'`
         - Streams sudo command output via Tauri Channel like other fixes
      • Frontend FixWizard:
         - Drop the auto-polling code entirely
         - Drop the "Apple installer is now open" callout
         - Drop the "10-15 min" hint (it's still long, but progress is
           visible in the streaming log so the wait feels active)
         - Same Install button, same output panel, same "fixState success"
           handler — symmetry with InstallRustup
      • The regression-test signal: on a vanilla VM, clicking "Install
        Xcode Command Line Tools" should pop ONE macOS admin password
        prompt, then stream ~10-15 min of softwareupdate output to the
        console panel, then return success. No GUI dialogs from Apple
        ever appear.
    Trigger conditions for revisiting:
      • A user explicitly asks for fully unattended/headless CLT install
      • A user complains about Apple's dialog being hidden behind our app
      • We add CI/scripted-install support (provisioning Macs at scale)
      • macOS 27 or 28 makes the existing GUI dialog flow worse somehow
    Until then: stick with the current `xcode-select --install` flow.
    It works, it's Apple-blessed, and the auto-polling mitigates the
    main UX cost.

  - FEATURE: iCloud sync — projects, settings, and state across Macs.
    User request (2026-04-09). Full iCloud integration: projects sync
    across devices, settings follow the user, window state restores on
    any Mac. Use case: work on MacBook at coffee shop, continue on Mac
    Studio at home — same projects, same theme, same open tabs.
    Three layers:
      1. Project sync (iCloud Drive)
         • Move (or symlink) the projects folder into the app's iCloud
           container: ~/Library/Mobile Documents/iCloud~com.rustic-
           playground.<edition>/projects/
         • iCloud Drive handles upload, download, conflict resolution
         • Each .rs file, Cargo.toml, rustic.toml, content/ files all
           sync automatically
         • Conflict resolution: iCloud creates "... 2" copies on
           conflict. App should detect these and surface a merge UI
           or "pick a version" dialog. For v1, just surface the
           conflict visually and let the user choose.
         • Offline support: iCloud Drive caches locally, so the app
           works offline and syncs when connectivity returns.
         • Large files: target/ directories must be EXCLUDED from sync
           (build artifacts are huge, machine-specific, and
           regeneratable). Use .nosync extension or .gitignore-style
           exclusion via iCloud's resource values.
           `NSURL.setResourceValue(true, forKey: .isExcludedFromBackupKey)`
         • Book projects: sync or re-seed on each device? Re-seeding
           is simpler (book data is baked into the binary). Sync only
           user-created projects. Mark book projects with a "don't
           sync" flag in rustic.toml.
      2. Settings sync (NSUbiquitousKeyValueStore)
         • Theme, font size, font family, tab size, cargo path, enabled
           languages, wizard_completed — all sync via Apple's KVS.
         • NSUbiquitousKeyValueStore is limited to 1 MB total / 1024
           keys — more than enough for settings.
         • Changes propagate in seconds (faster than iCloud Drive).
         • Conflict resolution: last-write-wins is fine for settings
           (user's most recent preference is the right one).
         • Implementation: Tauri command that reads/writes KVS via
           Objective-C bridge. Frontend polls or listens for change
           notifications (NSUbiquitousKeyValueStoreDidChangeExternally).
      3. Window state sync (iCloud Drive or KVS)
         • window-state.json (size, position, sidebar width, output
           height, open tabs, active tab, layout) syncs so the user
           picks up where they left off on another Mac.
         • Caveat: window position is screen-dependent. If MacBook has
           a 14" display and Studio has a 27", restored position may
           be off-screen. Clamp to visible screen bounds on restore.
         • Open tabs reference playground names, which only make sense
           if the projects also synced. Dependency on layer 1.
    iCloud entitlements needed:
      • com.apple.developer.icloud-container-identifiers
      • com.apple.developer.icloud-services (CloudDocuments + KVS)
      • com.apple.developer.ubiquity-container-identifiers
      • Requires Apple Developer Program membership ($99/year) — same
        as signing/notarization. Can't test iCloud without it.
    Tauri + iCloud considerations:
      • Tauri doesn't have built-in iCloud support. Need to bridge
        to Apple's CloudKit / iCloud Drive APIs via Objective-C or
        Swift interop from Rust. Options:
          (a) Swift plugin compiled as a .dylib, loaded by Tauri
          (b) objc2 crate for direct Objective-C message passing
          (c) Shell out to `defaults` for KVS (hacky but works for
              simple reads/writes)
        Recommend (b) for KVS (small surface area) and rely on
        iCloud Drive's filesystem integration for project sync
        (just move the projects folder to the iCloud container —
        the OS handles the rest).
    Settings toggle:
      • iCloud sync should be opt-in with a toggle in Settings:
        "Sync projects and settings via iCloud" (default: off).
      • When enabled, migrate existing local projects to iCloud
        container. When disabled, migrate back to local.
      • Show sync status in Settings: "Last synced: 2 min ago" or
        "Syncing..." or "Offline — will sync when connected."
    Edition isolation:
      • Each edition has its own iCloud container (different bundle
        ID → different container). Rust Edition and Power Edition
        DON'T share projects via iCloud, same as local storage.
      • If the user wants cross-edition project access, that's a
        separate feature (local import/export, not iCloud).
    Privacy:
      • User's code lives in their own iCloud account. We never see
        it. No telemetry, no server-side access. Apple manages the
        encryption and access control.
      • Document clearly: "Your projects sync through YOUR iCloud
        account. We have no access to your code."
    NOT in v1 of iCloud support:
      • Real-time collaboration (two users editing same playground)
      • Cross-edition sync (Rust ↔ Power)
      • Selective sync (sync some projects but not others)
      • Version history beyond what iCloud Drive provides natively
      • iOS/iPadOS companion app (but iCloud container is ready for
        it if we ever build one)
    Priority: Medium. Valuable for multi-Mac users (the user's own
    workflow: MacBook + Mac Studio). Not critical for v0.3.4 launch.
    Revisit after Rust Edition ships and we see how many users have
    multi-Mac setups.
    Depends on: Apple Developer Program membership (for iCloud
    entitlements). Same dependency as signing/notarization — these
    will likely be acquired together.

  - FEATURE: Parallel playground runs within a single edition.
    User observation (2026-04-08): across editions the app happily runs two
    playgrounds in parallel (filesystem isolation + process isolation free
    from macOS), but within ONE edition we're single-run-at-a-time by
    choice. Ship this as a proper feature to match Unix/terminal mental
    models and VS Code-style multi-run workflows.
    Technical reality:
      • Cargo's lock is on the BUILD (per target dir), not on execution.
        Two already-built playground binaries can run in parallel just
        fine — the OS happily forks parallel processes of the same exec.
      • Playgrounds from DIFFERENT projects within an edition already
        have different target dirs, so even the builds could run in
        parallel without any lock contention.
      • Playgrounds from the SAME project share a target dir, so builds
        serialize on .cargo-lock — but execution is still parallel once
        they're built.
      • Nothing at the OS or Cargo layer prevents N-way parallelism.
    The blocker is UX. The current app has exactly one of each:
      1. Output panel — single Console view that streams stdout/stderr
      2. stdin input field — single text box for read_line() style input
      3. isRunning status flag — drives status bar, stop button, etc.
      4. Stop button — singular, no answer to "stop what?" with multiple
      5. Run block — one collapsible block per run in the timeline
    To support N-way parallel runs we'd need to pluralize all five:
      • Tabbed or split Console with a panel per active run
      • Per-run stdin inputs (the active tab's run gets keyboard focus)
      • A "Runs" list (sidebar section or tab bar) showing each active
        run with its lifecycle state (Saving/Compiling/Running)
      • Individual Stop buttons per run + a "Stop All" menu action
      • Status bar that shows either the active run OR a summary
        ("2 runs active") when multiple
    Design sketch:
      • Run list as a horizontal tab bar above the Output panel, similar
        to terminal emulators' tab bar or VS Code's debug sessions
      • Each tab shows playground name, running state icon, stop button
      • Click a tab → switches the Console view to that run's output
      • ⌘R starts a new run (doesn't kill existing ones); the
        stop-and-run confirmation dialog is removed because it's no
        longer needed
      • ⌘. stops the active-tab run; ⌘⇧. stops all
      • When all runs finish, tabs persist for history until user closes
        them or starts a new run that would reuse the slot
      • Maximum N (configurable, default 4?) to prevent runaway resource
        use — past the limit, the oldest run's slot gets reused
    Keyboard shortcuts to rework:
      • ⌘R — start new run (new tab)
      • ⌘. — stop active-tab run
      • ⌘⇧. — stop all runs
      • ⌘1..9 — switch to run tab N (like terminal emulators)
      • ⌘W in Run tab context — close that run tab (if finished)
    Book projects:
      • Book playgrounds can be run in parallel too — same mechanism.
    Settings:
      • "Maximum concurrent runs" setting (default 4, min 1, max 10)
      • "Auto-close finished runs after N seconds" setting (default off)
    Compared to existing tools:
      • Swift Playgrounds.app: one run at a time (same constraint as ours
        today). Swift Playgrounds users won't feel regressed.
      • VS Code: multi-run via Debug panel + tabs. Closest analog.
      • JetBrains IDEs: multi-run via Run tool window with per-run tabs.
      • Terminal emulators: trivially parallel (open another tab). Our
        target model for power-user workflows.
    Why NOT in v1 (v0.3.x / v0.4):
      • Major UX refactor — touches Output panel, status bar, run
        lifecycle, menu items, keyboard shortcuts. Weeks of work.
      • Beginners (the primary audience) typically run one thing at a
        time. Adding this feature risks making the app feel busier
        without meaningful benefit for them.
      • The Swift Playgrounds model (single run) is the mental model
        for the learning-tool positioning. Changing it pulls us toward
        "lightweight IDE" which is a different product.
      • Running Rust Edition + Power Edition side-by-side already gives
        you 2-way parallelism if you really need it — not elegant but
        zero additional work to use.
    When to build it:
      • After v0.4+ when the core feature set is stable.
      • If user feedback specifically asks for it (watch for "I want to
        compare outputs of two playgrounds side by side" type requests).
      • Could also be a "Power Edition exclusive" — Rust Edition stays
        beginner-simple, Power Edition gets the multi-run tab bar.
        Aligns with Power Edition's "for users who want more" positioning.
    Priority: LOW for now. Revisit based on feedback. Great candidate
    for a Power-Edition-only feature differentiation.

  - REFACTOR SPRINT: tech-debt cleanup after the v0.3.4 Rust Edition
    release ships. Not pre-release work — these don't block shipping, but
    should be done before the Power Edition ramp. Collect new items here
    as they're discovered.
    Items so far:
      • Parallelize check_toolchain subprocess spawns.
        Current: ~13 sequential Command::output() calls, ~150-400ms total
        on a fully-equipped dev machine. Modal open latency is noticeable.
        Approach: spawn all independent probes concurrently
        (Command::spawn → child.wait_with_output() collected via join),
        then assemble the ToolchainStatus from the results. Could cut
        total time ~5x since probes are I/O-bound and independent. Main
        complexity: some probes are conditional on others (e.g. rustup-
        show-active-toolchain needs rustup_installed). Careful ordering
        or a two-phase approach (phase 1: presence, phase 2: details)
        keeps correctness.
      • Consolidate clang/swift detection logic. Currently duplicated
        between cargo_commands.rs::check_toolchain and
        rustic_manifest.rs::detect_clang_version / detect_swift_toolchain.
        Same xcrun/clang/swiftc shim concerns apply to both sites.
        Extract a shared `macos_toolchain` module (or add to languages/
        mod.rs) so the CLT guard + version extraction lives in one place.
        Aligns with the "Promote Xcode CLT to a global prerequisite"
        refactor already on the roadmap — could be done together.
      • Code-split the Monaco editor bundle. Current vite build warning
        every time: "Some chunks are larger than 500 kB" — index-*.js is
        ~4 MB / ~1 MB gzipped, dominated by Monaco. Dynamic import()
        split would reduce initial load and let non-editor UI
        (Welcome Wizard, Settings) render before Monaco is ready. Follow
        vite's manualChunks guidance in rollupOptions.
      • Audit all "currently unused field in JSON response" type decls.
        We found installed_toolchains as dead code during this sprint;
        there may be others. Grep ToolchainStatus consumers vs the
        backend JSON keys and remove any drift.
      • sync-version.sh sed bug (noted in project_next_session memory).
        The Cargo.toml version replacement pattern fails silently and
        the version isn't actually bumped. Had to hand-edit during the
        v0.3.4 bump. Fix before next version bump.

      • Menus remain enabled when modal dialogs are open (Welcome
        Wizard, Settings, FixWizard, Help, About, etc.). User can
        trigger actions (New Project, Run, Export, etc.) through menus
        while a modal is up, causing state changes in the dimmed UI
        behind the modal. Observed: New Project created behind the
        Welcome Wizard.
        Root cause: menus and modals are two independent state systems.
        rebuild_menu() only considers playground/project state, not
        whether a modal is active. Tauri's menu system is global (always
        active) while modals are frontend-only (CSS overlay).
        Fix approaches:
          (a) Track a global `modalActive` state in App.svelte. When
              any modal is open, call rebuild_menu with all items
              disabled (or a minimal safe set: just Quit, Hide, About).
              On modal close, rebuild with normal state. Simple but
              adds a rebuild_menu round-trip per modal open/close.
          (b) Introduce an AppMode enum (Normal, Wizard, Settings,
              FixWizard, Help, About) that drives BOTH menu state and
              UI rendering. Menu items are enabled/disabled based on
              the current mode. More architectural but cleaner — each
              mode explicitly declares what actions are available.
          (c) Frontend-only: intercept menu events in App.svelte and
              silently drop them if any modal is open. Cheapest fix but
              feels hacky — menus visually look enabled but do nothing.
        Recommended: (b) for long-term, (a) as a quick v0.3.5 fix.
        The AppMode enum aligns with the "single AppState struct"
        suggestion from the code review. Both menus and UI would derive
        from one source of truth.
        Priority: Medium. Not a data-corruption risk (modals block the
        main editor, so file edits can't happen). But it's confusing UX
        and could cause unexpected state changes behind the modal.
        Discovered: 2026-04-09 during testing.

      • Book projects: two read-only bypass bugs (2026-04-09):
        1. Cargo.toml "Add Dependency" button is still enabled. The
           editor tab is correctly read-only, but the dependency
           manager toolbar action bypasses the check and writes to
           the book project's Cargo.toml.
        2. Content folder: "+ Add File" button is correctly disabled,
           but drag-and-drop from Finder still works — files can be
           dropped into a book project's content folder.
        Fix: gate add_dependency/remove_dependency + import_content_file
        backend commands on !is_book_project. Frontend: disable
        toolbar dep buttons AND reject drag-drop events when the
        active project is read-only.
        Priority: Low. Doesn't corrupt anything (book projects can
        be re-loaded to reset), but violates the "book projects are
        read-only" contract.

      • Copy to Project — two issues (2026-04-09):
        1. Only available for book project playgrounds. Should work for
           ANY playground → copy from any project to any other project.
           Use case: user has utility code in one project they want to
           reuse in another. Currently they have to manually copy the
           .rs file via Finder. The context menu action should appear
           for all playgrounds (book AND user), with a target project
           picker that excludes the source project.
        2. When no editable Rust project exists (e.g. only book projects
           loaded), "Copy to Project" is still enabled in the context
           menu. Clicking it shows an error: "No Rust project available,
           create one first." Harmless but confusing — the menu item
           should be disabled (greyed out) when there are no valid
           target projects. Check: filter projects by (a) is user
           project (not book/readonly), (b) is same language type as
           source, (c) is not the source project itself. If the
           filtered list is empty, disable the action.
        Priority: Low. Not a data issue — just UX polish. Fix together
        in v0.3.5 or the next feature release.

      • ARCHITECTURE: State-driven UI refactor.
        Current state: UI behavior is spread across dozens of
        independent $state variables, ad-hoc `if` checks, and
        implicit assumptions. Examples of what breaks:
          - Menus stay enabled when modals are open (logged above)
          - Book project read-only bypassed by dep manager + drag-drop
          - Pill status computed separately from wizard status
          - Menu rebuild doesn't know about modal state
          - Editor read-only check is separate from toolbar button
            enable/disable logic
        Root cause: no single source of truth for "what can the user
        do right now." Each component independently decides its own
        enabled/disabled state based on partial information.
        Proposed fix: introduce a centralized UI state model that
        ALL components derive from. Two levels:
          (a) AppMode enum — what "screen" is active:
              Normal, Wizard, Settings, FixWizard, Help, About,
              NewPlayground, CopyToProject, Export
              → drives: which menus are enabled, which shortcuts work,
              whether backdrop blocks interaction
          (b) ProjectContext struct — what the active project allows:
              { isBookProject, isReadOnly, isLocked, hasPlayground,
                hasActiveTab, isDirty, toolchainState, projectType }
              → drives: which toolbar buttons are enabled, which
              context menu items appear, whether drag-drop is accepted,
              whether dep manager works, whether save/run/export are
              available
        Implementation:
          - AppMode lives in App.svelte as a single $state
          - ProjectContext is $derived from existing state variables
          - Every component reads from these two sources instead of
            checking raw booleans independently
          - Menu rebuild takes AppMode + ProjectContext as inputs
          - Toolbar, sidebar, editor all derive enabled states from
            ProjectContext
          - Modal open/close transitions update AppMode atomically
        Benefits:
          - One place to audit "what can the user do in state X"
          - New features automatically get correct enable/disable
            behavior by declaring their AppMode requirements
          - Menu-modal sync fixed as a side effect
          - Book project read-only enforced uniformly
          - Easier to test (check state → check derived permissions)
        This is the single most impactful refactor for long-term
        maintainability. Every UI bug we logged in this session
        (menu-modal, book deps, book drag-drop, copy-to-project
        enable state) traces back to the same root cause: no
        centralized state model.
        Priority: HIGH for refactor sprint. Do before Power Edition.
        Discuss design in detail before implementing — this touches
        every component.

      • DMG volume icon: when the DMG is mounted, the Desktop volume
        icon shows the same app icon as the installed .app. Standard
        macOS convention is to show a disk/external-drive icon with
        the app icon overlaid. Fix: post-build script that converts
        DMG to read/write, mounts it, copies a custom .VolumeIcon.icns
        to the mount root, sets the custom icon flag via SetFile -a C,
        unmounts, and converts back to compressed/read-only. ~15 lines
        of bash. Low priority — cosmetic polish only.
        Discovered: 2026-04-10.

    Code review findings (specs/code-review-v0.3.4.md, 2026-04-09):
    Top 5 by impact — address before Power Edition ships:
      • Fix unbounded output buffer in stream_pipe
        (playground_commands.rs:516-561). A playground that outputs a
        single huge line with no newline grows memory without bound →
        OOM. Add MAX_LINE_LEN (~1 MB) and truncate/emit partial lines.
      • Fix missing source path validation in import_content_file
        (content_commands.rs:75-117). Source path is never checked for
        traversal or symlinks. User could import /etc/passwd into a
        project. Validate source, reject symlinks, optionally restrict
        to safe directories.
      • Fix unsafe unwrap on child process pipes
        (playground_commands.rs:484-485, 657). child.stdout.take() and
        child.stderr.take() use .unwrap() which panics if pipes are
        None (edge case during process termination race). Replace with
        .ok_or("Failed to capture stdout/stderr")?.
      • Add timeout to cargo check (playground_commands.rs:635-656).
        Live check runs with no timeout — on slow machines or large
        projects, the process could hang indefinitely. Wrap in
        tokio::time::timeout(Duration::from_secs(30), child.wait()).
      • Fix race in check_playground process cancellation
        (playground_commands.rs:618-625). Killing a previous cargo
        check by PID can hit a reused PID if the process already exited
        naturally. Use a generation counter or verify the process is
        still alive before sending SIGTERM.

    Additional review items (lower severity, address opportunistically):
      • Race condition: mutex lock held across await boundary
        (playground_commands.rs:482). Move sync lock outside async path.
      • Shell pattern in rustup install (lib.rs:601-604). sh -c with
        piped curl is unsafe pattern even with hardcoded URL. Consider
        download-then-execute.
      • No cleanup on failed export (export.rs:437-523). Partial files
        left on disk. Use temp dir + atomic rename.
      • No concurrent edit handling — last write wins silently. Add
        optimistic locking before multi-window support ships.
      • Zombie process potential (playground_commands.rs:563-581). Call
        .wait() after SIGKILL to reap children.
      • Dead parameters in build_menu (menu.rs:23, 26). Remove
        _playground_count and _project_type from signature.
      • No config caching (lib.rs:135-149). load_config() re-reads
        disk every call. Cache in app state.
      • Hardcoded Rust edition "2024" (lib.rs:167-171). Make
        configurable or detect from rustc.
      See specs/code-review-v0.3.4.md for full details on all 25 items.

  - FEATURE: Expand Run/Stop toolbar into Check / Run / Test / Format / Lint.
    User request (2026-04-08). Currently the only editor action is Run
    (which is cargo run: build + execute). Users want more granular
    actions for the common dev loop.
    Proposed actions:
      • Check     (⌘B)    — cargo check, validates compile without running
      • Run       (⌘R)    — current behavior (cargo run), unchanged
      • Test      (⌘U)    — cargo test --bin <name>, runs #[test] fns
      • Stop      (⌘.)    — current behavior, kills whatever is running
      • Format    (⌘⇧F)   — rustfmt on current file
      • Clippy    (⌘⇧K)   — cargo clippy, stricter lint pass
      • Build Release     — cargo build --release (no execution)
    Toolbar UX:
      • Keep Save, Run, Stop as primary buttons — don't clutter.
      • Add a small dropdown arrow next to the Run button (▾) that
        reveals Check / Run / Test as sibling actions.
      • Format / Clippy / Build Release live in a new "Build" top-level
        menu (alongside Project / Playground / Run / Edit), OR extend
        the existing Run menu with all of them under clear separators.
      • Match Xcode's mental model: Build (⌘B), Run (⌘R), Test (⌘U)
        are distinct, separate keyboard shortcuts.
    Format specifically:
      • Pipe current editor content through rustfmt via stdin → stdout,
        replace editor content with formatted result. Avoids save-reload
        dance and keeps Monaco's undo stack intact (user can ⌘Z if they
        don't like it).
      • Add "Format on save" setting — default off for v1 so it's not
        surprising. When on, ⌘S runs rustfmt before writing to disk.
      • For C/C++, use clang-format if available (ships with Xcode CLT).
      • For Zig, use `zig fmt` (built-in).
      • For Swift, swift-format isn't default — skip for v1 or document
        as "not supported for Swift yet".
    Clippy specifically:
      • `cargo clippy --bin <name> -- -W clippy::all`
      • Output streams to console like a normal run.
      • Warnings/errors get parsed as diagnostics markers in the editor
        (reuse the existing live-check markers infrastructure).
      • Possibly merge with live-check: if clippy is installed, live
        check already uses it via cargo check + clippy lints. The
        explicit Clippy action gives a stricter pass (e.g. -D warnings).
    Test specifically:
      • `cargo test --bin <name>`, streams to console.
      • For v1, just show raw output. Future: parse the test summary
        ("running N tests ... test result: ok. X passed; Y failed")
        and show a nice pass/fail badge in the status bar.
      • If the playground has no #[test] functions, show a toast
        "No tests found in this playground" instead of silent success.
    Language matrix (what each language supports in v1):
      • Rust   — Check, Run, Test, Format, Clippy, Build Release (full)
      • C/C++  — Check (parse-only), Run, Format (clang-format), Build Release
      • Zig    — Check, Run, Test, Format, Build Release
      • Swift  — Check, Run, Build Release (no fmt, no clippy-equivalent)
      Menu items dynamically enable/disable based on the active project's
      language — reuse the existing Lang enum dispatch pattern.
    Backend mechanics:
      • New Tauri commands: check_playground, format_playground,
        clippy_playground, test_playground, build_release_playground.
        Each takes project + playground name + streams output via Channel.
      • Reuse stream_pipe infrastructure from playground_commands.rs.
      • Format is special: returns the formatted source as a String
        response, not a streaming command. Frontend replaces editor
        content atomically.
    Status bar during long actions:
      • Show the action name during runs:
        "Checking…" / "Running…" / "Testing…" / "Linting…" / "Formatting…"
      • Reuse the existing Saving → Compiling → Running lifecycle pattern.
    Book projects:
      • Check + Run only. Format, Test, Clippy, Build Release all touch
        files or produce artifacts — keep disabled for book projects.
    Scope for v1:
      • Just Check + Format + Clippy + Test for Rust, plus the toolbar
        dropdown and keyboard shortcuts. Build Release can wait for the
        Download Executable feature (which already needs a release build).
      • Other languages: subset per the matrix above, designed later.

  - FEATURE: Download compiled executable (debug or release).
    User request (2026-04-08). Complements "Export Project" which ships
    source + Cargo.toml. Users who wrote a useful CLI tool want to share
    the working binary with someone who doesn't have Rust installed.
    UX:
      • Two new menu items under Project (alongside "Export Project"):
         - "Download Executable (Debug)"     — fast build, larger binary
         - "Download Executable (Release)"   — slow build, optimized
      • Alternative: replace the single "Export Project…" item with an
        "Export ▸" submenu containing all three options. Less menu
        clutter, more discoverable as a family.
      • Status bar / console feedback during build (reuse run-lifecycle
        Compiling → Done machinery — same as a normal ⌘R run).
      • On success, show a toast or small success dialog with:
         - Where the file was saved
         - Short note that the binary is unsigned → first run on a
           receiver's Mac needs right-click → Open to bypass Gatekeeper
         - "Reveal in Finder" button to pop the folder open
      • Toolbar Export button could become a dropdown with all three
        options (v2 concern — don't add UI clutter in v1).
    Backend mechanics:
      • New Tauri command: `build_playground_binary(project, playground,
        mode: "debug" | "release") -> BinaryPath`
      • Run `cargo build --bin <playground>` or `cargo build --release
        --bin <playground>` with the same target dir used by playground
        runs (target/playground-runs/ — avoids lock contention with
        cargo check).
      • Stream compile output via Tauri Channel to the console panel,
        same pattern as the existing run command.
      • On success, locate the binary at
        `target/playground-runs/{debug,release}/<playground>`
        and copy it to the final destination.
      • On failure, return the compile error; don't copy anything.
    Destination path (confirmed 2026-04-08):
      • Default: ~/Downloads/<project_name>/<playground_name>
      • Creates the ~/Downloads/<project_name>/ subfolder if it doesn't
        exist. Each subsequent download adds another binary to that
        folder — clean per-project separation, easy to find.
      • No file extension on macOS — just the playground name (matches
        how cargo produces binaries).
      • Prefill a save-file dialog with this default so the user can
        override if they want (pick a different location, rename, etc.).
      • If the same filename already exists, the dialog handles overwrite
        confirmation (Tauri dialog plugin default behavior).
      • Project directories stay source-only — no build artifacts leak
        into them outside the existing target/ dir.
    Content files warning:
      • Playgrounds that read PLAYGROUND_CONTENT won't have that env var
        set when run standalone from the downloaded binary. Pre-build
        check: if the playground source contains "PLAYGROUND_CONTENT",
        show a warning dialog: "This playground reads from the content
        folder. The downloaded binary won't have access to those files
        unless you set PLAYGROUND_CONTENT manually when running it."
      • Give the user a "Continue" / "Cancel" choice.
    Binary metadata to surface in success dialog:
      • Architecture (aarch64 / x86_64 — use `file` or mach-o header)
      • File size
      • Mode (debug/release)
      • Example of how to run it: `./<playground_name>` from Terminal
    Gatekeeper note in the success dialog (critical for UX):
      • "On another Mac, right-click → Open the first time to bypass
        Gatekeeper. Or run: xattr -dr com.apple.quarantine <path>"
      • Without this note, recipients will think the binary is broken.
    Menu wiring:
      • src-tauri/src/menu.rs — add the two items under Project menu.
      • Frontend menu handlers in App.svelte.
      • Keyboard shortcuts: skip for v1 (Export has ⌘⇧E already; don't
        overload).
    NOT in v1 (explicit exclusions):
      • Cross-compile to Linux/Windows (needs rustup targets + linker
        setup — separate feature).
      • Code signing (needs Apple Developer ID + signing identity).
      • Stripping symbols further than cargo does by default.
      • Bundling content files alongside the binary.
      • Packaging as .app or .pkg installer.
    Power Edition consideration:
      • C/C++: same concept, compile with clang to an executable, same
        download flow. Language modules already handle compilation.
      • Swift: same, swiftc produces executables.
      • Zig: same, zig produces executables.
      • All four languages should support this feature — don't build
        it Rust-only. The language registry (languages/mod.rs) already
        knows how to build each type; this feature just needs a
        `build_binary` dispatch method per language.

  - FEATURE: Rust Modules tab — shared code across playgrounds in a project.
    User request (2026-04-08). Currently every playground is a standalone
    binary in src/bin/ with no way to share code — users who want a helper
    function across multiple playgrounds have to copy-paste it. Modules fix
    this.
    UX concept:
      • New "Modules" section in the sidebar, parallel to "Content".
      • Click "+ Add Module" → name input → creates src/modules/<name>.rs
        with a starter template (a `pub fn` and a doc comment).
      • Modules are editable in Monaco like playgrounds; same rename/delete
        context menu actions.
      • Scope is global within a project: every playground has access to
        every module. No per-module visibility config — keep it simple.
      • Playgrounds import via `use <pkg>::modules::<name>::*;` where
        <pkg> is the Cargo.toml [package] name (stable per-project).
    Backend mechanics:
      • Physical layout: src/modules/<name>.rs for each module, plus an
        auto-generated src/modules/mod.rs that re-exports all of them
        (`pub mod foo; pub mod bar;`).
      • src/lib.rs auto-generated with `pub mod modules;` so the library
        crate surfaces the modules tree. Auto-ensure [lib] block exists
        in Cargo.toml with path = "src/lib.rs".
      • First module creation triggers the lib/mod.rs scaffolding; delete
        of the last module should leave the lib intact but warn the user
        (don't silently remove scaffolding — might break existing use
        statements in their playgrounds).
      • mod.rs is regenerated on every add/rename/delete — never
        hand-edited. Treat it as derived state, like Cargo.lock.
    Tauri commands needed:
      • list_modules(project) -> Vec<ModuleInfo>
      • create_module(project, name, template_slug)
      • read_module(project, name) -> String
      • save_module(project, name, contents)
      • rename_module(project, old, new) — must also update any use
        statements in playgrounds? (see "Rename gotcha" below)
      • delete_module(project, name) — regenerates mod.rs
    Rename gotcha:
      • If user renames a module, playgrounds referencing it break.
        Options: (a) auto-rewrite use statements across all playgrounds
        (complex, error-prone, touches files the user didn't expect);
        (b) warn the user on rename and let them fix manually; (c) block
        rename if any playground references it (too restrictive).
        Lean (b) — clear warning dialog, user decides.
    Crate-name stability:
      • Playgrounds use `<pkg_name>::modules::` which bakes the Cargo
        package name into imports. Renaming the PROJECT (which renames
        the Cargo package name) breaks all module refs.
      • Mitigation: either (1) use a fixed internal crate name like
        `playground_modules` regardless of project name (hide the rename
        from Rust), or (2) warn on project rename and auto-rewrite. Lean
        (1) — more robust, hides implementation detail from users.
    Export implications:
      • When exporting a playground, the exporter must also include the
        modules it references (or all of them — simpler). Update
        scripts/export.rs Rust export path to walk src/modules/ and
        include every .rs file.
      • Exported README should document the module structure.
    Live checking:
      • cargo check already covers the whole package (lib + all bins),
        so errors in modules show inline via existing diagnostics.
      • No new machinery needed on the diagnostics side.
    Book projects:
      • Book projects are read-only by design — disable "+ Add Module"
        and make any existing modules read-only.
      • Copy-to-Project for a book playground should also copy any
        modules it references (diff/ask user if modules would collide
        with existing ones in the target project).
    Sidebar iconography:
      • Distinct file-badge icon for modules (vs playgrounds and content
        files). Cargo orange tint? Or a `fn{}` style glyph?
    Templates for new modules:
      • "Empty module" — just `pub fn hello() -> &'static str`
      • "Data model" — pub struct with derive Debug/Clone + impl block
      • "Error helper" — custom Result + Error enum scaffold
      • "Common utilities" — file I/O wrappers, env helpers
      • "Unit tests only" — #[cfg(test)] mod tests scaffold
    Interactions to think about:
      • What if a playground has `fn main()` and a module has `fn main()`?
        The lib's main would be shadowed — not a real issue since lib
        crates don't have main. Harmless.
      • What if user tries to create a module named "main" or "lib"?
        Blocklist these at the Tauri command level.
      • What about cyclic module deps? Rust handles this at the crate
        level, should be fine.
    Scope for v1 of this feature:
      • Just the mechanics above. No per-module docs, no module discovery
        from crates.io, no inter-module visibility config. Keep it tight.
      • Power Edition: Zig, Swift, and C/C++ have their own module/header
        concepts — this feature is Rust-only for v1, design separately
        for other languages later.

  - REFACTOR: Promote Xcode CLT to a global prerequisite (must do before
    Power Edition / Phase 2 ships).
    Why: CLT is a platform prerequisite for nearly all native languages,
    not just Rust. clang IS Xcode CLT, swiftc IS Xcode CLT, Rust's linker
    (cc) + macOS SDK come from CLT. Zig is the only LARGELY-independent
    language: it ships its own LLVM compiler, its own lld linker, and
    bundled macOS SDK TBD stubs that let trivial programs link libSystem
    without CLT. But anything that imports Apple frameworks
    (CoreFoundation, AppKit, etc.) or hits non-trivial system features
    will need real SDK headers from CLT. So Zig has a SOFT dependency
    where the others have HARD dependencies.
    Currently we model CLT as a Rust-specific cascade state (rust_state =
    'clt_missing'), which is convenient for the Rust Edition but
    architecturally wrong — when Power Edition ships with all four
    languages, the Rust-centric framing won't fit. A user with broken CLT
    on a C/C++ project should see the same Install button, same FixWizard,
    same auto-polling experience.

    Hard vs soft block matrix:
      Rust   → HARD block (no link, no compile output)
      Clang  → HARD block (clang IS CLT — without it, no compiler at all)
      Swift  → HARD block (swiftc IS CLT — without it, no compiler at all)
      Zig    → SOFT warning ("install CLT if you hit linker errors with
                Apple framework imports"); pill stays yellow not red;
                trivial programs still work.
    Refactor scope:
      Backend (cargo_commands.rs)
      • Lift xcode_clt_installed out of rust_state into its own top-level
        status field (`xcode_clt: { installed, path }` already exists; just
        stop letting rust_state collapse into 'clt_missing').
      • rust_state cascade goes back to 4 values: not_installed → no_default
        → missing_components → healthy.
      • clang and swift toolchain checks should report a "blocked by CLT"
        sub-state when CLT is absent (instead of just "not found"). For
        clang/swiftc, CLT *is* the compiler — these will already fail to
        find the binary without CLT.
      • Zig: surface CLT as a SOFT recommendation. Zig itself works
        without CLT for trivial programs (bundled SDK stubs cover
        libSystem), but warn that Apple framework imports require CLT.
        Pill goes yellow, not red. Don't block runs.
      Backend (lib.rs)
      • InstallXcodeCLT FixAction stays as-is (already language-agnostic).
      Frontend (ToolchainFixWizard.svelte)
      • Treat CLT as a separate top-of-cascade check rendered ABOVE the
        per-language status. Possibly a dedicated "Prerequisites" status
        card at the top of the modal that shows when CLT is missing AND
        any selected language depends on it.
      • The split-layout (Help Me Install / I'll Do It Myself) stays the
        same — manual mode shows xcode-select --install as step 1 for any
        affected language; guided mode shows the same Install Xcode CLT
        button regardless of which language led the user here.
      Frontend (ToolchainWizard.svelte)
      • The toolchain step's status grid should show CLT as a top row
        BEFORE the per-language sections, with a single "Install Xcode CLT"
        action that benefits all languages at once.
      • Each language section can still show its own status, but CLT
        becomes a shared prerequisite badge.
      Frontend (App.svelte)
      • pillStatus for rust/clang/swift considers CLT a HARD block (red).
      • pillStatus for zig considers CLT a SOFT warning (yellow) — only
        when CLT is also missing. If Zig is healthy and CLT is present,
        pill stays green. If Zig is healthy but CLT is missing, pill is
        yellow with tooltip explaining the framework-import limitation.
      • pillText for HARD CLT-blocked state: "Xcode CLT required".
      • pillText for SOFT CLT-warning Zig state: "{zig version} · CLT
        recommended" or similar.
      Naming
      • Consider renaming ToolchainFixWizard.svelte to something
        language-agnostic (PrereqFixWizard? ToolchainSetupModal?) since it
        will no longer be Rust-specific.
      • The Help → Rust Help → Rust Toolchain… menu item stays Rust-named,
        but the modal it opens is shared infrastructure.
    Test plan: Round 2 BRANCH 0 currently lives under "Rust Toolchain Wizard"
    — should be re-categorized as a global prereq test. Add equivalent
    BRANCH 0 tests for clang and swift (vanilla VM, no CLT, click pill,
    same modal opens).

v0.3.2 — Welcome Wizard + Language Gating
  Status: complete — released 2026-04-04

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

v0.3.3 — Edition Builds
  Status: complete — released 2026-04-06

  Overview:
  Ship two editions: Rust Edition first (focused Rust learning tool), then
  Power Edition a few weeks later (all 4 languages). Same codebase, different
  configs. Tauri --config overrides handle app name/identifier/icon;
  VITE_EDITION env var controls frontend behavior via editions.ts registry.
  C/Zig/Swift single-language editions remain in the registry for future use
  but are not being distributed now.

  Release plan:
  - Phase 1: Rust Edition — "Rustic Playground — The Rust Edition"
    Sharpest pitch: one language, one audience (Rust learners).
    Website + GitHub Releases DMG.
  - Phase 2: Power Edition — "Rustic Playground" (a few weeks later)
    Full multi-language experience for users who want all 4 languages.

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

  10. Saved snapshots (.saved/) with revert and undo-revert (App.svelte revertCache)
  11. Menu overhaul: per-project Learn menu, Check for Updates, dynamic product name
  12. Run lifecycle status: Saving → Compiling → Running stages in console + status bar
  13. Close-dirty-tab dialog (Save / Don't Save / Cancel)
  14. Book chapter "Read Online" links (Rust Book, Swift Book) in status bar
  15. Status bar below Monaco editor (VS Code style) — status + chapter link
  16. Toolbar reorganization: toolchain info left, settings icon right, no center
  17. Dynamic window title: App — Project — Playground
  18. Minimum window width bumped 800→900px
  19. Update checker via GitHub Releases API (startup banner + menu item)
  20. Official language logos (SVG) replacing text badges across all components
      (Sidebar, ProjectSwitcher, ToolchainWizard, toolbar — cargo logo for Rust toolchain)
  21. Book auto-select when enabling a language, reactive book/books labels
  22. Test log: 233 test cases across 42+ sections

  Known Issues:
  - Enter key does not trigger primary action in Settings modal or confirm dialogs
    (Escape works; Enter works in NewPlaygroundModal and CopyToProjectModal).
    Probable cause: Enter was wired to apply() which applies but doesn't close —
    may just need to wire to done/cancel instead. Track via GitHub issue.
  - ⌘⇧/ (Help) shortcut doesn't trigger — macOS intercepts it for system Help Search.
    May need a different accelerator or workaround.
  - Cargo.toml: "Add Dependency" button doesn't focus crate name input after click.
    Also needs: crate search (crates.io API), full CRUD (add/delete/update),
    validate crate name + version against crates.io before adding.
  - About modal icon shows macOS padding (black corners around squircle).
    Source images have padding baked in — need unpadded artwork or CSS crop.

  Post v0.3.3 Ideas (now backlog for v0.3.4 / v0.4):
  - "Build failed" status bar should show the line number where the build failed
  - App quit with dirty tabs: prompt "Save & Exit / Discard & Exit / Cancel" listing dirty playgrounds
  - (done) Lock icon moved to status bar below editor
  - Status bar contextual labels: "Read Only", "Editing Locked", file type, line/col position, etc. Lock icon: red tint when locked, green when editable.
  - New Project type selector: expand to fill tab bar width, no text wrapping for "Zig (0.15)", move Zig to end
  - Automated UI testing (e.g. Playwright/WebdriverIO + Tauri driver) to reduce manual regression testing
  - Unit test playground template (Rust #[test] scaffold)
  - Enable RUST_BACKTRACE for detailed panic traces in console output
  - Embedded real terminal/console (like VS Code integrated terminal) replacing current output panel
  - Run timeout guard: auto-kill playgrounds after configurable duration (e.g. 60s) to catch infinite loops
  - Console: show elapsed run time (ms/s) alongside timestamp when run completes
  - Book chapter URLs: fetch from rust-lang/book SUMMARY.md at build time or validate on app startup to handle future book restructures
  - (done) Rust Book 2024 Edition update: added ch17 async/await, renumbered ch17-20→ch18-21, 21 chapters total
  - Refactor: Book chapters driven by data files (JSON/TOML) instead of hardcoded Rust source (~2700 lines per book)
  - Refactor: Templates driven by data files instead of hardcoded TypeScript arrays
  - Refactor: HelpModal content driven by data files instead of inline HTML
  - Sidebar: sort playgrounds by creation order (or configurable) instead of alphabetical, so book chapters display in sequence
  - Toolchain installer/repair flow: Wizard guides new users through step-by-step
    rustup install when no toolchain is detected. Settings detects broken toolchains
    (e.g. rustup present but no default channel set, missing components) and offers
    a guided fix — "Run `rustup default stable`", "Install missing component X",
    etc. — instead of just showing "not found".


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
IDEA: Linux Port — Rust Edition (deprioritized)
─────────────────────────────────────────────────────────────────────────────
Status: Parked — low ROI for target audience
Logged: 2026-04-04

Background
  Same Tauri/Svelte/Monaco stack, Rust Edition only. Packaged as .deb, .rpm,
  .AppImage. GNOME HIG compliance (header bar, flat buttons, Adwaita palette,
  platform-conditional CSS). Test targets: Ubuntu 22.04+, Fedora 38+.

Why It's Parked
  Target audience is beginners learning Rust. Linux users who've set up a dev
  environment are comfortable with cargo + their preferred IDE. The overlap of
  "wants to learn Rust" + "on Linux" + "would benefit from a playground over
  CLI" is small. Engineering effort doesn't justify the reach.

Trigger Condition
  Revisit if there's clear demand (workshop/university lab use cases, user
  requests) or if the Windows port ships and Linux becomes low-hanging fruit.

─────────────────────────────────────────────────────────────────────────────
IDEA: Windows Port — Rust Edition (deprioritized)
─────────────────────────────────────────────────────────────────────────────
Status: Parked — high value but high effort
Logged: 2026-04-04

Background
  Same Tauri/Svelte/Monaco stack, Rust Edition only. Tauri uses WebView2 (Edge)
  on Windows. Packaged as .msi and/or .exe installer. Windows has the largest
  population of beginner Rust learners (students, bootcamp attendees).

  Windows-specific work:
  - MSVC vs GNU toolchain detection and guidance
  - Windows path handling (backslashes, %USERPROFILE%, Program Files spaces)
  - WebView2 runtime dependency (may need bundling or install prompt)
  - Keyboard shortcuts: Ctrl+R/S/N (Tauri remaps automatically)
  - Installer: .msi via Tauri bundler (WiX) or NSIS
  - Windows Defender / SmartScreen code signing considerations
  - Storage: %APPDATA%/com.rustic-playground.rust/

Why It's Parked
  Higher reach than Linux (most beginner Rust learners are on Windows), but
  Windows brings significant platform pain: MSVC toolchain setup, path issues,
  WebView2 dependency, code signing for SmartScreen. Want to avoid this
  complexity until the macOS version is fully polished and distributed.

Trigger Condition
  Revisit after website + DMG distribution are live and macOS version is stable.
  If user demand is strong, Windows port has higher priority than Linux.

─────────────────────────────────────────────────────────────────────────────
IDEA: TUI Edition — CLI playground with embedded editor
─────────────────────────────────────────────────────────────────────────────
Status: Parked — scope after GUI distribution is live
Logged: 2026-04-04

Background
  A terminal-based Rustic Playground: single Rust binary, runs everywhere.
  Install via `cargo install rustic-playground`. Split-pane TUI with file tree,
  embedded editor, and streaming output — like the GUI app but in terminal.
  Inspired by lazygit, helix, and Claude Code.

Proposed Stack
  - ratatui + crossterm for TUI framework
  - tui-textarea or custom editor widget with tree-sitter syntax highlighting
  - Direct function calls to existing language modules (no Tauri IPC layer)
  - Same project structure and data directories as GUI edition
  - Same backend: languages/, playground_commands, cargo_commands, templates,
    book data — ~80% of src-tauri/src/ reused directly

  ┌─────────────────────────────────────────────────────────┐
  │  TUI App (ratatui)                                      │
  │                                                         │
  │  ┌──────────┐  ┌──────────────────┐  ┌──────────────┐  │
  │  │ Sidebar  │  │ Editor           │  │ Output       │  │
  │  │ projects │  │ tree-sitter      │  │ streaming    │  │
  │  │ files    │  │ highlighting     │  │ stdout/err   │  │
  │  │          │  │                  │  │ run status   │  │
  │  └──────────┘  └──────────────────┘  └──────────────┘  │
  │                                                         │
  │  Status bar: project name, language, run state          │
  └─────────────────────────────────────────────────────────┘

What You'd Reuse vs Rewrite
  Reuse: all language modules, project CRUD, config/settings, templates,
  book chapter data, export logic, toolchain detection. Backend is done.
  Rewrite: UI layer (ratatui replaces Svelte), command dispatch (direct
  Rust calls replace Tauri IPC — actually simpler). No Tauri dependency.

Strengths
  - `cargo install` — one command, works on macOS/Linux/Windows/WSL/SSH
  - Zero GUI dependencies — no WebView, no platform-specific code
  - Naturally reaches Linux/Windows users who live in the terminal
  - Complements the GUI: same project format, portable between editions
  - Smaller binary, faster startup, lower resource usage

Hard Parts
  - Editor quality gap: tui-textarea is functional but far from Monaco.
    Needs tree-sitter integration for syntax highlighting. No autocomplete,
    no inline error squiggles (would need diagnostics pane instead).
  - "Easier than terminal" pitch weakens — this IS a terminal app. Value
    shifts from "no terminal needed" to "managed workflow" (lazygit model).
  - Keyboard conflicts with terminal emulators (Ctrl+S, Ctrl+R, etc.)
  - Live error checking: cargo check results shown as diagnostics list
    rather than inline markers. Different UX, still useful.

Why It's Parked
  The GUI app isn't distributed yet. Shipping a second edition before the
  first one reaches users splits focus. The backend reuse makes this a
  medium-effort project, not a large one — worth doing, but after the macOS
  GUI is live and feedback is rolling in.

Trigger Condition
  Revisit after website + DMG distribution. If cross-platform demand is the
  main feedback theme, the TUI edition solves it more efficiently than
  porting the GUI to Linux and Windows separately. Could also serve as the
  foundation for a `rustic-playground` crate on crates.io.

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
IDEA: Multi-Version Toolchain Picker
─────────────────────────────────────────────────────────────────────────────
Status: Parked — future feature
Logged: 2026-04-15

Background
  The toolchain pill in the toolbar currently shows the installed rustc version
  and health status (green/yellow/red). This pill could become a version picker:
  click it to switch between installed Rust toolchains (stable, beta, nightly,
  or pinned versions like 1.85.0).

  rustup already manages multiple toolchains:
  - `rustup toolchain list` → enumerate installed versions
  - `rustup run <toolchain> cargo run ...` → run with a specific toolchain
  - `rustup toolchain install <version>` → add a new version

  The version picker extends this to a UI-first experience: the pill shows the
  active toolchain, clicking opens a dropdown of installed toolchains, selecting
  one switches the project's active toolchain. New toolchains can be installed
  from the same dropdown.

Proposed UX
  1. Click the toolchain pill → dropdown appears
  2. Dropdown shows installed toolchains with versions (e.g., stable 1.87.0,
     nightly 2026-04-10, beta 1.88.0-beta.1)
  3. Active toolchain has a checkmark
  4. Click a different toolchain → switches immediately, pill updates
  5. "Install Toolchain..." row at bottom → opens a dialog to install
     stable/beta/nightly or a specific version
  6. Scope: per-project — stored in project config (Cargo.toml metadata
     or rust-toolchain.toml). Mirrors how real Rust projects pin toolchains.
     Teaching this pattern is part of the "learn the toolchain" philosophy.

Backend
  - New command: list_rust_toolchains → parses `rustup toolchain list`
  - New command: set_active_toolchain → updates project/app config
  - Modify run_playground to use `rustup run <toolchain> cargo run ...`
    instead of bare `cargo run` when a non-default toolchain is selected
  - New command: install_toolchain → `rustup toolchain install <version>`,
    streaming output via existing Channel infra

Why It's Valuable
  - Beginners: test code against stable vs nightly when The Book or a
    crate requires it
  - Advanced: verify code compiles on MSRV (minimum supported Rust version)
  - Educators: demonstrate edition differences (2021 vs 2024)
  - Natural extension of the existing pill — no new UI chrome needed

Why It's Parked
  Current version gate (v0.3.5) only enforces a minimum version. Multi-version
  support is a different tier of complexity: per-project config, toolchain
  state management, UI for install/switch. Ship the launch first, revisit
  when there's user demand or when the toolchain pill UX feels limiting.

Trigger Condition
  User feedback requesting nightly/beta support, or educator use cases where
  version pinning matters.

─────────────────────────────────────────────────────────────────────────────
IDEA: Native Embeddable Code Editor Crate
─────────────────────────────────────────────────────────────────────────────
Status: Parked — separate project, long-term
Logged: 2026-04-15

Background
  No production-quality embeddable code editor component exists in Rust.
  Every Rust desktop app that needs code editing either embeds Monaco via
  a webview or builds from scratch. Xi (Google-backed) was abandoned.
  Zed built one but welded it to GPUI. Nobody extracted the reusable widget.

  The gap: a Rust crate that provides a code editor component with
  tree-sitter syntax highlighting, rope buffer, and LSP client support —
  platform-agnostic, renderable via any native backend.

Proposed Shape
  - Rust crate (library, not an app)
  - Core: rope buffer + tree-sitter highlighting + LSP client
  - Rendering trait: crate emits styled frames (spans, cursors, diagnostics,
    line numbers); consumer renders via their framework (GPUI, SwiftUI,
    iced, egui, wgpu)
  - First consumer: Rustic Playground v2.0 replaces Monaco with it

Research Before Starting
  - COSMIC / System76: `cosmic-text` is a text layout engine. Check if
    COSMIC's editor app extracted a reusable widget with tree-sitter/LSP.
  - Lapce: Rust editor — check if their editor component is extractable
  - Zed GPUI: check latest state of text input primitives

Why It's Parked
  Separate project from Rustic Playground, multi-year scope. Revisit after
  launch traction is established. Could be the project that gets rust-lang's
  serious attention more than the playground app alone.

─────────────────────────────────────────────────────────────────────────────
IDEA: Cloud-Substrate Playground (Native Apps + Remote Containers)
─────────────────────────────────────────────────────────────────────────────
Status: Parked — possible Phase 3 evolution, contingent on Phase 2 traction
Logged: 2026-04-19

Background
  Market positioning between two existing product classes:
    - Pure web playgrounds (play.rust-lang.org, old repl.it) — too thin,
      ephemeral, single-file, can't persist real work.
    - Full cloud IDEs (Replit, Codespaces, Gitpod) — drifted into
      full-workstation territory, lost the "playground" discipline.
  Neither holds the middle ground. Replit started as cloud REPL and got
  pulled toward full IDE by user demand (files → folders → git → SSH →
  deploy). Staying a playground is a harder product discipline than
  becoming an IDE.

  Gap: native app per platform, cloud as shared substrate. Analogous to
  Dropbox, 1Password, Notion, Things, iA Writer — "native everywhere, cloud
  is the sync layer." These win vs web competitors on UX because they can
  use platform-specific affordances (keyboard shortcuts, gestures, menu
  bar integrations, system share sheets, offline mode) that browser tabs
  can't reach.

  Apple's containerization framework is macOS-only and won't come to
  iPadOS (JIT restrictions + app sandbox model). So local containers are
  macOS-only structurally. Cloud containers remove that constraint and
  open the door to iPad, Windows, Linux, even web-as-viewer.

Proposed Shape
  - Native frontend per platform (macOS first, iPad next, others later)
  - Projects + user data persist in cloud (syncs across devices)
  - Compile/run proxied to cloud containers (Firecracker or similar)
  - Blank-container model preserves "learn the toolchain" angle — user
    still runs rustup/cargo/etc. inside a fresh Linux env; not hidden,
    just relocated. Ephemeral by default, persistent opt-in.
  - Each platform gets a tailored native UX (⌘R on Mac, iPad gestures,
    etc.) rather than one browser-based UI stretched everywhere.
  - Web presence is view-only / sharing ("open this playground link on
    any device"), not a full web app — that would collide with
    play.rust-lang.org's space.

What It's NOT
  - Not a Replit clone. Stays playground, not workstation.
  - Not a pure web product. Native-first is the differentiator.
  - Not phase 2. Phase 2 (per project_strategy_2026) is native + local
    containers + multi-language on macOS. This is phase 3: extend that
    to other platforms via cloud.
  - Not polyglot from day one. Systems-languages focus likely fits the
    brand better than "every language."

Sequencing (if traction proves out)
  1. Phase 1 — ship Rustic Playground macOS (current). Build taste +
     brand + user base. Validate desire for native-feel playgrounds.
  2. Phase 2 — native rewrite + local containerization on macOS.
     Monetization introduced. Still single-platform.
  3. Phase 3a — add cloud backend as opt-in. Existing Mac app gets
     "Rustic Cloud" toggle. Sync projects, remote compile option.
     Works fully offline if disabled.
  4. Phase 3b — iPad app, SwiftUI-shared with macOS. Same cloud backend.
  5. Phase 3c — web viewer. "View shared playgrounds in any browser."
     Read-only or very limited edit. Defensive, not primary.
  6. Phase 3d (maybe never) — Windows/Linux. Tiny learning-audience,
     big engineering burden; defer or skip.

Why It's Parked
  Entirely contingent on Phase 1 + 2 traction. Strong risk of
  pre-optimizing for platform expansion that may not be worth it.
  Capital-intensive (container compute = real money; abuse prevention
  is a full-time concern). Fundamentally a different business than the
  current free-open-source desktop app — requires SaaS ops, billing,
  support, potentially team scale.

  Not worth building toward architecturally right now. Even building
  the macOS app as if this will happen would invite scope creep. Keep
  the current product focused; re-evaluate this entry after Phase 1
  launch lands and real user feedback tells us whether iPad / other
  platforms are a genuine top-of-mind ask.

Decision Trigger
  Revisit when all three are true:
    - Phase 1 traction established (stars, active users, organic
      community growth, not just launch spike).
    - Phase 2 shipped and monetized (proves people pay for polish).
    - Repeated user feedback asking for iPad / cross-platform access.
  If any of those is missing, this stays parked.

─────────────────────────────────────────────────────────────────────────────
IDEA: Notebook-Style Editor — interleaved prose + code cells
─────────────────────────────────────────────────────────────────────────────
Status: Parked — post-launch evaluation
Logged: 2026-04-20

Background
  A Jupyter-style editing mode where a playground is a sequence of cells:
  markdown/prose cells for explanations and code cells that execute. Prose
  cells could auto-convert to Rust comments on save so the file remains a
  valid `.rs` source. Target use cases: Rust Book chapters, tutorials,
  teaching sessions, self-documenting experiments.

Proposed Direction: Two Complementary Features (B + C)
  Rather than one big rewrite, do two smaller, orthogonal pieces:

  B. Inline prose blocks inside `.rs` playgrounds
     Effort: ~few days.
     Scope: existing Rust (and other) playground project types.
     - Keep single source file and Monaco. Render `/*md ... */` or `//!`
       blocks as inline prose via Monaco view zones / decorations.
     - No changes to compile pipeline, file format, error mapping, or
       exports. Fully backward compatible.
     - Source file stays valid Rust — can still `cargo run` and export
       like today. Teaching polish without changing the model.
     - Delivers ~70% of the teaching feel for ~10% of the cost.
     - Tradeoff: prose and code share one text buffer — less clean than
       true cells, but honest about what the underlying file is.

  C. "Rust Notebook" — a new project type, not a replacement
     Effort: ~2–4 weeks once prioritized.
     Scope: a distinct project type alongside Rust/Clang/Zig/Swift in the
     `Lang` enum. New entry in the project switcher: "New Rust Notebook".
     - Backed by evcxr as the Rust Jupyter kernel — true cell execution
       with persistent state between cells.
     - File format: one `.ipynb`-style document per notebook (or a
       bespoke Rustic format that evcxr can read). NOT a `.rs` file.
     - UI: real cell-based editor (cell list, per-cell Monaco, markdown
       cells rendered, run-cell affordance, per-cell output).
     - Dependencies: evcxr's `:dep` magic. Distinct from Cargo manifest.
     - Export: notebook → `.ipynb` (Jupyter-compatible), or notebook →
       flattened `.rs` for people who want to graduate to a playground.
     - Sits next to regular playgrounds, doesn't replace them. Users
       choose the right tool: `.rs` playground for building something,
       notebook for exploring / teaching / stepping through.

  Rejected: Option A — true notebook semantics forced onto `.rs` files
     Concatenation-into-`fn main()` + rustc-line-remapping is a lot of
     infrastructure to build a worse version of what evcxr already does.
     If we want real cells, use evcxr (Option C). If we want teaching
     polish in regular playgrounds, do Option B. No middle ground worth
     the engineering.

Why It's Parked
  Launch-week-sensitive. Current priority is shipping v0.3.6 and running
  the staggered community launch (r/learnrust Tue, r/tauri Thu, r/rust
  next Mon, Show HN Tue). Editor work mid-launch is the wrong shape of
  risk.

  Beyond launch timing: validate demand first. Plenty of teaching
  playgrounds today get by with rich `//` comments — HelpModal + Rust
  Book chapter playgrounds already read well without cell structure.

Trigger Conditions (independent for each piece)
  Option B — inline prose blocks:
    Revisit when a book chapter or tutorial playground starts feeling
    cramped inside plain `//` comments, or multiple users ask for
    "richer explanations alongside the code." Low-risk, reversible,
    ship when the itch shows up.

  Option C — Rust Notebook project type:
    Higher bar. Revisit only when BOTH are true:
      - Multiple independent users request true notebook-style
        exploration (not a one-off ask).
      - Option B has shipped and confirmed that inline prose isn't
        sufficient for the use case — i.e. users specifically want
        per-cell execution and persistent state, not just better
        prose rendering.
    evcxr integration is enough work that it needs real demand behind it.

─────────────────────────────────────────────────────────────────────────────
