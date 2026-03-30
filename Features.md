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
