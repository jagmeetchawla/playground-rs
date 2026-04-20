# LinkedIn — launch-day prep

> **Status:** Draft. Post to LinkedIn first (~30 min before r/learnrust) so there's fresh activity when visitors click through from Reddit.
> **Last edit:** 2026-04-20

---

## Profile updates (one-time, do first)

### Headline
Keep it short; LinkedIn truncates. Options:

**A. Integrate Rustic Playground (my pick):**
```
Building Rustic Playground · Former VP Cloud · Agentic Systems
```

**B. Keep existing, let featured section do the work:**
```
Former VP Cloud · Building Agentic Systems
```

If you go with A, visitors see "Rustic Playground" in the first 3 words — useful when they arrived from a Rustic Playground post. If you go with B, the headline stays evergreen and you rely on the featured section + recent post.

### Featured section
Pin 1–2 items at the top of your profile:

1. **Primary pin:** Link to `rustic-playground.app` — LinkedIn will auto-fetch the OG image (hero icon + title). Add caption: *"Rustic Playground — Swift Playgrounds for Rust, on macOS."*

2. **Secondary pin (optional):** The blog post `rustic-playground.app/blog/why-i-built-this/`. Caption: *"Why I built it — origin story."*

### About section
Keep it close to what you have; add a single line near the end:

> *Currently building [Rustic Playground](https://rustic-playground.app) — a macOS desktop app for learning Rust, inspired by Swift Playgrounds.*

Don't rewrite the whole About. Minimal edit keeps it genuine.

---

## Launch-day post

Post this ~30 min before the r/learnrust post goes up. Reason: if the Reddit post sends traffic to your LinkedIn, the newest thing on your activity feed is the launch, not whatever was there from months ago.

### Visual
Attach the hero screenshot from `rustic-playground.app`. LinkedIn rewards image-heavy posts algorithmically.

### Body (~220 words)

```
After a sabbatical and a lot of Rust Book chapters, I shipped Rustic Playground today — a macOS desktop app for learning Rust, inspired by Swift Playgrounds.

Why I built it: I bought the Rust Book in 2018, didn't open it for seven years. When I finally sat down this year I missed what Swift Playgrounds gave me in 2015 — write some code, press play, see output, move on. Instead I was shuttling between terminal, editor, and terminal again. So I built the middle ground.

Each playground is a real Cargo package, not a sandbox. Press ⌘R and stdout streams live. All 20 Rust Book chapters come pre-loaded. The Welcome Wizard handles rustup install/repair so beginners don't need to fight the toolchain before writing their first line.

macOS-only on purpose — rather one platform be excellent than three mediocre.

One thing I didn't expect: Rust + AI-assisted dev (Claude Code) turned out to pair surprisingly well. The compiler's strictness catches a class of AI slip-ups before they ever run.

Free, open source, signed + notarized:
→ https://rustic-playground.app

Full origin story (why now, why macOS, what I learned):
→ https://rustic-playground.app/blog/why-i-built-this/

If you're learning Rust on a Mac, I'd love for you to try it. Feedback and feature requests welcome.

#rustlang #macOS #indiedev
```

### Hashtags
Three is the LinkedIn sweet spot. `#rustlang` reaches the Rust community on LinkedIn; `#macOS` catches Apple devs; `#indiedev` surfaces in the solo-founder feeds.

---

## Sequencing today

| Time | Action |
|---|---|
| Now | Update headline, featured pins, about-section line |
| +15 min | Post the LinkedIn post above |
| +30 min | **Then** post to r/learnrust |
| +30 min onward | Monitor LinkedIn + Reddit for comments, respond for 4–6h |

---

## Notes

- **Don't repost Reddit copy verbatim.** LinkedIn audience skews professional/enterprise; lead with "shipped" and the origin story, not "Show HN" vibes.
- **Don't overhype.** LinkedIn has an allergy to marketing-speak. Keep the post factual and personal.
- **Reply to comments promptly.** LinkedIn's algorithm surfaces posts with sustained OP engagement much more than one-and-done posts.
- **Second-degree connections are your amplifiers.** If a few ex-colleagues like or comment, the post gets shown to their networks — which is how LinkedIn organic reach actually works.
