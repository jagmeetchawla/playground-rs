# Release Plan — Rustic Playground: The Rust Edition

---

## Pre-release (before announcing)

- [ ] 1. E2E test Rust Edition build (`VITE_EDITION=rust cargo tauri build --config editions/rust.json`)
- [ ] 2. Fix any issues found in testing
- [ ] 3. Upload DMG to GitHub Releases with release notes
- [ ] 4. Enable GitHub Discussions in repo settings (Settings → General → Features → Discussions)
- [ ] 5. Configure GitHub Pages (Settings → Pages → Source: main branch, /docs folder)
- [ ] 6. Set up DNS for rusticplayground.dev → GitHub Pages
- [ ] 7. Verify website is live at rusticplayground.dev

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

## Strategic Reframe — 2026-04-21 Evening

After r/learnrust landed quiet and digesting a sharper framing from a parallel
thinking session — *"HN + Product Hunt + owned channels + direct outreach carry
launch weight; Reddit rides the wave 2-3 weeks later after genuine community
cred is built"* — the remainder of this week's plan is restructured:

- **Show HN is the primary launch moment** (Tue 4/28). It's the anchor day,
  not the cascade climax.
- **Reddit is supporting, not primary.** r/tauri still worth doing (permissive
  showcase sub). **r/rust dropped** — karma gap too large; defer to Month 3+
  after genuine community participation builds cred.
- **Product Hunt deferred to Mon 5/11** — worth doing right, not cramming
  alongside HN.
- **Owned channels (email list) and direct outreach (press, podcasts, creators)
  added as parallel tracks** — previously absent from the plan; the missing
  pillars of the reframe.
- **Orientation:** Rustic Playground launch is also a live testbed for
  distribution strategies that apply to future products. Not every tactic
  needs to succeed — outcomes are data. This frame reduces pressure and
  makes the plan more honest.

---

## Schedule Slide — 2026-04-22 (original) + 2026-04-24 (full-week push)

**2026-04-22:** Foundation work shifted from Wed 4/22 → Thu 4/23 (life).

**2026-04-24 (this revision):** Full +1 week push. Personal-life load made
it the wrong week to launch. No change to the sequence or content — just
dates. Foundation work (done 4/23) carries forward. Everything from
r/tauri onward shifts by exactly 7 days:

- r/tauri: Fri 4/24 → **Fri 5/1**
- Ecosystem carpet-bomb: Sat 4/25 → **Sat 5/2**
- Rest: Sun 4/26 → **Sun 5/3**
- HN prep: Mon 4/27 → **Mon 5/4**
- Show HN anchor: Tue 4/28 → **Tue 5/5**
- Product Hunt: Mon 5/11 → **Mon 5/18**

TWiR-with-HN timing alignment still works (Sat 5/2 submission → Tue 5/5
issue → same-day amplification with Show HN).

---

## Launch — Day 2 (Thu 2026-04-23) — Foundation

Three "owned channels" pillars: email capture, direct outreach list, SEO
foundation. All foundational, all should be in place before Show HN traffic
arrives Tuesday.

- [ ] 10. **Set up email capture on rusticplayground.dev** (~45 min)
  - Buttondown free tier (≤100 subscribers free, indie-friendly)
  - Add signup form to `docs/index.html` — likely near the hero CTA or footer
  - Two frames: *"Notify me of Rustic Playground updates"* + *"Be first to know
    when Rustic Zig ships"* (captures Zig-curious too)
  - Rationale: every visitor from now through HN/PH launches is a capturable
    email; list compounds for future launches (especially Rustic Zig)

- [ ] 11. **Assemble press / podcast / creator outreach list** (~1–2h)
  - Rust / systems-lang / macOS-dev podcasts (e.g., Rustacean Station, Filter
    Podcast if still active, MacGenius, Dev Tool Deep Dives)
  - Indie-dev / developer-tool journalists (TechCrunch-lite tier: The Register
    dev beats, MacStories, Daring Fireball quick-link)
  - Smaller Mac-dev creators reviewing developer tools on YouTube / Bluesky
  - Rust newsletter authors (This Week in Rust is step 12, but there are
    several Substack-era Rust newsletters worth pinging)
  - Target: 15–20 names + their contact channels (email or DM)
  - Output: simple Google Sheet or local markdown — name, channel, why they'd
    care, contact, sent/not-sent status

- [ ] 11a. **SEO foundation for rusticplayground.dev** (~1h technical + 30 min submissions)
  - **Why now:** Google isn't finding the site. New domains hit a "sandbox"
    effect (2-4 weeks minimum natural indexing); without Search Console
    submission and proper crawl guidance, that window stretches longer. Show
    HN spike will drive traffic; long-tail SEO captures the trickle that
    follows for months/years.
  - **Technical pieces** (can be done in this session — Claude implements):
    - [ ] Create `docs/robots.txt` — explicit crawl guidance, points to sitemap
    - [ ] Create `docs/sitemap.xml` — lists home + blog post + future pages
    - [ ] Add `<link rel="canonical">` to home page meta
    - [ ] Add Twitter card meta tags (`twitter:card`, `twitter:title`, etc.) to
      both home and blog
    - [ ] Add JSON-LD structured data (Schema.org `SoftwareApplication`) to home
      page — helps Google understand what the page IS (a downloadable Mac app)
  - **Submissions** (require user action — Google + Microsoft accounts):
    - [ ] Submit to Google Search Console at [search.google.com/search-console](https://search.google.com/search-console)
      - Add property `rusticplayground.dev`
      - Verify ownership (TXT record at Namecheap OR HTML file upload)
      - Submit sitemap once verified
    - [ ] Submit to Bing Webmaster Tools at [bing.com/webmasters](https://www.bing.com/webmasters)
      - Same flow; Bing covers Bing + DuckDuckGo + ~10% of US searches
  - **Quick wins for backlinks** (low effort, high SEO value):
    - [ ] Add a one-line "Featured in" block to README.md if/when posts go up
    - [ ] Make sure GitHub README links to rusticplayground.dev prominently
    - [ ] dev.to article (Friday's task) — adds quality backlink

- [ ] Check r/learnrust post state one final time (accept whatever it is)

---

## Launch — Day 3 (Fri 2026-05-01)

- [ ] 12. **r/tauri** — Tauri-dev framing, drafts at `specs/reddit-drafts.md`
  - Tauri devs, not Rust learners. Lead with stack + gotchas.
  - Small, permissive, showcase-friendly sub — Tier 3 in the reframe
  - Target: ~9–10am ET; 4–6h at keyboard for comments after posting
  - Can reference r/learnrust thread for continuity (not trust signal
    if that post stayed quiet)

- [ ] 13. **Send first 5–10 press / creator pitch emails** (parallel track)
  - Short, personal, specific. Not a mass blast.
  - Template: "I built X, I think it'd fit your [specific coverage / audience],
    here's a DMG, here's why I thought you'd care specifically."

---

## Launch — Day 4 (Sat 2026-05-02) — Ecosystem Carpet-Bomb

Multiple permissive venues in one day. Reuse r/tauri copy with tweaks for each.
Low cognitive load per venue. Saturday timing is mostly submission-style work
(not heavy posting), and **TWiR submission Saturday lands in the Tuesday 5/5
issue — same-day amplification with Show HN.**

- [ ] 14. **This Week in Rust** submission — [this-week-in-rust.org](https://this-week-in-rust.org)
  - **Critical timing:** Sat submission = Tue 4/28 issue = same-day boost as Show HN
  - Don't miss the Sun deadline

- [ ] 15. **Rust Users Forum** — Showcase category at [users.rust-lang.org](https://users.rust-lang.org). Detailed write-up OK here.

- [ ] 16. **Tauri Discord / Zulip** — #showcase channel, short friendly announcement

- [ ] 17. **dev.to article** — repurpose the origin-story blog post
  (`docs/blog/why-i-built-this/index.html`). Canonical link back to
  rusticplayground.dev. Zero additional writing cost.

- [ ] Follow up on first press/creator pitches; send second batch if needed

- [ ] **Optional: small v0.3.7 release** (same as before, but lower priority
  now that Show HN is the anchor). One commit, one tag, one GitHub release.
  Shows repo activity ahead of HN. Skip if energy is needed elsewhere.

---

## Launch — Day 5 (Sun 2026-05-03) — Rest / Buffer

- [ ] Rest. Don't post anything new. Protect energy for Tuesday.
- [ ] Optional: polish Show HN post (`specs/show-hn-draft.md`) — re-read aloud,
  trim, tighten.
- [ ] Optional: check email signups, respond to any outreach replies that came in
- [ ] Optional: glance at Cloudflare dashboard for weekend traffic patterns

---

## Launch — Day 6 (Mon 2026-05-04) — Show HN Prep Day

- [ ] ~~18. r/rust~~ — **DROPPED.** Karma gap too large for a zero-history
  account; would likely be mod-filtered or buried. Defer to Month 3+ after
  genuine Rust community participation builds cred.

- [ ] 18. **Final Show HN post polish** — re-read draft aloud, trim, tighten
  - Title locked before posting (HN doesn't let you edit titles)
  - Verify all links work, hero screenshot displays in HN preview
  - Test the first-self-reply comment copy with ongoing community thread URLs

- [ ] 19. **Verify launch-day assets** working
  - rusticplayground.dev loads fast
  - DMG download link works
  - Demo videos on site play correctly
  - GitHub Discussions visible and welcoming

- [ ] 20. **Prep "ask list"** — people to alert about the HN post after it goes live
  - LinkedIn connections who engaged with launch-day post
  - Tauri Discord / Rust Users Forum threads where you already posted
  - Email list subscribers (if the list has anyone by Monday)

---

## Launch — Day 7 (Tue 2026-05-05) — THE ANCHOR DAY

- [ ] 21. **Hacker News Show HN** (draft at `specs/show-hn-draft.md`)
  - Target time: **8–10am ET** (peak HN traffic, most front-page activity)
  - Title: *"Show HN: Rustic Playground — Swift Playgrounds, but for Rust"*
  - Reference ongoing community threads in first self-reply comment
  - **FULL-DAY COMMITMENT** — reply to every comment, stay engaged all day
  - No multitasking. This is the day.

- [ ] 22. **Amplify on LinkedIn** once HN post goes live (not before)
  - Short post: "Launched on Show HN — [link]. Thoughts welcome."

- [ ] 23. **Short announcement in Tauri Discord** with HN link

- [ ] 24. **Email list subscribers** get a "Show HN is live" note (if list exists)

- [ ] 25. **Twitter/X** — SKIP. You don't use it; forcing a tweet is
  inauthentic and zero ROI without a following.

---

## Post-launch (Week 2+, 2026-04-29 onward)

### Mon 2026-05-18 — Product Hunt launch

- [ ] Assemble PH assets — tagline, gallery images, demo video, first-comment
  copy, tagline variations
- [ ] Line up a hunter (established PH user to submit on your behalf; optional
  but lifts ceiling)
- [ ] Pre-launch notification list — email subs + LinkedIn + Tauri Discord
- [ ] Launch day — full-day commitment, similar to Show HN

### Ongoing — slow-burn infrastructure

- [ ] **Blog cadence** on jagmeet.dev/blog — 1 post every 2-3 weeks
  - Topic ideas: Tauri 2 gotchas (can repurpose r/tauri content), how Welcome
    Wizard works, shipping notarized DMGs in 2026, the journey back to
    hands-on coding after years of management
- [ ] **Community participation** — pick ONE (Tauri Discord or Rust Zulip)
  - Show up weekly, help newcomers, answer questions
  - No self-promotion for first 2-3 months; build presence as a participant
  - After 3 months: r/rust karma will be there + Lobsters invitation possible
- [ ] **OSS contributions** — small PRs to Rust/Tauri ecosystem as you encounter
  real bugs. Natural, not forced.

### Post-launch monitoring

- [ ] Monitor GitHub Issues + Discussions for feedback
- [ ] Update rusticplayground.dev links to point to HN / PH / active threads
  as they land
- [ ] Iterate on product based on feedback — prioritize bugs, then requests
- [ ] Track in `project_launch_traction.md` memory: what channels drove what
  traffic, which tactics produced what outcomes (this is the "testing for
  future products" data)

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
