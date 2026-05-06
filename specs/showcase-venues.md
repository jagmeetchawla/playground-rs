# Showcase-Friendly Venue Strategy

> **Status:** Strategic framing doc (v2, updated 2026-04-21 evening).
> Tactical week-by-week schedule lives in `specs/release-plan.md`.
> This doc captures the *why* behind venue tiering and cadence.

---

## Why this doc exists

The r/learnrust post (2026-04-21) landed quiet — ~1K feed impressions, 0–1
upvotes, 0 comments after hours. Evening traffic didn't dramatically change
the picture. During the lull, two things crystallized:

1. **Reddit isn't the primary launch engine for a cold-launch dev tool.**
   The 10:1 self-promo rules, karma gates, and `/new` fuzzing on major subs
   (r/programming, r/learnprogramming, r/cpp, r/rust for a new account) mean
   those channels are effectively closed until genuine community participation
   builds cred — a 2-3 month slow investment.

2. **The real engines are HN + Product Hunt + owned channels (website,
   email list, blog) + direct outreach (press, podcasts, creators).**
   Reddit rides that wave 2-3 weeks later, not the other way around.

The implication: **narrow to venues where showcase is welcome, tier them
by primacy, and treat this launch as a live testbed for strategies we'll
reuse on future products** (Rustic Zig, and beyond). Not every tactic has
to succeed — outcomes are data for the next launch too.

---

## Tiered venue framework

### Tier 1 — Primary launch engines

These carry the weight. Cold-launch success routes through these, not
through Reddit.

- **Show HN** — the single highest-leverage post type for dev tools.
  Welcomes showcase by design. No karma gate. Rewards craft + substance.
  A successful Show HN drives 10K+ visits in a day and lasting GitHub
  star momentum. **This is the anchor launch moment.**
- **Product Hunt** — equal-tier with HN for dev tools. Monday launches
  are convention. Requires dedicated prep (gallery, tagline, first
  comment, hunter). A distinct launch moment, not a same-day double.
- **Owned channels** — your website, blog, and email list. Compound over
  months. Without an email list, every visitor leaves un-capturable,
  which means each launch starts from zero. Email capture is foundational.

### Tier 2 — Direct outreach (parallel track)

Often overlooked, surprisingly effective. Personal, specific outreach
to 15-30 individuals rather than 1 mass post.

- **Podcast hosts** in your space (Rust, macOS dev, indie tooling)
- **Dev-tool journalists** (TechCrunch dev beat, The Register, MacStories)
- **Smaller creators** reviewing dev tools on YouTube / Bluesky / blogs
- **Newsletter authors** — not just major ones; niche newsletters have
  high-intent audiences and are easier to land in
- **Key figures in the community** whose nod amplifies reach (e.g.,
  Andrew Kelley for Zig-adjacent work, core Rust team members for Rust
  work — approached thoughtfully, not as promotion)

### Tier 3 — Permissive showcase subs / channels

Smaller, welcoming, low-karma-gate venues. Ride the HN/PH wave.

- **r/tauri** — tiny but on-brand, welcomes stack + gotchas posts
- **r/learnrust** — already posted; quiet but harmless
- **Rust Users Forum** (users.rust-lang.org) — Showcase category
  explicitly welcomes this
- **Tauri Discord / Zulip** — #showcase channels by design
- **Rust Zulip** — announce after genuine participation, not as entry point
- **Lobsters** — invitation-only, earned through community participation
  over months
- **dev.to** — self-publish, cross-post, canonical link back to site
- **This Week in Rust** newsletter — one-time mention, high signal
- **Indie Hackers** — journey + product combo posts welcome

### Tier 4 — Karma-gated subs (deferred to Month 3+)

Can't enter cold. Require genuine community participation first.

- **r/rust** — big sub, adversarial toward new-account self-promo.
  Worth earning access to (200+ karma from comments helps). Month 3+.
- **r/programming, r/learnprogramming, r/cpp, r/coding** — 10:1 rules,
  enforced. Save for months after consistent non-promo participation.

**These aren't lost opportunities.** They're traps for cold-launchers.
Save them for after 2-3 months of consistent commenting (not posting)
in the sub. When you finally do post, the 10:1 math works and mods
see you as a community member.

---

## Cadence — slow burn over spike

- **This week (4/21–4/28):** tactical plan in `specs/release-plan.md`.
  Summary: Show HN on Tue 4/28 is the anchor. Wed 4/22 = email list +
  outreach list. Thu = r/tauri. Fri = ecosystem carpet-bomb. Sat-Sun =
  rest. Mon = HN prep. Tue = HN launch day.
- **Weeks 2-4:** Product Hunt on Mon 5/11. Educator outreach via
  daughters at GT/Berkeley/Columbia. Blog cadence begins.
- **Month 2+:** 1 substantive venue post per week max. Mostly organic
  growth through search, word-of-mouth, GitHub discovery. Community
  participation in chosen Discord/Zulip is weekly.
- **Month 3+:** Reddit re-entry becomes possible (big subs). Second
  product launch preparation (Rustic Zig) leverages the email list
  built in Month 1.

---

## Orientation for this launch

**This launch is a live testbed for distribution strategies we'll
reuse.** Rustic Playground works with or without a viral moment. Product
compounds through quality + search + word-of-mouth regardless of Week 1
numbers. The learning is the point.

Specifically we're testing:
- Whether HN-anchored launch works for the positioning ("complement to
  play.rust-lang.org")
- Whether direct outreach (press, podcasts, creators) produces more
  returns than Reddit at this scale
- Whether owned channels (email list) produce real future leverage
- Whether the 2-3 month community-cred investment actually unlocks
  the karma-gated subs

All of this applies directly to Rustic Zig (next launch), v0.4 of
Rustic Playground, and any future product. **Even "failure" here is
data worth having.**

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

- **Blog cadence on jagmeet.dev/blog** — 1 post per 2–3
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
