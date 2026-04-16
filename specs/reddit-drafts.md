# Reddit Drafts — r/learnrust + r/rust

> **Status:** Drafts. Split-day posting on peak Reddit days:
> - **r/learnrust:** Monday 2026-04-20, ~9–10am ET (smaller, friendlier sub first — dry run for copy)
> - **r/rust:** Thursday 2026-04-23, ~9–10am ET (peak traffic, 2–3 days to fold in r/learnrust learnings)
>
> **Pre-post checklist:** verify each subreddit's current self-promo rules, confirm flair options, have hero screenshot ready to upload.
> **Last edit:** 2026-04-16

---

## r/learnrust (Monday 2026-04-20)

**Target flair:** `project` or similar (check current options)

### Title

```
A macOS playground for learning Rust — Rust Book chapters pre-loaded, no terminal required
```

Alt:

```
Built an app to make working through the Rust Book easier on macOS
```

### Body

```
Hey r/learnrust — built this for exactly the people in this subreddit.

I started the Rust Book this year after putting it off for seven (ha). What I wanted was what Swift Playgrounds gave me in 2015: open a thing, write code, press play, see output, move on. Rust Book + terminal + editor + back to terminal was the opposite experience.

So I built **Rustic Playground**, a macOS desktop app.

[hero screenshot]

**What it does:**
- All 20 Rust Book chapters pre-loaded as read-only playgrounds — click to read the code, click "Copy to Project" to hack on your own version
- Welcome Wizard sets up `rustup` for you on first launch (or repairs it if something's broken)
- Each playground is a real `.rs` file in a real Cargo project — so you're learning the actual toolchain, not a sandboxed imitation
- Press ⌘R, stdout streams live in a panel below the editor
- Live error highlighting as you type, via `cargo check`

**What it helps with:**
- You don't need to fully understand Cargo before you can start — the app scaffolds it
- You don't have to keep switching between terminal and editor
- Your experiments are saved as you go — each chapter is a separate project, easy to revisit
- The compiler's feedback shows up right in the editor, not hidden in a terminal buffer

**What it doesn't do:**
- Not a tutorial — you still read the Book (either via "Read Online" links in the app or your own copy)
- Not a replacement for terminal skills you'll need eventually — the goal is to let you focus on Rust concepts first, and pick up the rest of the workflow later

Free, open source (MIT/Apache-2.0), signed + notarized DMG: https://rustic-playground.app

Happy to answer anything — especially from folks just starting out. **What tripped you up in the first few chapters?** That's the feedback that'll shape the next version.
```

### Notes on r/learnrust tone

- Closing question ("what tripped you up") invites discussion, which Reddit's algorithm rewards
- Emphasize the *not* list too — honesty about what it isn't reduces "but actually" comments
- Less stack detail than r/rust — beginners don't care about Svelte 5 vs Tauri tradeoffs

---

## r/rust (Thursday 2026-04-23)

**Target flair:** `show & tell` (or `project` / `announcement` — check current options)

### Title

```
I built a macOS playground for learning Rust, inspired by Swift Playgrounds
```

Alt if the first reads too pitchy to moderators:

```
Rustic Playground — a macOS desktop app for learning Rust (Show & Tell)
```

### Body

```
Hey r/rust — I'm Jagmeet, solo dev on Rustic Playground.

**What it is:** A macOS desktop app where each project is a Cargo package and each playground is a `.rs` file in `src/bin/` with its own `fn main()`. Press ⌘R — stdout/stderr streams live. No terminal required, but the toolchain isn't hidden; it's just `cargo` underneath.

**Why:** I bought the Rust Book in 2018, didn't open it for seven years. When I finally sat down this year, I missed what Swift Playgrounds gave me in 2015 — write code, press play, see output, move on. rust-analyzer + VS Code is great, but it isn't a playground. RustRover is a great IDE but hides the toolchain. I wanted to learn the language *and* the toolchain. So I built the middle ground.

[hero screenshot]

**What's in v0.3.5:**
- 20 Rust Book chapters pre-loaded as read-only playgrounds, "Copy to Project" to hack on them
- Welcome Wizard handles `rustup` install/repair on first launch
- Live `cargo check` diagnostics surfaced as Monaco markers
- Signed + notarized DMG

**Positioning:** Meant as a **complement** to play.rust-lang.org, not a replacement. The web playground is perfect for "try this snippet." Rustic Playground is for working through The Book locally, keeping your experiments, and building multi-file projects.

**Stack:** Tauri 2 + Svelte 5 + Monaco + Rust backend. macOS-only on purpose — I'd rather one platform be excellent than three mediocre.

Free, open source (MIT/Apache-2.0): https://rustic-playground.app

Also got good feedback on r/learnrust earlier this week: [PASTE r/learnrust THREAD URL AFTER MONDAY'S POST]

Happy to answer questions about the Tauri build, how the book examples are shipped in the binary, the process-tree kill dance for `cargo run`, or why Rust + agentic dev turned out to pair surprisingly well.
```

### Expected pushback (pre-plan responses)

- **"Why not just use rust-analyzer + VS Code?"** — Straight answer: I do, for real projects. This is for learning, specifically the feedback-loop friction of repeated `cargo run`.
- **"Why macOS-only?"** — Solo dev, limited time, Mac was what I had. Linux/Windows GUI ports are parked, not forbidden — if this takes off I'll revisit.
- **"Tauri? Why not native?"** — Native macOS rewrite is an explicit future milestone. Tauri lets me ship to real users today and iterate fast.
- **"Is this sandboxed?"** — No, intentionally, like Xcode/Terminal. Compiling arbitrary Rust code inside a sandbox is a nightmare and defeats the learning goal.

---

## Split-day mechanics

- **Monday 2026-04-20 ~9–10am ET: r/learnrust.** Lower-stakes dry run on a strong Reddit day. Then:
  - Monitor comments, reply for 4–6h after posting, then check in daily Tue–Wed
  - Note any wording that landed poorly or questions that came up repeatedly
  - Fold learnings into the r/rust post copy before Thursday
- **Thursday 2026-04-23 ~9–10am ET: r/rust.** Include a link to the r/learnrust thread as a trust signal.
- **Don't cross-post verbatim** using Reddit's cross-post feature — each subreddit gets its own tailored post
- Upload the hero screenshot directly to each post (image + text works now on Reddit)

## Posting checklist (Monday, r/learnrust)

- [ ] Verify r/learnrust current rules (wiki + sidebar)
- [ ] Hero screenshot ready (preferably the one from rustic-playground.app homepage)
- [ ] Flair selected correctly
- [ ] Post title matches current convention (check recent top posts in the sub)
- [ ] Fresh browser session logged into Reddit account
- [ ] Free 4–6h afterward to reply to comments

## Posting checklist (Thursday, r/rust)

- [ ] Re-check r/rust current self-promo rules
- [ ] Insert r/learnrust thread URL into the body (line: "Also got good feedback on r/learnrust…")
- [ ] Fold any r/learnrust learnings into the copy (wording, FAQ, etc.)
- [ ] Flair selected correctly (show & tell)
- [ ] Free 4–6h afterward to reply to comments

## After posting (both)

- **Update release-plan.md** — check off steps 8 and 9 once both are live
- **Save the thread URLs** — you'll link them from the Show HN first-comment ("there's also ongoing discussion on r/rust: [link]")
- **Watch for FAQs** — answers that come up repeatedly should be folded into the Show HN body + the site's FAQ
