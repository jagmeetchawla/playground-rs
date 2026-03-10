SPECIFICATION V0
Status
- Version: v0
- Date: 2026-03-10
- Author: Jagmeet Chawla (jagmeet.chawla@gmail.com) with help from gpt-5.3-codex
- Source of truth at time of archive: codebase state + chat decisions
- Please note that initial codebase (v0) was built by Jagmeet Chawla with help from claude code
Product
What
- Provide a Rust playground runner where each playground is a module file in `src/` named `<name>_playground.rs`.
- Support two core CLI modes:
- `cargo run` lists available playground module names.
- `cargo run -- <module>` executes `<module>_playground::run()`.
- Keep adding a new playground low-friction by requiring only one file with `pub fn run()`.
- Establish modular specification management:
- Active specification in `specs/specifications.md`.
- Archived completed specifications in `specs/completed/` with versioned filenames.
Why
- Reduce boilerplate and manual wiring when experimenting in Rust.
- Preserve compile-time safety while approximating automatic module discovery.
- Keep implementation work aligned to explicit specs and avoid drift by making exclusions and acceptance criteria explicit.
Architecture
How
- `build.rs` scans `src/` for files ending in `_playground.rs`, strips suffixes, sorts names, and writes generated `src/_playgrounds.rs` with `playgrounds!(...)`.
- `src/main.rs` defines `playgrounds!` macro that expands to:
- Module declarations for each playground via `paste` crate identifier concatenation.
- CLI dispatch match arms from module string to `<module>_playground::run()`.
- Usage/listing output when no module argument is provided.
- `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/_playgrounds.rs"));` inlines generated macro invocation at compile time.
- Example module implemented: `src/variables_playground.rs` with `pub fn run()`.
Constraints
- Product constraints:
- Scope is a local CLI playground runner, not a packaged framework.
- Playground discovery is file-name convention based (`*_playground.rs`).
- Only modules with `pub fn run()` are valid execution targets.
- Technical constraints:
- Rust edition is 2024.
- Dependency set is minimal; current external dependency is `paste`.
- No runtime reflection; discovery and dispatch must be compile-time compatible.
- Generated file `src/_playgrounds.rs` is build output and should not be manually edited.
- `.gitignore` must exclude generated/build artifacts (`/target`, `src/_playgrounds.rs`).
Acceptance Criteria
- Discovery and listing:
- Given at least one `*_playground.rs` file, running `cargo run` prints usage text and available module names.
- Generated dispatch:
- After `cargo check` or `cargo run`, `build.rs` generates `src/_playgrounds.rs` containing `playgrounds!(...)` with discovered modules.
- Module execution:
- Running `cargo run -- variables` executes `variables_playground::run()` and prints expected demo output (`x`, `y`, shadowed `x`, greeting/pi line).
- Unknown module handling:
- Running `cargo run -- <unknown>` prints `Unknown module: '<unknown>'` to stderr without panic.
- Add-new-playground workflow:
- Creating `src/<new>_playground.rs` with `pub fn run()` causes `<new>` to appear in listing and be executable without manual edits in `main.rs`.
- Spec workflow:
- Active spec exists at `specs/specifications.md`.
- Completed spec archive includes this file as `specs/completed/specs-v0.md`.
Exclusions
- Do not introduce runtime plugin loading or dynamic library discovery.
- Do not replace Cargo/build-script flow with custom external codegen tooling.
- Do not require manual module registration in `main.rs` for each playground.
- Do not commit generated `src/_playgrounds.rs`.
- Do not expand v0 to include remote execution, web UI, persistence, or telemetry.
- Do not change the archive model from versioned files in `specs/completed/`.
Notes
- Repository currently has no commit history yet; this v0 spec captures the initial implemented baseline.
- `specs/specifications.md` remains the active editable spec for future iterations.
