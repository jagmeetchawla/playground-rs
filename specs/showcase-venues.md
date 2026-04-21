# Showcase-Friendly Venue Strategy (Draft)

> **Status:** Draft, written 2026-04-21 after r/learnrust post landed quiet.
> Purpose: narrow the distribution plan to venues where showcase is welcome,
> and adopt a slow-burn cadence over a spike. **For review — revisit tomorrow.**

---

## Why this draft exists

The r/learnrust post (2026-04-21, ~noon ET) drew ~1K feed impressions, 0–1
upvotes, 0 comments in the first 90 min. Evening ET traffic may improve the
numbers, but even the best case here is "okay, not great." During that lull
we realized:

- Many large dev subs (r/learnprogramming, r/programming, r/cpp) have 10:1
  self-promo rules we don't meet with an ~empty new Reddit account.
- Reddit's `/new` fuzzing + first-time-poster trust penalties make it hard
  to establish momentum without karma history, regardless of post quality.
- The staggered "launch week spike" framing puts unhelpful pressure on each
  post. Product is good; distribution doesn't need to be frantic.

**Revised posture:** narrow to venues where showcase is welcome, spread
posts over weeks (not days), lean into evergreen/organic channels that
compound.

---

## This week (remainder of 2026-04-21 to 2026-04-28)

**Keep:**

- [ ] **Thursday 2026-04-23 — r/tauri** (keep as-is from existing draft).
  Smaller, technical, welcomes showcase. Stack+gotchas framing fits.
  Draft already in `specs/reddit-drafts.md`.

- [ ] **Friday 2026-04-24 — This Week in Rust** submission.
  Explicitly welcomes community projects. High signal for Rust devs.
  Newsletter reach compounds.

- [ ] **Tuesday 2026-04-28 — Show HN**. HN is literally the "I built X"
  venue. Craft-and-substance posts fare well there when product holds up.
  Keep as the anchor launch post of the week. Draft at `specs/show-hn-draft.md`.

**Soft-reconsider:**

- [ ] **Monday 2026-04-27 — r/rust.** Originally the peak-traffic post.
  Reconsider tone:
  - If r/tauri (Thu) + TWiR (Fri) went well → keep, but update copy to
    reference those signals.
  - If r/tauri landed quiet too → substitute with **Rust Users Forum**
    (users.rust-lang.org, Showcase category) which explicitly welcomes
    showcase. No karma requirements, no self-promo ratio rules.

**Add (optional, as energy permits):**

- [ ] **dev.to blog post** — repurpose `blog/why-i-built-this` into a
  dev.to article with canonical link to the website. Zero additional
  writing cost. dev.to audience is dev-tool-curious.

- [ ] **Tauri Discord / Zulip announcement** — community is small but
  built for showcase. Short, friendly announcement in #showcase channel.
  Already planned per `project_tauri_angle.md` memory.

- [ ] **Indie Hackers product launch** — solopreneur/builder audience,
  different from Reddit/HN. Journey narrative welcome.

**Drop (or never had):**

- ❌ r/learnprogramming, r/programming, r/cpp, r/coding — all have 10:1
  rules or karma minimums. Cold-launching here is at best mod removal,
  at worst a ban. Not worth the risk.

---

## Week 2+ (2026-04-29 onward) — slow burn

The idea: stop thinking in spikes, start thinking in compounding.
2–3 touchpoints per week for a few months, each one designed to reach
a distinct audience slice. Some examples, in rough priority order:

### High-value, evergreen

- **Lobsters** — invitation-required but members invite thoughtfully.
  High-signal community. Mention of Rustic Playground via Rust/Tauri
  threads is natural. (Can't directly submit without invite; focus on
  being invited through commenting or a Rust/Tauri contributor vouching.)

- **This Week in Rust mentions** — one-time announcement post is the
  ceiling; ongoing mentions happen naturally if users star/fork.

- **Blog cadence on rustic-playground.app/blog** — 1 post per 2–3
  weeks keeps the site "alive" for SEO + gives you something to link to.
  Post ideas: "gotchas building a Tauri 2 app", "how the Welcome Wizard
  works", "shipping a code-signed + notarized macOS DMG in 2026".
  Each post is also potential dev.to / Hacker News content.

- **Awesome-rust / Awesome-tauri PRs** — meta-list inclusion drives
  slow but steady GitHub discovery. One-time effort, evergreen payoff.

### Community engagement (pre-promotional)

- **Rust Zulip** — active participation, answering questions, helps build
  community presence WITHOUT promotion. Months later, one-off "I made a
  thing" announcement lands better because you're a known participant.

- **Rust Discord #beginners** — same logic. Help learners with their
  Rust issues; occasionally mention the app when actually relevant.

- **r/rust karma buildup** — post thoughtful comments on others' threads
  for 2–3 months. When you finally post Rustic Playground, the 10:1-rule
  argument is neutralized.

### Niche showcase venues

- **r/macapps** — tiny but on-brand. Mac-specific audience.
- **r/SwiftPlayground** (and similar Swift Playgrounds subs) — positioning
  play already primes "familiar but for Rust."
- **Product Hunt** — once-only launch. Schedule for a Monday with advance
  notice to your network (LinkedIn, GitHub watchers). High ceiling, needs
  coordinated push.

### Educator outreach (already planned — `release-plan.md` Weeks 2–4)

- **Daughters at GT / Berkeley / Columbia** — warm intros to Rust-touching
  CS courses. Lowest-risk, highest-conversion option in the whole plan.
  See release-plan.md "CS Program Outreach" section.
- **Cold TA outreach at other Rust-teaching schools** — Stanford CS110L,
  Brown CSCI 1260, CMU systems courses. After daughters' threads give you
  a "pattern" that works.

---

## Venues to skip entirely (10:1 rules + karma gates)

- r/learnprogramming
- r/programming
- r/cpp
- r/coding
- r/webdev (wrong audience anyway)
- Any sub whose sidebar mentions "no self-promotion" or "karma minimum" without
  an explicit showcase exception

These aren't lost opportunities — they're traps. Posting there without
karma history gets you mod-removed, which costs goodwill and account
health. Save them for Month 3+ after you've built karma through thoughtful
non-promo commenting.

---

## Karma-building strategy (if you want to unlock those subs later)

Takes 2–3 months of consistent but small time investment:

1. **Pick 2–3 high-value subs** you want to eventually post in. Likely
   candidates: r/rust, r/programming, r/learnprogramming.
2. **Comment, don't post.** 3–5 thoughtful comments per week on others'
   threads in those subs. Answer questions, share experience.
3. **Build 100+ karma per sub** organically. No shortcuts; upvoted comments
   come from genuine helpfulness.
4. **After 2–3 months**, single self-promo post lands differently. You
   have account history, the 10:1 arithmetic works, mods see you as a
   community member.

This is a slow investment but it's the only real path into those subs.

---

## Cadence discipline

Launch fatigue is a thing. Rough rhythm to protect against it:

- **This week:** 1 substantive post per non-weekend day, max. Reply
  4–6h per post. Stop watching any single post after 24h.
- **Weeks 2–4:** 1 substantive post per 2–3 days. Interleave blog posts,
  Discord mentions, educator emails.
- **Month 2+:** 1 substantive post per week. Mostly organic, compounding
  through search + word-of-mouth + GitHub discovery.
- **Never:** delete-and-repost the same thing. Every venue gets one shot.

---

## Open questions to revisit (Wednesday 2026-04-22)

- How did evening ET traffic change r/learnrust reception? (Check ~9am ET Wed.)
- Does r/tauri Thursday feel worth it, or should we skip directly to
  Rust Users Forum + dev.to + Show HN?
- Is Product Hunt worth scheduling for a specific Monday in Week 2 or 3?
- What's the right first blog post for the website after the origin story?
- Should we accept the r/learnrust outcome and not pursue r/rust at all
  this week, saving it for Month 3 post-karma-building?

---

## Bottom line

The product is good. The distribution doesn't have to be frantic. Narrow
the venues, spread them out, lean into evergreen channels, and let the
slow compounding do its work. Educator outreach + GitHub organic discovery
+ consistent blog cadence will probably produce more sustainable traction
than any single launch-week spike anyway.
