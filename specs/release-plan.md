# Release Plan — Rustic Playground: The Rust Edition

---

## Pre-release (before announcing)

- [ ] 1. E2E test Rust Edition build (`VITE_EDITION=rust cargo tauri build --config editions/rust.json`)
- [ ] 2. Fix any issues found in testing
- [ ] 3. Upload DMG to GitHub Releases with release notes
- [ ] 4. Enable GitHub Discussions in repo settings (Settings → General → Features → Discussions)
- [ ] 5. Configure GitHub Pages (Settings → Pages → Source: main branch, /docs folder)
- [ ] 6. Set up DNS for rustic-playground.app → GitHub Pages
- [ ] 7. Verify website is live at rustic-playground.app

---

## Launch — Day 0 (Mon 2026-04-20) ✅

- [x] Clean up LinkedIn profile (headline, featured section, about line)
- [x] **LinkedIn launch post** (went up early — gets 24h compounding before Reddit traffic arrives)

---

## Launch — Day 1 (Tue 2026-04-21)

- [x] 8. **r/learnrust** — learner-first framing, drafts at `specs/reddit-drafts.md`
  - Emphasize: built-in Rust Book examples, no terminal required, Welcome Wizard
  - Lower-stakes dry run before the bigger subs
  - ~9–10am ET; stay at keyboard 4–6h for comments
  - Paste thread URL into the r/tauri and r/rust draft bodies
  - **POSTED 2026-04-21:** https://www.reddit.com/r/learnrust/comments/1srp94a/built_a_macos_playground_app_while_learning_rust/

---

## Launch — Day 3 (Thu 2026-04-23)

- [ ] 10. **r/tauri** — Tauri-dev framing, drafts at `specs/reddit-drafts.md`
  - Different audience (Tauri devs, not Rust learners). Lead with stack + gotchas.
  - Can reference r/learnrust thread as a trust signal if that post went well

---

## Launch — Day 4–5 (Fri–Sat 2026-04-24 to 25)

- [ ] 11. **This Week in Rust** — submit to the newsletter at [this-week-in-rust.org](https://this-week-in-rust.org)
  - They feature community projects. Very high signal for Rust developers.

- [ ] 12. **Rust Users Forum** — post at [users.rust-lang.org](https://users.rust-lang.org)
  - Showcase category. More detailed write-up OK here.

- [ ] **Optional: small v0.3.7 release** — one commit, one tag, one GitHub release.
  - Ideal content: fix or improvement from r/learnrust / r/tauri feedback (shows responsiveness). Fallback: extra template, shortcut, or polish item.
  - Keeps the GitHub repo looking active ahead of r/rust (Mon) + Show HN (Tue). Dead-looking repos hurt trust.
  - Full release flow: `NOTARIZE=1 ./scripts/build-editions.sh rust` → tag → push → gh release create. See `reference_release_checklist.md` memory.

---

## Launch — Day 7 (Mon 2026-04-27)

- [ ] 13. **r/rust** — lead with screenshot, one-line pitch, link to website
  - Title idea: *"I built a macOS playground for learning Rust, inspired by Swift Playgrounds"*
  - Include: screenshot, website link, mention it's free + open source + MIT
  - Link to r/learnrust and r/tauri threads + TWiR mention if all went well
  - Don't link GitHub directly — link to rustic-playground.app

---

## Launch — Day 8 (Tue 2026-04-28)

- [ ] 14. **Hacker News** — submit as Show HN (draft at `specs/show-hn-draft.md`)
  - Title: *"Show HN: Rustic Playground – Swift Playgrounds, but for Rust"*
  - HN likes: native apps, developer tools, thoughtful engineering
  - Reference all prior community threads in the self-reply first comment

- [ ] 15. **Twitter/X** — post with `#rustlang` hashtag and screenshot

---

## Post-launch

- [ ] 16. Monitor GitHub Issues and Discussions for feedback
- [ ] 17. Update r/rust link on website to point to actual post
- [ ] 18. Iterate based on feedback — prioritize bugs, then feature requests

---

## CS Program Outreach (Week 2+, after Reddit/HN results are in)

> Status: **idea captured, figure out later.** Do NOT start during launch week —
> it pulls focus from the Reddit/HN spike and educator emails land better with
> "launched last week on HN / r/rust, here's the reception" as social proof.

**Why this audience:** CS undergrads are the sharp-end "Swift Playgrounds for
Rust" case — they're told to install rustup + pick an IDE + learn Cargo + learn
the language simultaneously. The app collapses the first three so they focus on
the fourth. First LinkedIn engagement (2026-04-20) came from a CS undergrad
building AI agents, which fits the pattern even as a network-goodwill data point.

**Warm-intro starting point (highest-leverage option):**
- Jagmeet's daughters are at **Georgia Tech, UC Berkeley, and Columbia** —
  three top CS programs with strong systems/Rust footprints. A student
  introducing a tool to their own TA or professor is vastly higher-signal
  than cold outreach from a stranger.
- Pilot move: ask each daughter which course in their program touches Rust
  (or systems programming more broadly), and whether they'd be willing to
  mention the app to that course's TA or prof. No pressure — if they're
  not comfortable, don't push.
- If even one of three lands a "TA tried it, shared it in lab" moment,
  that's a stronger footprint than 20 cold emails.

**Broader who-to-target (once warm intros have run):**
- Universities with Rust in CS curriculum: Stanford CS110L, Brown CSCI 1260,
  CMU 15-410 variants, a few systems courses elsewhere. Enumerate before
  broad outreach.
- Professors teaching the specific Rust-adjacent courses (not department chairs).
- **TAs running the labs** — underrated channel. Students listen to TAs, and
  TAs are usually thrilled when someone reduces their office-hours burden.
- Student Rust clubs and hackathons — lower-effort, lower-stakes parallel track.

**The ask (important — don't pitch "teach with my tool"):**
> "For office hours and students who can't get rustup installed on their
> laptop — this removes ~2 hours of setup friction. Free, open source,
> macOS-only for now. Not asking you to change your syllabus."

Small footprint, easy yes. Positions the app as a support tool, not a
curriculum replacement.

**Timing:**
- NOT during launch week (2026-04-20 to 2026-04-28). Would split attention.
- Start Week 2–3 of post-launch, once there's a public launch story to
  reference.
- Aligns with north star (rust-lang.org / play.rust-lang.org complement):
  the desktop/local counterpart to a hosted web playground.

**Open questions (figure out when we pick this up):**
- Which universities, concretely? Need a list of Rust-teaching CS programs
  with professor + TA contact info.
- Email templates — one for professors, one for TAs, one for student clubs.
- Tracking: how do we know if outreach converts? UTM tags on website links?
  Self-reported via signup form?
- Do we want a "for educators" page on the website? Probably, eventually.

---

## Tips

- **Lead with the screenshot** — people scroll past text
- **One-line pitch:** "Write Rust, press ⌘R, see output. No terminal."
- **Link to the website**, not directly to GitHub
- **Stagger posts** over ~5 days to manage feedback and adapt messaging
- **Don't announce until steps 1–7 are complete**
