ARCHITECTURE

---

v1.0 — Tauri Desktop App (current)

Stack
- Tauri 2.0       — app shell, Rust backend, macOS WKWebView
- Svelte + Vite   — frontend UI framework (lean, no virtual DOM)
- Monaco Editor   — code editor (same engine as VS Code)
- Rust backend    — existing runner logic exposed as Tauri commands

Project Structure

playground-rs/
├── src/
│   ├── main.rs              — Tauri app entry point (replaces CLI runner)
│   └── bin/                 — playgrounds (unchanged)
│       ├── hello.rs
│       ├── chapter3.rs
│       └── ...
├── ui/                      — frontend (React + Monaco)
│   ├── src/
│   │   ├── App.tsx          — root layout (sidebar + editor + output)
│   │   ├── Sidebar.tsx      — playground list, new button
│   │   ├── Editor.tsx       — Monaco wrapper
│   │   └── Output.tsx       — output panel, streaming
│   ├── index.html
│   └── package.json
├── specs/
├── Cargo.toml               — Tauri + shared playground deps
├── tauri.conf.json          — Tauri app config
└── README.md

How it works

1. App launches → frontend loads in WKWebView
2. Sidebar reads playground list via Tauri command → list_playgrounds()
3. User selects playground → load_playground(name) returns file contents → Monaco loads it
4. User edits → changes held in editor state
5. User hits Run (or Cmd+R) →
   a. save_playground(name, content) writes back to src/bin/<name>.rs
   b. run_playground(name) spawns cargo run --bin <name> as a child process
   c. stdout/stderr streamed line-by-line via Tauri events → output panel
6. Status indicator updated throughout: idle → compiling → running → done/error

Toolchain Detection & Setup
- On launch: check common paths in order:
    1. App config (previously saved path)
    2. ~/.cargo/bin/cargo
    3. which cargo (PATH lookup)
    4. /usr/local/bin/cargo, /opt/homebrew/bin/cargo
- If found: read version via cargo --version, store path in Tauri app config
- If not found: show first-run wizard
- Install path: rustup-init downloaded from static.rust-lang.org, run with -y --no-modify-path
- All cargo invocations use the stored absolute path — never rely on PATH alone

Tauri Commands (Rust → Frontend)
- detect_toolchain()              → Option<ToolchainInfo { path, version }>
- install_rustup()                → streams events: install_progress, install_complete
- set_toolchain_path(path)        → Result<ToolchainInfo>
- list_playgrounds()              → Vec<String>
- load_playground(name)           → String (file contents)
- save_playground(name, content)  → Result
- run_playground(name)            → streams events: output_line, run_complete
- check_playground(name, content) → streams events: check_diagnostic, check_complete
- new_playground(name)            → Result (creates src/bin/<name>.rs with template)

Live Check Pipeline
1. User types → frontend debounces 500ms
2. save_playground(name, content) — writes current editor content to disk
3. check_playground(name) — spawns cargo check --bin <name> --message-format json
4. Parses JSON diagnostic output (file, line, col, message, severity)
5. Emits check_diagnostic events → frontend converts to Monaco editor markers
6. Monaco renders squiggles + hover tooltips inline

Output Streaming
- stdout lines  → output_line event { stream: "stdout", line: "..." }
- stderr lines  → output_line event { stream: "stderr", line: "..." }
- compiler errors → output_line event { stream: "error", line: "..." }
- process exit  → run_complete event { code: i32 }
- Frontend renders each stream with a different colour: white / red / amber

---

v0 — CLI Runner (archived, see specs/archive/)

Approach
- Each playground is a standalone binary in src/bin/<name>.rs with a fn main() entry point.
- Cargo auto-discovers every .rs file in src/bin/ as a binary target.
- The runner (src/main.rs) is a thin clap CLI: list, interactive pick, run.
- cargo run <name> delegates to cargo run --bin <name>.

Why replaced
- CLI requires a terminal — adds friction for casual use.
- No editor, no output history, no persistent state.
- The Tauri app preserves the same src/bin/ playground model underneath — only the
  interface changes.

---

Pre-v0 — build.rs + macro approach (see playground-rs-alt/)

The original implementation used build.rs to scan src/ for *_playground.rs files,
generated _playgrounds.rs with a declarative macro, and dispatched via pub fn run().
Required: build.rs + include!() + custom macro + paste crate.
Replaced because Cargo's src/bin/ auto-discovery handles everything natively.
