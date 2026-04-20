# Reddit Drafts — r/learnrust + r/tauri + r/rust

> **Status:** Drafts. Three-sub staggered posting on peak Reddit days:
> - **r/learnrust:** Monday 2026-04-20, ~9–10am ET (smaller, friendlier sub first — dry run for copy)
> - **r/tauri:** Wednesday 2026-04-22, ~9–10am ET (Tauri-specific framing, stack + gotchas angle)
> - **r/rust:** Thursday 2026-04-23, ~9–10am ET (peak traffic, 2–3 days to fold in r/learnrust learnings)
>
> Each post is tailored — don't cross-post verbatim. r/learnrust leads with learner value, r/tauri leads with stack + gotchas, r/rust leads with positioning vs play.rust-lang.org.
>
> **Pre-post checklist:** verify each subreddit's current self-promo rules, confirm flair options, have hero screenshot ready to upload.
> **Last edit:** 2026-04-20

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
- No ads, no donation nags, no account sign-up. MIT/Apache-2.0, and staying that way.

Free, open source, signed + notarized DMG: https://rustic-playground.app

Two small asks if you find it valuable:

1. Share feedback in the comments — what features would you like to see? What tripped you up in the first few chapters?
2. Take a moment to star the repo: https://github.com/jagmeetchawla/rustic-playground

That's it.
```

### Notes on r/learnrust tone

- Closing question ("what tripped you up") invites discussion, which Reddit's algorithm rewards
- Emphasize the *not* list too — honesty about what it isn't reduces "but actually" comments
- Less stack detail than r/rust — beginners don't care about Svelte 5 vs Tauri tradeoffs

---

## r/tauri (Wednesday 2026-04-22)

**Target flair:** `Show & Tell` / `Project` / similar (check current options)

### Title

```
Shipped my first Tauri 2 app — macOS Rust learning playground (stack + gotchas)
```

Alt (punchier):

```
Built a Rust learning app with Tauri 2 + Svelte 5 + Monaco — some takeaways
```

### Body

```
Hey r/tauri — shipped my first real Tauri 2 app and wanted to share the stack + some things I learned the hard way.

**App: Rustic Playground** — a macOS desktop app for learning Rust, inspired by Swift Playgrounds. Write a `.rs` file, press ⌘R, stdout/stderr streams live. Each playground is a real Cargo package with its own `fn main()`, not a sandbox.

[hero screenshot]

**Stack:**
- Tauri 2 shell
- Svelte 5 (runes: `$state`, `$derived`, `$effect`)
- Monaco editor via ES module import
- Rust backend (~1500 lines across 6 modules)
- Signed + notarized DMG, fully automated in build pipeline

**Tauri-specific gotchas that cost me hours:**

1. **Capabilities fail silently.** Every `window.*`, `dialog.*` API call needs an explicit permission in `capabilities/default.json`. If you miss one, the call just… doesn't happen. Zero error in the console. Burned half a day on a broken resize handler before I realized.

2. **WKWebView eats `window.confirm/alert/prompt`.** They appear to work in dev but do nothing. Use the Tauri dialog plugin or build a custom modal.

3. **macOS menu items are baked at build time.** `MenuBuilder` constructs the menu once; can't toggle `.enabled()` after construction. I rebuild the entire menu on relevant state changes.

4. **Killing `cargo run` needs the full process tree.** SIGTERM to the cargo PID leaves the compiled binary running. Walk the process group or recurse through children.

5. **Serde u32 rejects JS floats silently.** JavaScript drag math produces `324.5`. Rust `u32` fails to deserialize without a clear error — the IPC call just throws. `Math.round()` everything before passing numeric values to commands.

6. **Shared `target/` causes cargo lock conflicts.** If your app runs `cargo check` on user code in the same `target/` the main binary uses, `cargo build` and `cargo check` can deadlock. Separate target dirs per use case.

**What Tauri 2 got right for me:**
- IPC is fast and ergonomic once capabilities are sorted
- Bundler produces clean signed DMGs with minimal config
- Svelte 5 + Vite + Tauri hot-reload loop is solid
- Plugin ecosystem is small but what's there works (dialog, shell, updater)

**What I wish existed:**
- A native embeddable code editor crate (I'm stuck on Monaco in a webview because no Rust equivalent has the features yet)
- Better upfront docs on the capabilities list — it's long and mostly copy-paste from examples
- A standard pattern for post-build DMG modification (I wrote my own volume-icon-strip + re-sign + notarize + staple script)

Free, open source (MIT/Apache-2.0): https://rustic-playground.app
Source: https://github.com/jagmeetchawla/rustic-playground

Happy to talk about any of the above — the build pipeline, the process-tree kill dance, the capabilities rabbit hole, or why Rust + agentic dev turned out to pair surprisingly well.
```

### Notes on r/tauri tone

- Audience is developers building Tauri apps, not end users. Lead with technical substance, not marketing.
- Gotchas list is the most shareable part — "I hit this too" comments = engagement
- Mention the stack combinations (Svelte 5 + Tauri + Monaco) since that's a common search
- Keep the "what I wish existed" short but honest — invites the community to say "there's X" which you might not know about

### Expected discussion topics to be ready for

- **"Why Svelte 5 over React/Solid/Vue?"** — Tauri community skews pragmatic; Svelte 5 runes are still fresh, some devs haven't tried them.
- **"How's Monaco performance in WKWebView?"** — it's fine for files <5k lines; larger files start to feel it.
- **"Why not GPUI / Dioxus / Iced / egui?"** — honest answer: none of them has a code editor component with tree-sitter + LSP that matches Monaco, and building one is a multi-year project.
- **"Any plans for Windows/Linux?"** — parked, low ROI for the target audience (macOS Rust learners). See roadmap.

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

## Staggered-post mechanics

- **Monday 2026-04-20 ~9–10am ET: r/learnrust.** Lower-stakes dry run on a strong Reddit day. Then:
  - Monitor comments, reply for 4–6h after posting, then check in daily Tue–Wed
  - Note any wording that landed poorly or questions that came up repeatedly
  - Fold learnings into the r/tauri and r/rust post copy
- **Wednesday 2026-04-22 ~9–10am ET: r/tauri.** Different audience (Tauri devs, not Rust learners) — distinct framing around stack + gotchas. Can reference r/learnrust thread if the conversation went well.
- **Thursday 2026-04-23 ~9–10am ET: r/rust.** Include links to both earlier threads as trust signals.
- **Don't cross-post verbatim** using Reddit's cross-post feature — each subreddit gets its own tailored post
- Upload the hero screenshot directly to each post (image + text works now on Reddit)

## Posting checklist (Monday, r/learnrust)

- [ ] Verify r/learnrust current rules (wiki + sidebar)
- [ ] Hero screenshot ready (preferably the one from rustic-playground.app homepage)
- [ ] Flair selected correctly
- [ ] Post title matches current convention (check recent top posts in the sub)
- [ ] Fresh browser session logged into Reddit account
- [ ] Free 4–6h afterward to reply to comments

## Posting checklist (Wednesday, r/tauri)

- [ ] Verify r/tauri current rules (small sub — likely permissive, but check)
- [ ] Optionally insert r/learnrust thread URL if that post landed well ("also got thoughtful feedback on r/learnrust")
- [ ] Hero screenshot + maybe a dev-oriented shot (e.g., the app running with Tauri dev tools)
- [ ] Flair selected correctly (Show & Tell / Project)
- [ ] Free 4–6h afterward — r/tauri audience is technical and asks deep questions

## Posting checklist (Thursday, r/rust)

- [ ] Re-check r/rust current self-promo rules
- [ ] Insert r/learnrust AND r/tauri thread URLs into the body if both went well
- [ ] Fold any r/learnrust / r/tauri learnings into the copy (wording, FAQ, etc.)
- [ ] Flair selected correctly (show & tell)
- [ ] Free 4–6h afterward to reply to comments

## After posting (all three)

- **Update release-plan.md** — check off step 8 (r/rust), step 9 (r/learnrust), and note r/tauri as an added channel
- **Save all thread URLs** — you'll link them from the Show HN first-comment ("also ongoing discussion on r/rust + r/tauri + r/learnrust: [links]")
- **Watch for FAQs** — answers that come up repeatedly should be folded into the Show HN body + the site's FAQ
- **Aggregate gotchas discussion** — r/tauri might surface additional Tauri pain points worth a follow-up blog post
