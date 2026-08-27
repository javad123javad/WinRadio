---
title: WinRadio UX Reconciliation — vs. prd.md
status: draft
created: 2026-08-25
---

# Reconciliation: DESIGN.md / EXPERIENCE.md vs. prd.md

Scope: every FR (FR-1 through FR-14), every Glossary term, and every NFR checked for UX coverage.
Per instructions, the Now-Playing Dashboard / Info Tile scope expansion (§6.1 of the PRD) is
already correctly reflected in the spines and is *not* flagged here as a gap.

## FR-by-FR coverage

| FR | Coverage | Note |
|---|---|---|
| FR-1 Search + filters | Partial | Text search covered (Search field row); genre/country/language filter combination has no concrete UI — see Gap 1 |
| FR-2 Preview a result | Covered | Station row click-to-play, both list states |
| FR-3 Manage favorites | Partial | Add/remove (star) covered; reorder only mentioned in passing — see Gap 5 |
| FR-4 Play from list | Covered | Station row |
| FR-5 Transport (play/pause/resume/volume/mute) | Partial | Play/pause/volume covered; explicit mute control unaddressed — see Gap 6 |
| FR-6 Now-playing metadata (name + live track/title) | Partial | Station name has a typography role; live track/title text has none — see Gap 3 |
| FR-7 Location Tile | Covered (at spine level) | Generic Info Tile shell + degraded state; per-tile visual detail correctly deferred to key-screen mocks |
| FR-8 Weather Tile | Covered (at spine level) | Same as above |
| FR-9 Stream Info Tile | Gap | "Resolved server IP" field never appears; DESIGN.md's typography example substitutes "coordinates" — see Gap 2 |
| FR-10 Tray presence | Covered | IA table + Tray row |
| FR-11 Sleep Timer | Gap | No dashboard/transport control, no active-timer state, no cancel affordance — see Gap 4 |
| FR-12 Persistent state | Mostly covered | Volume-persist-on-release and cold-load-from-local-storage called out; last-played-station restore and theme restore not explicitly stated |
| FR-13 Settings panel | Covered | IA row + "Settings saved" state |
| FR-14 Dark/light theme toggle | Gap | Tokens fully defined in DESIGN.md; no nav icon, no Settings-modal line item, no component ever says where the user flips it — see Gap 7 |

## Gaps found

### 1. FR-1's genre/country/language filter has a nav icon but no interaction pattern

PRD FR-1: "Genre/country/language filters can combine with a text query or be used alone to
browse." EXPERIENCE.md's Component Patterns table only says: "Filter refines whichever is
currently showing rather than being its own destination" — no widget type (dropdown, chip row,
side panel), no indication of how genre/country/language combine visually, and no state pattern
for "filters active." A concrete FR with combinable multi-field filtering has a nav icon and
nothing else.

### 2. FR-9's "resolved server IP" is missing from both spine documents; DESIGN.md's example conflates it with Location Tile data

PRD FR-9: Stream Info Tile shows "codec, bitrate, country, and the resolved server IP of the
stream." DESIGN.md's Typography section, describing `{typography.label-caps}`, says it "handles
dense technical readouts (Info Tile stats: codec, bitrate, **coordinates**)" — coordinates belong
to the Location Tile (FR-7), not Stream Info (FR-9). The IP field FR-9 actually asks for is not
mentioned anywhere in DESIGN.md or EXPERIENCE.md. This reads as a drafting slip that also signals
the Stream Info Tile's content wasn't independently checked against FR-9's field list.

### 3. FR-6's live track/title metadata has no assigned component or typography treatment

PRD FR-6: "Dashboard shows the current Station's name **and, when the stream provides it, the
live track/title metadata**." DESIGN.md's Typography section assigns `{typography.display}` to
"the current station name" only — no style, placement, or truncation/wrap behavior is specified
for the live track/title text, and EXPERIENCE.md's Component Patterns table has no row for it at
all (only Station row, Info Tile shell, Transport bar, Nav icon-button, Search field are listed).
Half of FR-6 is unaddressed.

### 4. FR-11 Sleep Timer — a full Glossary term and its own feature section — has almost no UI presence

PRD Glossary: "**Sleep Timer** — A user-set duration after which playback auto-stops." FR-11:
"User can set a duration after which playback automatically stops." The only place Sleep Timer
appears in either spine is EXPERIENCE.md's Information Architecture table, and there only as
"Sleep Timer *defaults*" configured inside the Settings modal (FR-13's job, not FR-11's). FR-11
itself — the act of setting/starting/cancelling an active timer during a session — has:
- no nav icon or dashboard control,
- no Component Patterns row,
- no State Patterns row for "Sleep Timer running" (e.g., a countdown, or *some* visible
  indication the timer is armed) or for what happens/what the user sees when it fires.

This is the largest concrete gap found — a named, numbered FR with a dedicated Glossary entry is
effectively represented by one adjective ("defaults") in an IA table cell.

### 5. FR-3's reorder requirement is only mentioned as an aside in the "Banned" list

PRD FR-3: "User can add a Station to Favorites, remove it, and **reorder** the Favorites list."
The only place reordering is addressed is EXPERIENCE.md's Interaction Primitives → Banned:
"no drag-to-reorder gestures beyond simple up/down reorder controls for Favorites... a simpler
move-up/move-down affordance is fine." That's a constraint on *how not* to build it, not a
component spec — there's no Component Patterns row, no visual treatment in DESIGN.md's
`station-row` definition (which only defines play-icon + name + hover state), and no note on
whether the up/down controls are always-visible (per the doc's own no-hover-only rule) or where
they sit relative to the favorite star.

### 6. FR-5's "mute" is not distinguished from "volume at zero"

PRD FR-5: "User can play, pause, and resume the current Station, and adjust/**mute** volume."
DESIGN.md's `transport-bar` component and EXPERIENCE.md's Component Patterns both describe only a
volume slider ("Volume slider updates live on drag, persists on release"). Neither mentions a
discrete mute toggle/icon, so it's unclear whether FR-5's "mute" is meant as a dedicated
one-click affordance (typical) or is expected to be inferred as "drag to zero" (which would lose
the pre-mute volume level unless a separate mechanism restores it).

### 7. FR-14's dark/light toggle has full token support but no UI location

PRD FR-14: "User can toggle between dark and light themes; the choice persists (FR-12)." DESIGN.md
fully defines a parallel light-mode token set (`surface-base-light`, `on-surface-light`, etc.) and
EXPERIENCE.md's Foundation and Accessibility Floor sections both reference "light-theme
equivalent" contrast pairs — but no document says where the toggle control lives. It's not in the
nav-icon-button list (Home/Favorites/Filter/Search), and the Settings modal's IA row only lists
"tray behavior and Sleep Timer defaults" as its contents, omitting the theme toggle. A fully
specified visual system for a control that has nowhere to live.

## Glossary term-usage check

Checked: Station, Directory, Favorite, Station List, Now-Playing Dashboard, Info Tile, Sleep
Timer, Tray.

- **Station, Directory, Favorite, Now-Playing Dashboard, Info Tile, Tray** — used consistently and
  identically to the PRD Glossary throughout both spine documents. No synonyms found.
- **Sleep Timer** — used correctly where it appears, but appears so rarely (see Gap 4) that
  consistency can't be meaningfully evaluated beyond the one IA-table mention.
- **Station List — minor drift.** The Glossary defines "Station List" as the canonical name for
  the persistent left-hand rail. EXPERIENCE.md introduces it correctly once ("Station List rail
  (Favorites by default)" in the IA table; "Station List rail (both Favorites and Search states)"
  in Component Patterns) but then refers to it as bare "**the rail**" everywhere else — "the rail
  swaps from Favorites to live search results," "Rail shows a short prompt," "the rail doesn't
  change what's showing" (Flow 1, step 4), etc. DESIGN.md never uses the phrase "Station List" at
  all — only `spacing.rail-width` and the `station-row` component name. This is consistent
  *internally* (both docs settle on "rail" as shorthand) but it is a de facto synonym for the
  PRD's Glossary term, and a component or file later named `Rail.vue` rather than
  `StationList.vue` would be a natural (if harmless) consequence. Low severity, but exactly the
  kind of drift the Glossary-consistency check is meant to catch.
- **"Home" nav icon — undefined, not a Glossary term.** DESIGN.md's nav-icon-button component and
  EXPERIENCE.md's Component Patterns both list a "Home" icon (inherited directly from the
  reference image) alongside Favorites/Filter/Search, but no PRD FR or Glossary term maps to
  "Home," and EXPERIENCE.md never states what it does that's distinct from Favorites — the
  Component Patterns row says "Only one of Home/Favorites/Search is 'active' at a time," implying
  Home is a separate state, but never defines what content it shows. Not a Glossary-consistency
  issue per se, but worth flagging alongside it since it's an undefined term riding along with the
  correctly-defined ones.

## NFR coverage

- **NFR-3 (graceful degradation)** — Concrete, not just prose. Info Tile degraded state (same
  shell, muted text, per-tile independence) and the Stream-failed transport-bar state both give
  NFR-3 an actual UI treatment, and DESIGN.md's Do's/Don'ts reinforces it ("never let a
  weather-API outage look like the app itself is broken"). No gap.
- **NFR-4 (offline handling)** — Partially concrete, one real gap. EXPERIENCE.md's State Patterns
  has an explicit "Offline (no network at all)" row, which is good — but it resolves offline
  search to the *same* copy as a genuine zero-results search: "No stations found for '{query}'."
  This satisfies NFR-4's literal bar ("not a crash or silent hang") but conflates two different
  user-facing conditions the PRD treats as distinct (FR-1's consequence calls out "Directory
  unreachable" specifically as needing "a clear inline error, not a silent empty list," implying
  it should read differently from a true no-match search). A user offline mid-search sees "No
  stations found for 'jazz'" and has no way to tell that it's a connectivity problem rather than a
  bad query — undercutting the "clear state" goal of both FR-1 and NFR-4.
- **NFR-1 (Platform), NFR-2 (Performance), NFR-5 (Privacy)** — Not UX-surface concerns in the way
  NFR-3/NFR-4 are (platform/perf/privacy are largely invisible to the UI); no gap to flag at the
  UX-spine level.

## Summary of gaps (ranked)

1. Sleep Timer (FR-11) — essentially no UI representation (largest gap).
2. Stream Info Tile's resolved IP (FR-9) — missing entirely, conflated with "coordinates" in
   DESIGN.md.
3. Live track/title metadata (FR-6) — no assigned component or typography treatment.
4. Theme toggle (FR-14) — full token system, no control location specified.
5. Filter-by-genre/country/language (FR-1) — nav icon named, no interaction pattern.
6. Offline vs. zero-results conflation (FR-1 consequence / NFR-4) — same copy for two different
   conditions.
7. Favorites reorder (FR-3) — mentioned only as a "Banned"-list aside, no component spec.
8. Mute (FR-5) — not distinguished from volume-at-zero.
9. "Station List" vs. bare "rail" — minor Glossary-term drift.
10. Undefined "Home" nav icon — not PRD-mapped.
