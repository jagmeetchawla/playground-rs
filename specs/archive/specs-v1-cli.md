# SPECS ARCHIVE — v1 CLI (src/bin approach)

```
Status
- Version:  v1-cli
- Archived: 2026-03-30
- Era:      Between v0 (build.rs) and v1.0 (Tauri GUI)
- Source:   git commit 231314e — "feat: initial commit — Rust CLI playground runner (src/bin approach)"
- Note:     These specs were active briefly before the Tauri GUI spec replaced them.
            Restored from git history on 2026-03-30 — they were never explicitly archived
            at the time of the transition.
```

---

## SPECIFICATION

```
Status
- Version: draft
- Date:
- Owner:

Product
What
- Describe what is being built.
- Define the user-facing outcome.

Why
- Describe the user or business problem.
- Explain why this matters now.

Architecture
How
- Describe the technical approach.
- Identify key components and interactions.

Constraints
- Product constraints: scope, deadlines, non-goals.
- Technical constraints: language/runtime, infrastructure, integrations, security, performance, compliance.

Acceptance Criteria
- List concrete, testable outcomes.
- Write criteria that can be translated into test cases.
- Include functional and non-functional checks when relevant.

Exclusions
- Explicitly list what must NOT be built or changed.
- Include out-of-scope features, technologies, and implementation paths.

Notes
- Add open questions, dependencies, and risks.
```

*Note: specifications.md was still a blank template at this commit.
The architecture.md below is where the real decisions were documented.*

---

## ARCHITECTURE

```
Approach
- Each playground is a standalone binary in src/bin/<name>.rs with a fn main() entry point.
- Cargo auto-discovers every .rs file in src/bin/ as a binary target — no registration, no codegen.
- The runner (src/main.rs) is a thin CLI that lists, picks interactively, and delegates to
  `cargo run --bin <name>`.
- Dependencies are shared across all playgrounds via a single Cargo.toml.

Project Structure
- src/main.rs              — the playground runner (clap CLI: list, interactive pick, run)
- src/bin/<name>.rs        — one file per playground, each fully self-contained
- specs/                   — project specs and docs
- specs/archive/           — archived past specs
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
```

---

## ACCEPTANCE CRITERIA

*acceptance-criteria.md was empty at this commit — criteria had not yet been written
for the CLI era. See specs-v1.0-tauri.md for the first full acceptance criteria.*
