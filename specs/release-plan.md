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

## Launch — Day 1

- [ ] 8. **r/rust** — lead with screenshot, one-line pitch, link to website
  - Title idea: *"I built a macOS playground for learning Rust, inspired by Swift Playgrounds"*
  - Include: screenshot, website link, mention it's free + open source + MIT
  - Don't link GitHub directly — link to rustic-playground.app

- [ ] 9. **r/learnrust** — same post, tailored for beginners
  - Emphasize: built-in Rust Book examples, no terminal required, Welcome Wizard

---

## Launch — Day 2–3

- [ ] 10. **This Week in Rust** — submit to the newsletter at [this-week-in-rust.org](https://this-week-in-rust.org)
  - They feature community projects. Very high signal for Rust developers.

- [ ] 11. **Rust Users Forum** — post at [users.rust-lang.org](https://users.rust-lang.org)
  - Showcase category. More detailed write-up OK here.

---

## Launch — Day 3–5

- [ ] 12. **Hacker News** — submit as Show HN
  - Title: *"Show HN: Rustic Playground – a macOS desktop app for learning Rust"*
  - HN likes: native apps, developer tools, thoughtful engineering

- [ ] 13. **Twitter/X** — post with `#rustlang` hashtag and screenshot

---

## Post-launch

- [ ] 14. Monitor GitHub Issues and Discussions for feedback
- [ ] 15. Update r/rust link on website to point to actual post
- [ ] 16. Iterate based on feedback — prioritize bugs, then feature requests

---

## Tips

- **Lead with the screenshot** — people scroll past text
- **One-line pitch:** "Write Rust, press ⌘R, see output. No terminal."
- **Link to the website**, not directly to GitHub
- **Stagger posts** over ~5 days to manage feedback and adapt messaging
- **Don't announce until steps 1–7 are complete**
