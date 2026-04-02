# Rustic Playground

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
>
> **No binary is distributed.** You must compile this yourself from source.
> If you received a pre-built binary from an untrusted source, do not run it.
>
> **You are responsible for the code you run.** This tool is for learning
> and experimentation in a controlled environment you own.

---

A macOS desktop app for running Rust experiments — inspired by Swift Playgrounds.
Write code, press **⌘R**, see output stream live. No terminal required.

Built with [Tauri 2](https://tauri.app) + [Svelte 5](https://svelte.dev) +
[CodeMirror 6](https://codemirror.net).

## Features

- **Live execution** — ⌘R compiles and runs; stdout/stderr streams in real time
- **Multiple projects** — each project is an isolated Cargo workspace with its own `Cargo.toml`
- **Multiple playgrounds** — every `.rs` file in `src/bin/` is its own runnable binary
- **Content files** — attach any file to a project via the Files panel; access at runtime via the `PLAYGROUND_CONTENT` env var
- **Cargo.toml editor** — edit dependencies directly in the app
- **Window state persistence** — layout, panel sizes, open tabs, and window size survive restarts
- **Rust Book examples** — load all 20 chapters of _The Rust Programming Language_ as ready-to-run playgrounds (**Help → Load Rust Book Examples…**)

## Requirements

| Tool | Version |
|---|---|
| macOS | 13 Ventura or later |
| Rust toolchain | stable — install via [rustup.rs](https://rustup.rs) |
| Node.js | 18+ |
| pnpm | 8+ |
| Tauri CLI | `cargo install tauri-cli --version "^2.0"` |

## Build & Run

```sh
git clone https://github.com/jagmeetchawla/playground-rs
cd playground-rs
cd ui && pnpm install && cd ..
cargo tauri dev        # development mode — hot reload
cargo tauri build      # release .app in src-tauri/target/release/bundle/
```

## How It Works

Each **project** is a Cargo workspace stored at:

```
~/Library/Application Support/com.playground-rs.app/projects/<name>/
├── Cargo.toml        ← shared dependencies for all playgrounds in this project
├── src/bin/
│   ├── hello.rs      ← seeded on first launch
│   └── <name>.rs     ← one file per playground
└── content/          ← runtime assets (accessible via PLAYGROUND_CONTENT)
```

Each `.rs` file is a standalone binary target with a `fn main()`.
The backend runs `cargo run --bin <name>` and streams stdout/stderr live.

## Content Files

Each project has a **content folder** — the same concept as Swift Playgrounds' assets bundle.
Drop any file there and access it from your playground code:

```rust
use std::{env, fs};

fn main() {
    let dir = env::var("PLAYGROUND_CONTENT").unwrap_or_default();
    let data = fs::read_to_string(format!("{dir}/data.csv")).unwrap();
    println!("{data}");
}
```

| Action | How |
|---|---|
| View a project's files | Click `▸` next to the project name in the sidebar |
| Add a new file | Click `[+ Add File]` in the expanded files section |
| Import from Finder | Drag a file onto the project row in the sidebar |
| Edit a text file | Click it — opens as an editor tab |
| Open a binary / image | Click it — opens with the default macOS app |
| Rename / Delete / Reveal | Right-click the file |

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| ⌘R | Run the active playground |
| ⌘. | Stop the running process |
| ⌘S | Save the active file |
| ⌘N | New playground |
| ⌘W | Close active tab |
| ⌘⇧N | New project |
| ⌘⇧/ | Help |

## Adding Dependencies

All playgrounds in a project share one `Cargo.toml`. Click the **Cargo.toml** entry
at the bottom of the sidebar to edit it directly:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
rand  = "0.8"
tokio = { version = "1", features = ["full"] }
```

## Rust Book Examples

**Help → Load Rust Book Examples…** creates one project per chapter of
[_The Rust Programming Language_](https://doc.rust-lang.org/book/),
all ready to run with ⌘R.

| # | Project | Approach |
|---|---|---|
| 1 | `ch01_getting_started` | hello world, format strings, variables |
| 2 | `ch02_guessing_game` | rand, Ordering, match, loop |
| 3 | `ch03_concepts` | data types, functions, control flow |
| 4 | `ch04_ownership` | move semantics, references, slices |
| 5 | `ch05_structs` | structs, methods, Display |
| 6 | `ch06_enums` | enums, match, if let |
| 7 | `ch07_modules` | inline modules, paths, use — plus CLI guide for file-based modules |
| 8 | `ch08_collections` | Vec, String, HashMap |
| 9 | `ch09_errors` | panic!, Result, ? operator |
| 10 | `ch10_generics` | generics, traits, lifetimes |
| 11 | `ch11_testing` | #[test], assert macros — plus `cargo test` instructions |
| 12 | `ch12_minigrep` | working grep using the Files panel, plus CLI guide |
| 13 | `ch13_closures` | closures, iterators, combinators |
| 14 | `ch14_cargo` | cfg!, build metadata — plus workspace/publish CLI guide |
| 15 | `ch15_smart_pointers` | Box, Rc, RefCell |
| 16 | `ch16_concurrency` | threads, channels, Arc/Mutex |
| 17 | `ch17_oop` | encapsulation, trait objects, state pattern |
| 18 | `ch18_patterns` | pattern syntax, match guards, @ bindings |
| 19 | `ch19_advanced` | unsafe Rust, advanced traits, macros |
| 20 | `ch20_web_server` | thread pool (fully runnable) — plus web server CLI guide |

Chapters that require a multi-file project or command-line arguments include a
`cli_guide.rs` playground that prints step-by-step terminal instructions when run.
Every chapter project also contains an `attribution.md` in its Files panel.

> **Attribution** — Examples are based on the curriculum of _The Rust Programming Language_
> by Steve Klabnik and Carol Nichols ([source](https://github.com/rust-lang/book),
> MIT / Apache-2.0, © Rust Project Developers 2010).
> Playground code is original educational Rust.
> Rustic Playground is not affiliated with or endorsed by the Rust Project.

## Project Structure

```
playground-rs/
├── src-tauri/
│   ├── src/lib.rs            ← all Tauri commands (projects, playgrounds, content files, seeding)
│   ├── capabilities/         ← Tauri 2 permission definitions
│   └── tauri.conf.json
└── ui/
    └── src/
        ├── App.svelte        ← main layout, state, event wiring
        └── lib/
            ├── Sidebar.svelte
            ├── Editor.svelte
            ├── Output.svelte
            ├── HelpModal.svelte
            └── AboutModal.svelte
```

## Security Model

See the warning at the top of this file. Additionally:

- Playground names are validated as `[a-z][a-z0-9_]*` — path traversal is blocked at the API layer
- The Tauri IPC bridge only accepts calls from the app's own WebView origin
- `cargo` is invoked via its absolute path (`~/.cargo/bin/cargo`), not via shell string interpolation
- Playground runs use a separate `target/playground-runs/` directory to avoid lock conflicts

## Release History

| Version | Highlights |
|---|---|
| v0.1.6.2 | Window state persistence (layout, panel sizes, tabs, window size) |
| v0.1.6.1 | Fix menu enabled/disabled sync for Delete Project / Delete Playground |
| v0.1.6 | Help modal, About modal, app icon, rename to Rustic Playground |
| v0.1.5 | Content files panel, drag-and-drop import, binary file support |
| v0.1.4 | Project management (new, rename, delete, duplicate, switch) |
| v0.1.3 | Cargo.toml editor tab |
| v0.1.2 | Tab bar, multiple open files, unsaved-change indicators |
| v0.1.1 | Sidebar, playground CRUD, live output streaming |
| v0.1.0 | Initial release |

## License

MIT
