# Show HN Draft — Rustic Playground

> **Status:** Draft, not yet posted. HN is Day 3–5 per [release-plan.md](release-plan.md) — r/rust + Rust Users Forum go first.
> **Last edit:** 2026-04-16

---

## Title (60 chars)

```
Show HN: Rustic Playground – Swift Playgrounds, but for Rust
```

HN cap is 80 chars; this leaves room. The "Swift Playgrounds for Rust" framing is universally understood on HN and front-loads the metaphor before the scroll.

---

## Body (~230 words)

Rustic Playground is a macOS desktop app for learning Rust — written because the Rust Book experience was a constant shuttle between terminal, editor, and terminal. Each project is a real Cargo package; each playground is a `.rs` file with its own `fn main()`. Press ⌘R, stdout/stderr streams live. No terminal required, but the toolchain isn't hidden — it's just `cargo` underneath.

The closest I'd had to "fun while learning a language" was Swift Playgrounds back in 2015, so I built the same shape for Rust. Full origin story: https://rustic-playground.app/blog/why-i-built-this/

Positioning: meant as a **complement** to play.rust-lang.org, not a replacement. The web playground is perfect for "try this snippet." Rustic Playground is for working through The Book, keeping your experiments, and building multi-file projects locally.

What's in it:
- Monaco editor, Svelte 5 + Tauri 2 shell, Rust backend
- 20 Rust Book chapters pre-loaded as read-only playgrounds, with a "Copy to Project" action
- Welcome Wizard handles `rustup` install/repair
- Free, open source (MIT/Apache-2.0), signed + notarized

macOS-only on purpose — Linux/Windows GUI ports are parked. Target audience is learners; rather one platform be excellent than three mediocre.

Download + source: https://rustic-playground.app

Happy to answer questions on the Tauri stack, the book-examples pipeline, or why Rust + agentic dev turned out to be a surprisingly good pairing.

---

## Posting checklist

- [ ] r/rust post live (Day 1 per release-plan.md)
- [ ] r/learnrust post live (Day 1)
- [ ] Rust Users Forum post live (Day 2–3)
- [ ] This Week in Rust submission sent (Day 2–3)
- [ ] Demo video at `docs/videos/demo.mp4` (optional but recommended before HN)
- [ ] First comment pre-written for self-reply (common HN pattern — technical deep-dive answer to "how did you build this")
- [ ] Post time chosen: Tue–Thu, 8–10am ET is the usual sweet spot for Show HN
- [ ] Available to answer comments for 4–6h after posting

## Notes for later iteration

- **Agentic dev angle:** kept in closing line as discussion hook. Blog post expands on it. Expect either (a) thoughtful AI-skeptic pushback to engage with honestly, or (b) follow-up questions about Claude Code workflow.
- **What NOT to mention:** C/C++/Zig/Swift, Power Edition, multi-language roadmap. Per [project_strategy_2026.md](../../../.claude/projects/-Users-jagmeetchawla-Developer-Projects-rustic-playground/memory/project_strategy_2026.md), external messaging is Rust-only.
- **First-comment idea** (self-reply to seed discussion):
  > A few things I'd flag if you're curious about the internals:
  > - The book examples are defined in Rust source (`rust_book.rs`, ~2700 lines) rather than loaded from markdown — made them easy to code-review and ship with the binary.
  > - Process lifecycle was the gnarliest part: killing `cargo run` has to walk the process tree, not just SIGKILL the parent.
  > - Tauri 2's capability system silently drops un-permitted calls — burned an afternoon on that one.
  > - Monaco in a WKWebView is… fine. Native editor component is the long-term itch but nothing in Rust has the feature set yet.
