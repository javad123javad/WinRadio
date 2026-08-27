---
title: WinRadio UX Reconciliation — vs. brief.md
status: draft
created: 2026-08-25
---

# Reconciliation: DESIGN.md / EXPERIENCE.md vs. brief.md

Scope of this pass: not FR-level detail (that's the PRD's job) — this checks whether the brief's
*qualitative* content (tone, the "relaxed personal project" framing, explicit non-goals, the
"no onboarding" constraint, the specific broken-scaffold evidence) survived into the UX spines
where it's relevant to UX, given that a spine focused on visual/interaction mechanics can easily
drop this kind of thing.

## What survived well (confirmed, not gaps)

- **"Personal tool, not a product" tone.** Brief: "single-user passion project... no audience to
  please but him." DESIGN.md's Brand & Style: "utilitarian dashboard, not consumer-app polish —
  this app has exactly one user and doesn't need to sell itself." EXPERIENCE.md's Voice and Tone:
  "a personal tool talking to its one user, not a product talking to a customer," with a concrete
  do/don't copy table reinforcing it. Well carried.
- **"No onboarding" constraint.** Brief: "no onboarding flow needed — it opens straight into
  'search or pick a favorite and hit play.'" EXPERIENCE.md's Foundation: "no auth, no
  multi-profile, no onboarding flow — the app opens straight into its one screen," and the
  Information Architecture table explicitly rejects separate Favorites/Now-Playing pages *because*
  they'd "reintroduce... unnecessary navigation surface that fights the brief's explicit
  'no onboarding flow'... constraint." Directly and explicitly carried, including the reasoning.
- **"Reliable, invisible, never worry if it's broken" vision.** Brief's Vision: "invisible in the
  tray until he wants it, reliable enough that he never thinks about whether it'll still be broken
  tomorrow." This shows up concretely in EXPERIENCE.md's State Patterns (Reconnecting, Stream
  failed, Info Tile degraded, Offline) and in DESIGN.md's Do's and Don'ts ("never let a
  weather-API outage look like the app itself is broken"). Good translation of a feeling into
  concrete states.

## Gaps found

### 1. UX spines quietly assume "repair the existing scaffold," which the brief explicitly left undecided

**Brief evidence:** "Open question, deliberately not decided here: whether to repair the existing
broken scaffold or start the codebase fresh. `[ASSUMPTION: left open — this is an
implementation-strategy call, not a product-scope call, and belongs to the architecture phase...]`"

**Spine evidence:**
- EXPERIENCE.md, Foundation: "Tailwind CSS utility classes (already present in the existing
  codebase — no component library like shadcn is in use; Tailwind is styled directly against
  `DESIGN.md` tokens)."
- EXPERIENCE.md, Information Architecture table, Settings row: "Modal overlay (matches the
  existing `SettingsModal.vue` already in the codebase)."

Both statements treat the current (per the brief, non-compiling, half-wired) scaffold as the
build foundation — which pre-empts the "repair vs. start fresh" call the brief and PRD (Open
Question 1, Open Question 3) both explicitly reserve for the architecture phase. This isn't
necessarily wrong, but it's a decision the brief said not to make yet, made implicitly by a
downstream document. Worth an explicit flag so the architecture phase isn't silently boxed in by
UX-stage file references.

**Recommendation:** Either soften these two references to "if the existing scaffold is retained"
framing, or explicitly note in EXPERIENCE.md that these are provisional pointers pending the
architecture-phase repair-vs-rebuild decision.

### 2. Brief's explicit non-goals aren't cross-referenced as anti-patterns the way reference-image cuts are

**Brief evidence:** "Explicitly out of scope: Recording streams to file; Equalizer / audio
effects; Casting to other devices (Chromecast etc.); Accounts, sync across machines, mobile
companion app."

**Spine evidence:** EXPERIENCE.md's "Inspiration & Anti-patterns" section is exactly the right
place for this and already does it well for *reference-image-derived* temptations — it explicitly
rejects a fourth Info Tile, a hover-reveal favorite star, and separate Favorites/Now-Playing
pages, each with a one-line rationale. But it never restates the brief's own out-of-scope list
(no EQ/visualizer, no cast button, no record button, no account/sync UI) even though a
Metro-style "Internet Radio" reference app of that era plausibly invites exactly these
affordances (EQ sliders, a cast icon, an account/sign-in tile). A future implementer working from
the UX spine alone — without re-reading the brief — has no guardrail against adding them.

**Recommendation:** Add 2-4 short entries to EXPERIENCE.md's Anti-patterns list mirroring the
brief's explicit non-goals (no equalizer/visualizer control, no casting affordance, no
record/save-stream control, no account/sign-in surface), the same way the reference-image cuts
are already handled.

### 3. (Minor) Accessibility Floor and Responsive & Platform sections are more rigorous than the brief's tone might call for

**Brief evidence:** "no roadmap pressure, no audience to please but him," and the app's success
bar is purely "does Javad use it" — no accessibility or multi-user requirement is ever raised.

**Spine evidence:** EXPERIENCE.md adds a full keyboard-operability floor (Tab order, focus rings,
Esc-to-close, WCAG AA contrast targets) and window-resize/minimum-width rules. The doc
self-justifies this ("costs little now"), and it isn't wrong to include — but it's UX-invented
scope with no brief or PRD NFR requiring it, for an app whose own brief frames it as "mouse-first"
personal software with an audience of one. Not a contradiction, just worth a conscious check that
this doesn't quietly grow the build.

**Recommendation:** No change required — flagging only so the added scope is a deliberate choice,
not a drift nobody decided on.
