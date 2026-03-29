CONVENTIONS

Naming
- Playground files: src/bin/<name>.rs — short, lowercase, no suffix (not <name>_playground.rs)
- Names should reflect what is being explored, not which chapter or source it came from (preferred over chapter3, chapter4 etc.)

Code Style
- Keep code idiomatic and explicit.
- Prefer clear names over short names.
- Add brief comments only where behaviour is non-obvious.
- Each playground is fully self-contained — no shared modules between playgrounds.
- Entry point is always fn main(), never pub fn run().

Dependencies
- Add to Cargo.toml only when needed by a playground.
- All playgrounds share the same dependency set — keep it lean.

Formatting and Lints
- All code must pass cargo fmt and cargo clippy -- -D warnings before committing.
