AGENTS GUIDE
Project
- Name: playground-rs
- Language: Rust
- Build tool: Cargo
Specs Workflow
- Primary spec file: specs/specifications.md
- Archive directory: specs/completed/
- Always read specs/specifications.md before planning or coding.
- Treat specs/completed/ as historical context only.
- When a spec is completed, archive it with a versioned filename in specs/completed/ (example: spec-v1.md, spec-v2.md).
- After archiving, refresh specs/specifications.md for the next active spec.
Spec Sections (required)
- Product: what and why.
- Architecture: how.
- Constraints: product and technical.
- Acceptance Criteria: testable criteria that map to test cases.
- Exclusions: explicit list of what not to do.
Agent Workflow
- Read the task and current spec fully before editing.
- Prefer small, focused diffs.
- Run formatting and checks before finalizing.
Local Commands
- cargo fmt
- cargo clippy -- -D warnings
- cargo test
- cargo run
Coding Conventions
- Keep code idiomatic and explicit.
- Prefer clear names over short names.
- Add brief comments only where behavior is non-obvious.
Change Checklist
- Ensure new or changed code is formatted.
- Ensure lints and tests pass.
- Update docs/specs when behavior changes.
