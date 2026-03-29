ARCHITECTURE

Approach
- Each playground is a standalone binary in src/bin/<name>.rs with a fn main() entry point.
- Cargo auto-discovers every .rs file in src/bin/ as a binary target — no registration, no codegen.
- The runner (src/main.rs) is a thin CLI that lists, picks interactively, and delegates to `cargo run --bin <name>`.
- Dependencies are shared across all playgrounds via a single Cargo.toml.

Project Structure
- src/main.rs              — the playground runner (clap CLI: list, interactive pick, run)
- src/bin/<name>.rs        — one file per playground, each fully self-contained
- specs/                   — project specs and docs
- specs/archive/         — archived past specs
- Cargo.toml               — shared dependencies for all playgrounds

Runner CLI
- `cargo run`              — interactive: lists playgrounds, prompts to pick one
- `cargo run <name>`       — runs a named playground directly
- `cargo run list`         — prints all available playgrounds (alias: ls)
- `cargo run -- --version` — prints version

Why src/bin/ over the previous approach
- The previous approach (v0) used build.rs to scan src/ for *_playground.rs files, generated
  src/_playgrounds.rs with a declarative macro, and dispatched via pub fn run().
- This required: build.rs + include!() + a custom macro + the paste crate — four moving parts
  for something Cargo already handles natively.
- The src/bin/ approach removes all custom machinery. Adding a playground is one file.

Adding a Playground
- Create src/bin/<name>.rs with fn main()
- Add any dependencies to Cargo.toml
- Run immediately with: cargo run <name>
