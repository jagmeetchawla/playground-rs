# playground-rs

> [!WARNING]
> ## ⚠️ DEVELOPER TOOL — NOT SANDBOXED — USE AT YOUR OWN RISK
>
> **This application is intentionally NOT sandboxed.**
>
> Like Xcode, VS Code, and Terminal, it must run outside macOS's App Sandbox
> because it compiles and executes arbitrary Rust code using your local
> `cargo` and `rustc` toolchain.
>
> **This means:**
> - Any code you write and run has **full access to your filesystem, network,
>   processes, and environment** — the same as code you'd run in Terminal
> - There is **no isolation** between playground code and your system
> - A playground that deletes files, exfiltrates data, or forks a bomb will
>   actually do those things
> - The app stores your playgrounds in
>   `~/Library/Application Support/com.playground-rs.app/` — writable by the app
>
> **No binary is distributed.** You must compile this yourself from source.
> If you received a pre-built binary from an untrusted source, do not run it.
>
> **You are responsible for the code you run.** This tool is for learning
> and experimentation in a controlled environment you own.

---

# playground-rs

A macOS desktop app for running Rust experiments — inspired by Swift Playgrounds.
Drop any `.rs` file into the workspace, write code, hit **Run**, see output.

Built with [Tauri 2](https://tauri.app) + [Svelte 5](https://svelte.dev) +
[Monaco Editor](https://microsoft.github.io/monaco-editor/) (the VS Code editor engine).

## Requirements

- macOS 11+
- [Rust + Cargo](https://rustup.rs) — the app will offer to install this on first launch if missing
- [Node.js](https://nodejs.org) 18+ and npm — to build the frontend
- [Tauri CLI](https://tauri.app/start/): `cargo install tauri-cli --version "^2.0"`

## Build & run

```sh
git clone https://github.com/jagmeetchawla/playground-rs
cd playground-rs
cargo tauri dev        # development mode — hot reload
cargo tauri build      # production .app + .dmg in src-tauri/target/release/bundle/
```

## How it works

The app manages a self-contained Cargo workspace at:
```
~/Library/Application Support/com.playground-rs.app/workspace/
├── Cargo.toml        ← shared dependencies for all playgrounds
└── src/bin/
    ├── hello.rs      ← seeded on first launch
    └── <your_file>.rs
```

Each `.rs` file is a standalone Cargo binary target with a `fn main()`.
The Tauri backend runs `cargo run --bin <name>` and streams stdout/stderr
live to the output panel.

## GUI usage

| Action | How |
|--------|-----|
| Run playground | `▶ Run` button or `Cmd+R` |
| Stop | `■ Stop` button or `Cmd+.` |
| Save | `Cmd+S` |
| New playground | `+` in sidebar or `Cmd+N` |
| Rename / Delete / Duplicate | Right-click playground in sidebar |

## CLI usage (no GUI)

The original CLI runner still works:

```sh
cargo run hello           # run a playground
cargo run list            # list all playgrounds
cargo run                 # interactive picker
```

## Adding dependencies

Edit `Cargo.toml` in the workspace directory — all playgrounds share it:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
rand = "0.8"
```

## Project structure

```
playground-rs/
├── src/
│   ├── main.rs           ← CLI runner (still works)
│   └── bin/              ← dev-mode playgrounds (used by cargo tauri dev)
├── src-tauri/            ← Tauri backend (Rust)
│   ├── src/lib.rs        ← commands: list, load, save, run, new, rename…
│   ├── tauri.conf.json
│   └── entitlements.plist
├── ui/                   ← Svelte 5 frontend
│   └── src/
│       ├── App.svelte
│       ├── lib/Sidebar.svelte
│       ├── lib/Editor.svelte   ← Monaco, language: rust
│       └── lib/Output.svelte
└── specs/                ← architecture, conventions, acceptance criteria
```

## Security model

See the warning at the top of this file. In addition:

- All playground names are validated as `[a-z][a-z0-9_]*` before any
  filesystem or process operation — path traversal is blocked at the API layer
- The Tauri IPC bridge only accepts calls from the app's own WebView origin
- `cargo` is invoked via its absolute path (`~/.cargo/bin/cargo`), not via
  shell string interpolation
- Playground runs use a separate `--target-dir` to avoid lock conflicts with
  the Tauri dev build

## License

MIT
