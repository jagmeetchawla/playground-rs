WORKFLOW

Agent Steps
- Read AGENTS.md first to orient.
- Read specs/specifications.md before any feature or iteration work.
- Read specs/architecture.md when making structural decisions.
- Prefer small, focused diffs.
- Run formatting and checks before finalising.

Spec Lifecycle
- Active spec lives in specs/specifications.md.
- When an iteration is complete, archive the spec to specs/archive/spec-v<N>.md.
- Refresh specs/specifications.md for the next iteration after archiving.
- Treat specs/archive/ as historical context only — do not edit archived specs.

Local Commands
- cargo fmt
- cargo clippy -- -D warnings
- cargo test
- cargo run
- cargo run list

Change Checklist
- New or changed code is formatted (cargo fmt).
- Lints pass (cargo clippy -- -D warnings).
- Tests pass (cargo test).
- specs/specifications.md and specs/architecture.md updated if behaviour or structure changed.
