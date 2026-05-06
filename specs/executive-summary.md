# Rustic Playground — Executive Summary

> **Note:** This document is public (lives in the open-source repo). Internal strategy, roadmap speculation, and monetization plans are intentionally excluded. For internal-only context, see the `project_strategy_2026.md` memory note.
>
> **Last edit:** 2026-04-20 (v0.3.6)

---

## One-line pitch

**A macOS desktop app for learning Rust, inspired by Swift Playgrounds.**

Write a `.rs` file, press ⌘R, see output stream live. No terminal required — but the toolchain isn't hidden either; it's just `cargo` underneath.

---

## The problem

Learning Rust has high friction. The standard path is:

1. Install `rustup` from a curl pipe to shell
2. Open a terminal, run `cargo new hello`
3. Open an editor, write code
4. Flip back to the terminal, run `cargo run`
5. Read the output, flip back to the editor
6. Repeat

Existing tools don't solve this for learners:

- **play.rust-lang.org** — excellent for single-snippet experiments, but browser-only, ephemeral, can't persist real work
- **VS Code + rust-analyzer** — a great IDE for professional development, but not a learning environment
- **RustRover** — polished IDE, but hides the toolchain (and paid)
- **Codecademy / Replit / Exercism** — web-based, ad-supported, require account sign-up, don't produce a real local toolchain

The result: motivated learners often stall at setup and never finish the Rust Book.

---

## The solution

**A native macOS desktop app** where:

- Each "playground" is a real `.rs` file in a real Cargo project with its own `fn main()`.
- Press **⌘R** and the app runs `cargo run`; stdout/stderr streams live to a panel below the editor.
- **20 Rust Book chapters** come pre-loaded as read-only playgrounds with a "Copy to Project" action to hack on your own version.
- A **Welcome Wizard** handles `rustup` install and repair on first launch, so beginners don't have to fight the toolchain before writing their first line.
- Live **`cargo check`** diagnostics appear inline in the editor as you type — the compiler is your teacher.
- **No ads, no donation nags, no account sign-up** — ever.

---

## Target audience

**Primary:** People learning Rust on a Mac. Often:
- CS students working through The Rust Programming Language
- Experienced developers (from C/C++, Python, Swift) picking up Rust
- Hobbyists exploring systems programming

**Secondary:** Educators / mentors pointing students at a Mac-friendly starter environment.

---

## Why macOS only (on purpose)

- Solo developer; limited bandwidth
- Target audience skews heavily toward Macs
- Rather one platform be excellent than three platforms be mediocre
- Apple Silicon + Tauri 2 + Monaco is a stable, productive stack

Linux and Windows ports are explicitly parked (low ROI for the target audience; large engineering burden).

---

## Differentiation

| vs. | How Rustic Playground differs |
|---|---|
| **play.rust-lang.org** | Persistent projects, multi-file, offline-capable, installs a real toolchain on your machine. (Intentionally a *complement*, not a competitor.) |
| **VS Code + rust-analyzer** | Purpose-built for learning, not general-purpose dev. Lower setup friction. |
| **RustRover** | Free and open source. Doesn't hide the toolchain; you learn it. |
| **Replit / Codecademy** | Native macOS app, not a browser tab. No ads, no donation prompts, no account required. Your code stays on your machine. |
| **Reading the Rust Book alone** | Interactive feedback loop. Book examples become playable programs, not static code fences. |

---

## Positioning

**Complement to [play.rust-lang.org](https://play.rust-lang.org), not a replacement.**

The web playground is perfect for "try this snippet." Rustic Playground is for working through The Book locally, keeping your experiments, and building multi-file projects.

---

## Current state (v0.3.6, 2026-04-20)

- **Shipping:** Signed + notarized DMG via GitHub Releases; Mac App Store path not yet pursued
- **License:** MIT / Apache-2.0
- **Tech stack:** Tauri 2 · Svelte 5 (runes) · Monaco editor · Rust backend (~1,500 lines)
- **Minimum OS:** macOS 12 (Monterey) on Apple Silicon
- **Distribution:** Direct download from [rusticplayground.dev](https://rusticplayground.dev) → GitHub Releases; update checker via GitHub Releases API
- **Release cadence:** ~monthly since v0.1.7 (Feb 2026)
- **Discord community:** live, with Discord invite in app + site

---

## Roadmap highlights (public)

**Recently shipped**
- v0.3.5: Rust toolchain version gate — enforces `rustc ≥ 1.85` floor for edition 2024
- v0.3.4: In-app toolchain installer + repair (guided + manual install paths)
- v0.3.3: Saved snapshots with revert, update checker, run lifecycle status
- v0.3.2: Welcome Wizard, language gating

**Near-term focus**
- Launch push: Reddit (r/learnrust, r/tauri, r/rust), This Week in Rust, Rust Users Forum, Show HN, LinkedIn, Twitter/X
- Community feedback → iterate on polish and frictions

**Longer-term ideas (parked — see `specs/roadmap.md`)**
- Multi-version toolchain picker (per-project rustc version switching)
- Native embeddable code editor crate (replacing Monaco — separate project)

---

## Team

**Solo developer:** Jagmeet Chawla. Former VP Cloud; currently focused on agentic systems and Rust tooling.

Built with substantial help from Claude Code (Anthropic) during a sabbatical — an origin the blog post discusses openly.

---

## Ask

Rustic Playground is free and open source. If you find it valuable:

1. **Try it.** Download at [rusticplayground.dev](https://rusticplayground.dev)
2. **Share feedback.** What features would you like to see? What tripped you up?
   - [GitHub Issues](https://github.com/jagmeetchawla/rustic-playground/issues)
   - [Discord community](https://rusticplayground.dev) (link in app + site)
3. **Star the repo** on [GitHub](https://github.com/jagmeetchawla/rustic-playground) — visibility helps reach more learners.

---

## Attribution

Rust Book examples loaded via Help → Load Rust Book Examples are based on *The Rust Programming Language* by Steve Klabnik and Carol Nichols. Licensed MIT/Apache-2.0. © Rust Project Developers (2010). Source: <https://github.com/rust-lang/book>

Playground code is original educational Rust, not verbatim from the book. An `attribution.md` is placed in every chapter's `content/` folder.
