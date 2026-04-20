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

## Prep — Mon 2026-04-20

- [ ] Clean up LinkedIn profile (headline, featured section, about line)
- [ ] See `specs/linkedin-post.md` for plan
- [ ] No public posts today — just prep

---

## Launch — Day 1 (Tue 2026-04-21)

- [ ] 8. **LinkedIn post** — launch announcement (draft in `specs/linkedin-post.md`)
  - Post ~30 min before r/learnrust so fresh activity is on the profile

- [ ] 9. **r/learnrust** — learner-first framing, drafts at `specs/reddit-drafts.md`
  - Emphasize: built-in Rust Book examples, no terminal required, Welcome Wizard
  - Lower-stakes dry run before the bigger subs

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

## Tips

- **Lead with the screenshot** — people scroll past text
- **One-line pitch:** "Write Rust, press ⌘R, see output. No terminal."
- **Link to the website**, not directly to GitHub
- **Stagger posts** over ~5 days to manage feedback and adapt messaging
- **Don't announce until steps 1–7 are complete**
