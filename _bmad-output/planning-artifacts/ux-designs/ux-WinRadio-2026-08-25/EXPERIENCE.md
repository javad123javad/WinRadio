---
title: WinRadio Experience
status: final
created: 2026-08-25
updated: 2026-08-25
sources:
  - ../../prds/prd-WinRadio-2026-08-25/prd.md
  - ../../briefs/brief-WinRadio-2026-08-25/brief.md
---

# WinRadio — Experience Spine

DESIGN.md and this file win on conflict with any mock, wireframe, or import.

## Foundation

Windows 10/11 desktop only, single fixed-size window (no responsive breakpoints — see Responsive & Platform for the one exception). Chosen stack per the brief/PRD: Tauri shell, Vue 3 frontend, Tailwind CSS utility classes — no component library like shadcn is in use; Tailwind is styled directly against `DESIGN.md` tokens. `[NOTE FOR UX]` The current codebase already contains Tailwind and a rough `SettingsModal.vue`, but whether that scaffold is repaired or replaced is explicitly undecided (PRD Open Question 1) — this spine specifies the target experience independent of that call; nothing here presumes the existing files survive. Single-tenant, single-user: no auth, no multi-profile, no onboarding flow — the app opens straight into its one screen.

Terminology note: the Glossary's **Station List** is referred to as "the rail" throughout this document as an established shorthand for the same thing — never a different surface.

## Information Architecture

| Surface | Reached from | Purpose | Mock |
|---|---|---|---|
| Main Dashboard | App open (default) | Station List rail (Favorites by default) + Now-Playing Dashboard + the three Info Tiles. The entire app lives here — realizes UJ-1. | [mockups/main-dashboard.html](mockups/main-dashboard.html) |
| Search results | Search nav icon / typing in the search field | Station List rail swaps from Favorites to live search results (FR-1, FR-2). Same rail, same row component — content source changes, layout doesn't. | [mockups/search-state.html](mockups/search-state.html) |
| Settings | Settings/gear icon | Modal overlay (`DESIGN.md.components.settings-modal`) — tray behavior, Sleep Timer defaults, and the dark/light theme toggle (FR-13, FR-14). Whether it's built on the existing `SettingsModal.vue` or a fresh one is an architecture-phase call, not specified here. | [mockups/settings-modal.html](mockups/settings-modal.html) |
| Tray | Closing the main window | Not a visual surface — a system state. Icon exposes play/pause (FR-10); reopening the tray icon restores the Main Dashboard exactly as left. | *(none — not a visual surface)* |

Spines win on conflict with any mock — a mockup illustrates a decision already made in the tables above, it never introduces a new one.

There is deliberately no separate "Favorites" page and no separate "Now Playing" page — the reference image's nav icons (Favorites, Filter, Search) are read as *filters on the one rail*, not page navigation (see `DESIGN.md.Components` — Nav icon-button — for why "Home" from the reference isn't carried over). This keeps the single-window structure the brief and PRD both call for (no onboarding, no dead ends).

→ Composition reference: three key-screen mocks in `mockups/` (linked in the table above), covering all three visual IA surfaces.

## Voice and Tone

Microcopy stays plain and technical-neutral — a personal tool talking to its one user, not a product talking to a customer. Brand posture lives in `DESIGN.md.Brand & Style`.

| Do | Don't |
|---|---|
| "No stations found for 'jazz'." | "Oops! We couldn't find anything 😢" |
| "Reconnecting…" | "Uh-oh, having some trouble!" |
| "Weather unavailable" (quiet, in the tile) | "⚠️ Error: weather service timeout" |
| "3 favorites" | "You have 3 amazing favorites!" |

## Component Patterns

Behavioral. Visual specs live in `DESIGN.md.Components`.

| Component | Use | Behavioral rules |
|---|---|---|
| Station row | Station List rail (both Favorites and Search states) | Click anywhere on the row starts playback immediately (FR-4) — no separate "select then play" step. A star/favorite toggle sits at the row's right edge, click-target separate from the row-click-to-play area so favoriting never accidentally triggers playback of a different station. In the Favorites list only, up/down reorder controls (`DESIGN.md.components.station-row.reorder-control-color`) sit at the row's left edge, always visible, moving that row one position per click (FR-3) — not drag-based. |
| Info Tile shell | Now-Playing Dashboard (×3: Location, Weather, Stream Info) | One shared component, three content variants (FR-7/8/9). Each independently shows its own empty/degraded state (`DESIGN.md.components.info-tile`) without affecting the other two or blocking playback — this is the UI expression of PRD NFR-3. |
| Transport bar | Now-Playing Dashboard, persistent whenever a Station is loaded | Play/pause is a single toggle button, not two separate buttons. Volume slider updates live on drag, persists on release (FR-12). A separate mute icon-toggle sits beside the slider — muting and unmuting restores the exact prior volume, it does not just drag the slider to zero. No seek control — internet radio streams have no scrubbable timeline; the position display is elapsed-time-only, informational. |
| Sleep Timer control | Transport bar | Icon button opens a short duration picker (e.g. 15/30/60/90 min) pre-filled from the Settings-panel default (FR-13); confirming arms it (FR-11). Once armed, the icon shows a small active badge — the only dashboard indicator, deliberately understated. Clicking the badged icon opens the same picker with a "Cancel timer" option. When it fires, playback stops via the normal pause state — no special "timer fired" modal. |
| Nav icon-button | Top bar (Favorites / Filter / Search / Settings) | Toggles which content fills the rail (Favorites vs. Search results) or opens the Settings modal. Only one of Favorites/Search is "active" at a time; Filter refines whichever is currently showing rather than being its own destination. |
| Search field | Appears under the Search nav icon | Debounced live query against the Directory (FR-1) — no explicit "Go" button needed for the common case; Enter also submits immediately for fast typers. |
| Filter control | Appears under the Filter nav icon, alongside Search | A small panel (not a full page) with three independent selects — genre, country, language (FR-1) — any combination applies together, and combines with an active text search if one is present. Selections show as removable chips beneath the search field so it's visible *what's* filtering the current rail contents, not just that filtering is possible. |
| Settings modal | Opened from the Settings nav icon | Simple label + control rows (`DESIGN.md.components.settings-modal`): minimize-to-tray toggle, Sleep Timer default duration, dark/light theme toggle (FR-13, FR-14). Changes apply immediately on interaction — no separate "Apply" button, matching the no-explicit-save pattern used everywhere else in the app (FR-12). |

## State Patterns

| State | Surface | Treatment |
|---|---|---|
| Cold app load | Main Dashboard | Station List rail shows Favorites immediately from local persistence (FR-12) — no loading spinner needed for the rail itself, since it's local data. Now-Playing Dashboard shows an idle empty state until a station is picked. |
| No favorites yet | Main Dashboard (first run) | Rail shows a short prompt ("No favorites yet — search to find a station") instead of an empty box. |
| No search results (query has a real answer: zero matches) | Search state | "No stations found for '{query}'." No retry button needed — the user just edits the query. |
| No search results (connectivity is the actual cause) | Search state | Distinct copy per PRD FR-1/NFR-4's "clear state" requirement — not the same message as a genuine zero-match: "Can't reach the station directory — check your connection." Same visual slot as the zero-match message, different text, so the user knows whether to edit the query or check their network. |
| Stream reconnecting | Transport bar | "Reconnecting…" replaces the elapsed-time display; playback controls stay visible but disabled until it resolves or fails (realizes UJ-1's edge case, PRD FR-5). |
| Stream failed (after retry) | Transport bar | Clear inline message ("Couldn't play this station") using `DESIGN.md.colors.error` — the one place error red is appropriate, since this is a genuine playback failure, not a degraded Info Tile. |
| Info Tile degraded | Now-Playing Dashboard | Per-tile quiet placeholder text ("Weather unavailable," "Location unknown," "Stream info unavailable") in `DESIGN.md.colors.on-surface-variant` — never red, never blocks the other tiles or playback (PRD NFR-3). |
| Sleep Timer armed | Transport bar (Sleep Timer control) | Small active badge on the icon (`DESIGN.md.components.sleep-timer-control`) — no countdown text, no persistent banner. Firing simply pauses playback; no modal or notification. |
| Offline (no network at all) | Global | Search and Info Tiles show their respective unavailable states (see the two "No search results" rows above); already-loaded Favorites and playback of an already-buffering stream are unaffected until the stream itself needs new data (PRD NFR-4). |
| Settings saved | Settings modal | Closes on save with no separate confirmation toast — the modal closing *is* the confirmation. |

## Interaction Primitives

Mouse-first — this is a desktop dashboard app for one user, not a keyboard-power-tool. But baseline keyboard operability is kept from day one since it costs little now:

- `Space` — play/pause the current station, when the main window has focus and no text field is focused.
- `Tab` / `Shift+Tab` — moves focus through nav icons → rail → transport controls in visual order.
- `Enter` — activates the focused station row (same as click) or submits the search field.
- `Esc` — closes the Settings modal.
- Mouse: click station row to play, click star to favorite, drag volume slider.

**Banned:** no drag-to-reorder gestures beyond simple up/down reorder controls for Favorites (PRD FR-3 asks for reordering, not necessarily drag — a simpler move-up/move-down affordance is fine and easier to keep keyboard-accessible), no hover-only affordances for anything that gates a required action (the favorite-star must be visible, not hover-reveal-only, since a mouse-only "reveal on hover" pattern fights keyboard operability).

## Accessibility Floor

Behavioral. Visual contrast lives in `DESIGN.md`. `[NOTE FOR UX]` This section and Responsive & Platform below add more rigor than the brief's "relaxed, mouse-first, audience of one" framing strictly requires — a deliberate, logged choice (`.memlog.md`) because the cost is low at this stage, not a silent scope expansion. Fine to cut back later if it never earns its keep.

- Every interactive element (station row, nav icon-button, transport controls, Info Tile — if it ever becomes actionable) is reachable via `Tab` and shows a visible focus ring using `DESIGN.md.colors.primary`.
- `DESIGN.md`'s dark-theme text pairing (`on-surface` #f5f5f7 on `surface-base` #101012 / `surface-raised` #18181b) and light-theme equivalent (`on-surface-light` #1b1b1f on `surface-base-light` #fafafa) both estimate well above the WCAG AA 4.5:1 body-text floor — both pairings are near-max contrast by design. The one pairing to verify once real screens exist: `on-surface-variant` (#9a9aa2 dark / #5c5c66 light) against its raised surface, used for Info Tile empty-state text and captions — estimated ≥ 4.5:1 but not yet measured against rendered output. `[ASSUMPTION]` Ratios are estimated from the hex values, not measured with a contrast tool.
- No information is conveyed by color alone: the "Reconnecting…" / "Couldn't play this station" states use text, not just a red tint, and the active nav icon uses a filled background plus the icon staying visible, not a color shift alone.
- `Tab` order matches the visual reading order: nav bar → rail → transport bar → Info Tiles.

## Responsive & Platform

WinRadio is a fixed desktop window, not responsive in the web sense — but it can still be resized or maximized by the user, so one rule applies: below a minimum window width, the Info Tile grid drops from a 3-across row to a 2-across (Location + Weather) with Stream Info wrapping beneath, before anything is hidden outright. The Station List rail never collapses — if the window gets too narrow for the rail plus dashboard, the app enforces a minimum window width instead of hiding the rail (losing "come back to a favorite" access is worse than a slightly-too-wide minimum window).

## Inspiration & Anti-patterns

- **Lifted from the reference image** (Windows-8-era "Internet Radio" Metro app, shared by the user): the flat dark tile-dashboard layout, the left station-list rail with a live count, the large now-playing panel with artwork + transport bar, and the small supporting Info Tile grid. This is the structural and functional north star for the whole IA above.
- **Lifted from the reference image, deliberately not copied:** the language toggle (GER/ENG) in the top-left — dropped per PRD §6.2, since this is a single-user English-only tool. The "Pin to Start" bottom action is also dropped — that's Windows 8 charm-bar chrome with no equivalent in a modern Tauri tray app.
- **Rejected — a fourth or fifth Info Tile:** per `DESIGN.md`'s Do's and Don'ts and PRD's SM-C1 counter-metric, the tile count is fixed at three. Any future "just one more tile" idea should be treated as a scope-creep flag, not a free add.
- **Rejected — hover-reveal favorite star:** tempting for a cleaner row at rest, but it fights keyboard operability (see Interaction Primitives) and adds a discoverability tax for a single user who should never have to hunt for the one control they use most.
- **Rejected — a "Now Playing" or "Favorites" as separate pages:** would reintroduce the kind of unnecessary navigation surface that fights the brief's explicit "no onboarding flow, opens straight into search-or-favorite" constraint.
- **Rejected — anything the reference image's era invites but the brief explicitly excluded:** no equalizer/visualizer control, no "cast to device" affordance, no record/save-stream button, no account/sign-in surface. A Metro-style "Internet Radio" app of that period plausibly has all four; WinRadio's brief rules them out entirely (brief.md §Scope, "Explicitly out of scope"), so none should creep in via a UI screen that "just has a spot for it."

## Key Flows

### Flow 1 — Javad finds a station and comes back to it later

*Mirrors PRD UJ-1 verbatim — see `prd.md` §2.2.*

1. Javad opens WinRadio from the tray. The Main Dashboard restores exactly as he left it — Favorites rail populated from local storage, Now-Playing Dashboard idle.
2. He clicks the Search nav icon, types "jazz" into the search field. The rail swaps to live results as he types (debounced).
3. He clicks play on a couple of results directly from the rail — each click replaces the currently loaded station (Component Patterns → Station row) — previewing without needing to favorite first.
4. He finds one he likes and clicks its star to favorite it. The rail doesn't change what's showing (he's still viewing search results), but the station now also exists in the Favorites list underneath.
5. He closes the main window. Per Information Architecture, this doesn't quit the app — it minimizes to Tray; playback continues uninterrupted.
6. **Climax:** That evening, he reopens WinRadio from the tray. The rail shows Favorites by default, with the new station right there at the top — no re-searching. He clicks it, it plays immediately.
7. **Edge case:** Mid-session, the stream drops (station server hiccup). Per State Patterns, the transport bar shows "Reconnecting…" and WinRadio retries automatically — Javad sees a brief state change, not silence, and doesn't have to do anything himself unless the retry genuinely fails.

→ Steps 1 and 6 render as [mockups/main-dashboard.html](mockups/main-dashboard.html); step 2 as [mockups/search-state.html](mockups/search-state.html).
