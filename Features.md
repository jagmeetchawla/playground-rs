# Features & Future Enhancements

Tracked ideas — ranging from near-term polish to longer-term architectural shifts.

---

## Near-term (v1.x)

### Stop button — actually kill the process
The Stop button currently just resets the status label. It needs to send `SIGKILL` (or `SIGTERM` + timeout) to the `cargo run` child process. Requires storing the `Child` handle in the Tauri backend and exposing a `kill_playground` command.

### Live error checking (`cargo check` squiggles)
Run `cargo check` in the background (debounced ~500ms after the user stops typing) and surface compiler errors as Monaco editor markers — red squiggles with hover messages. This is what RustRover/rust-analyzer does. Requires a separate `--target-dir` to avoid lock conflicts with active runs.

### Toolchain setup wizard
On first launch, detect whether `rustup` / `cargo` is installed. If not, show a one-time setup screen that offers to download and install rustup automatically, or lets the user point to an existing installation. Saves the resolved `cargo` path in app settings.

### Settings panel
- Editor font size and font family
- Tab size (2 vs 4 spaces)
- Editor theme (light / dark / system)
- Cargo path override (for non-standard toolchain locations)
- Preferred Rust edition

### Window state persistence
Remember window size, position, sidebar width, and which tabs were open between launches. Store in App Support via a small JSON settings file.

### Resizable panels
Let the user drag the sidebar and output panel borders to resize them. The editor should take all remaining space.

---

## Medium-term (v2.x)

### Multiple files / modules per playground
Right now each playground is a single `.rs` file in `src/bin/`. Allow a playground to be a mini-Cargo project (its own `Cargo.toml`, `src/` folder) so users can split code across modules and add dependencies.

### Dependency management UI
A simple panel to add crates from crates.io to a playground's `Cargo.toml` — search, add, remove — without touching the file manually.

### Playground sharing / export
Export a playground as a Gist or a link to the official Rust Playground (play.rust-lang.org). One-button share.

### Stdin support
Some programs read from stdin. Add an input field in the output panel that pipes into the running process.

---

## Long-term / Architectural (v3.x)

### Native container execution via `apple/containerization` *(macOS 26+)*

**Background (discussed March 2026):** macOS 26 ships Apple's native container framework — two open-source repos:
- [`apple/containerization`](https://github.com/apple/containerization) — embeddable **Swift package** (the engine)
- [`apple/container`](https://github.com/apple/container) — CLI tool built on top of it

This is not Docker. Each container runs in its own lightweight Linux VM via `Virtualization.framework`, booting in under a second, optimised for Apple Silicon.

**Why this matters for us:**

| Goal | How containers solve it |
|---|---|
| **Safety** | User code runs in an isolated VM — cannot touch the host filesystem, network, or processes |
| **Controlled stack** | We ship (or download) a pinned Rust toolchain image; no dependency on the host's `rustup` |
| **App Store distribution** | `Virtualization.framework` has App Store-compatible entitlements; main app stays fully sandboxed |
| **Toolchain versions** | Trivially offer stable / beta / nightly by switching container images |

**Proposed architecture:**
```
Tauri Rust backend
  → invoke Swift plugin (Tauri 2.0 supports Swift plugins)
      → apple/containerization Swift package
          → lightweight Linux VM (< 1 sec cold boot)
              → Alpine Linux + Rust toolchain image
                  → compile & run user code
          → stream stdout / stderr back via Tauri Channel API
  → Output panel
```

**Key decisions to make when we pick this up:**
1. **Bundle vs download** — Don't bundle the image in the `.app` (too large, ~400–600 MB). Download on first launch and cache in App Support. Model: how Xcode downloads simulator runtimes.
2. **Warm container** — Pre-boot the container in the background when the app launches; keep it idle between runs. Eliminates the cold-start wait for the user.
3. **macOS 26 minimum** — Either set macOS 26 as the new minimum, or maintain the current non-sandboxed path for macOS 15 and below as a fallback.
4. **App Store submission** — With the container path in place, this becomes a realistic target. The main app is fully sandboxed; execution happens inside the VM.

**References:**
- [Meet Containerization — WWDC25](https://developer.apple.com/videos/play/wwdc2025/346/)
- [apple/containerization on GitHub](https://github.com/apple/containerization)
- [apple/container on GitHub](https://github.com/apple/container)
- [Technical comparison with Docker — The New Stack](https://thenewstack.io/apple-containers-on-macos-a-technical-comparison-with-docker/)

---

### Full native rewrite — Swift + SwiftUI

**The idea:** Once the container backend is solid and the feature set is stable, rewrite the entire app in Swift and SwiftUI — no Tauri, no web stack, no JavaScript.

**Why it makes sense at that point:**
- The Tauri + Svelte approach was the right call to move fast and iterate. The web stack made the editor (Monaco), streaming output, and complex UI state easy to prototype.
- But Swift/SwiftUI is the natural home for a Mac-first developer tool. Native rendering, native animations, native controls — no WebView bridge, no JS runtime overhead.
- The container backend (`apple/containerization`) is already a Swift package. A native app talks to it directly with no FFI layer in between.
- SwiftUI's `NavigationSplitView` maps directly to our three-panel layout (sidebar / editor / console). The tab model maps to `TabView` or a custom `HStack` of tab buttons — identical concept, native implementation.
- A fully native app is smaller, faster to launch, uses less RAM, and passes App Store review more cleanly.

**What carries over:**
- All the product thinking, UX decisions, keyboard shortcuts, and feature set
- The Rust backend logic (playground CRUD, `cargo run` streaming, path validation) — this moves to Swift, calling the same underlying `cargo` binary or the container runtime
- The overall three-panel layout and interaction model

**What gets replaced:**
- Tauri → pure SwiftUI app target
- Monaco Editor → [CodeEditKit](https://github.com/CodeEditApp/CodeEditKit) or a custom `NSTextView`-backed editor with Tree-sitter for Rust syntax
- Svelte components → SwiftUI views
- TypeScript state management → `@Observable` / `@State` / SwiftData

**When to consider this:**
When the container path (above) is working and the v2.x feature set is complete. At that point the product is proven, the scope is known, and a native rewrite is a well-scoped project rather than a moving target.

---

## Separate project — AI session knowledge graph

**Discussed:** 2026-03-30, during playground-rs session

A structured graph to track work done across AI chat sessions — nodes for decisions,
features, files, sessions, motivations; edges for `motivated_by`, `implemented_in`,
`replaced_by`, `depends_on`. Solves the context loss problem when sessions expire.

Key insight from discussion: today's AI memory tools (ChatGPT memory, Cursor rules,
project knowledge) are all "store text, retrieve text." A graph is model-oriented —
the graph is the source of truth, documents are projections of it. A PRD generator
asks *"what should we write down?"* A knowledge graph asks *"what do we actually
know, and how does it connect?"*

AI-specific angle: instead of pasting a 3,000-word summary to resume a session, an
AI agent queries the subgraph relevant to the current task — precise, structured,
minimal context. Population must be automatic (AI writes it during sessions) or it
won't survive contact with real use.

Connects to: craft.ai (previous project by Jagmeet — document-oriented PRD/spec
generation). This idea is the evolution: from document-oriented to model-oriented.

**Tracking:** continuing in a separate project/chat.

---

## v4.0 — Multi-language support (systems languages without good playgrounds)

**The vision:** Expand beyond Rust to cover the languages that have no decent interactive playground or notebook — the ones where the only option today is "open a terminal, create a file, compile, run, repeat."

Python has Jupyter. JavaScript has every browser devtools and countless REPLs. Swift has Swift Playgrounds. Rust will have us. But a whole tier of systems-level languages are completely underserved.

### Target languages

| Language | Current state | Why it belongs here |
|---|---|---|
| **C** | Nothing. `cat > foo.c && clang foo.c && ./a.out` | The foundation of computing, taught everywhere, zero interactive tooling |
| **C++** | Compiler Explorer (web-only, assembly-focused) | Same gap as C; the language has grown enormously but playground tooling hasn't |
| **Zig** | Basic web playground only | New systems language with a lot of momentum; the tooling ecosystem is still young |
| **Ada** | Essentially nothing | Safety-critical systems language; used in aerospace/defence; completely ignored by the tooling world |
| **Fortran** | Nothing modern | Still actively used in scientific computing (climate models, physics simulations); Fortran 2023 is a real language |
| **D** | Nothing desktop | Statically typed, GC optional, fast — deserves better |
| **Nim** | Nothing desktop | Compiles to C, interesting language, small community, no playground |
| **Assembly (ARM/x86)** | Compiler Explorer (web) | Learning assembly on Apple Silicon with live output would be genuinely useful |

### Why the container backend makes this tractable

Without containers, adding each language means: detect the compiler on the host, handle different install paths, manage `PATH`, deal with missing toolchains. It's a support nightmare.

With the `apple/containerization` backend (v3.x), adding a language is:
1. Build or pull a container image that has the compiler installed
2. Mount the source file
3. Run the compile + execute command
4. Stream output back

The app's core plumbing — tab management, output panel, file storage, Channel streaming — stays identical. Each language is just a different image and a different build command.

### Language-specific considerations

**C / C++**
- Compiler: `clang` / `clang++` (ships with Apple Command Line Tools; also in the container image)
- Single-file model works well for a playground
- Live error checking via `libclang` — this is how most C/C++ IDEs do it
- File extensions: `.c` / `.cpp` — the tab badge changes from `RS` to `C` / `C++`

**Zig**
- `zig run file.zig` — single command, no separate compile step
- Excellent error messages; live checking via `zig ast-check`
- The Zig toolchain is self-contained and easy to install in a container

**Ada**
- `gnat` (GCC Ada frontend) in the container
- Verbose syntax but highly structured — actually well suited to a playground that shows you compiler output
- Badge: `ADA`

**Fortran**
- `gfortran` in the container
- Target scientific computing learners who are used to notebooks (Jupyter + Fortran kernel exists but is painful to set up)
- Badge: `F90` or `F`

### UI changes needed

- Language selector per playground (set at creation time, changeable)
- Tab badge reflects the language (`RS`, `C`, `C++`, `ZIG`, `ADA`, `F`)
- Monaco already has syntax highlighting for C, C++, and most of these — just set the `language` option
- File extension stored with the playground metadata
- Build command and container image resolved from a language config table

### App name / positioning

At v4.0 the app is no longer a Rust playground — it's a **systems language playground**. The name and branding should reflect that. Working title: **Systems Playground** or just **Playground** (reclaiming the generic name because nothing else does this well).

### What this is not

This is deliberately **not** trying to cover Python, JavaScript, Ruby, or any language that already has excellent interactive tooling. The focus is the gap: compiled, systems-level languages where the feedback loop today is entirely manual.
