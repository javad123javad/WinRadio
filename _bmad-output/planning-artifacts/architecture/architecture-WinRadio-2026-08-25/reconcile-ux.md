# Input Reconciliation — UX (DESIGN.md / EXPERIENCE.md) vs ARCHITECTURE-SPINE.md

Scope: WinRadio architecture finalize step. Checked every EXPERIENCE.md Component Patterns row, every IA surface, every State Patterns row (backend-dependent ones especially), and data-shape sufficiency (Station fields, Settings fields) against ARCHITECTURE-SPINE.md's Structural Seed, Capability→Architecture Map, and AD-4/AD-5 event model.

## 1. Component/surface → file mapping check

All EXPERIENCE.md Component Patterns rows and IA surfaces have a plausible home in the Structural Seed:

| EXPERIENCE.md item | Spine home | Verdict |
|---|---|---|
| Station row | `StationRow.vue` | OK |
| Info Tile shell (+3 variants) | `InfoTile.vue`, `LocationTile.vue`, `WeatherTile.vue`, `StreamInfoTile.vue` | OK |
| Transport bar | `TransportBar.vue` | OK |
| Sleep Timer control | Folded into `TransportBar.vue` per Capability Map row ("TransportBar.vue (Sleep Timer control)") | OK, but see Gap 3 below re: where its data (armed default) actually lives |
| Nav icon-button | `NavBar.vue` | OK |
| Search field | `SearchPanel.vue` | OK |
| Filter control (3 selects: genre/country/language) | `SearchPanel.vue` — comment says "search input + filter chips" | **Weak** — see Gap 5 |
| Settings modal | `SettingsModal.vue` | OK |
| Main Dashboard | `App.vue` + `StationList.vue` + `NowPlayingDashboard.vue` + `InfoTile.vue` grid | OK |
| Search results | `StationList.vue` content swap + `SearchPanel.vue` | OK |
| Settings | `SettingsModal.vue` | OK |
| Tray | `tray.rs` | OK |

No component or IA surface is structurally homeless. The gaps below are about whether the event model and data shapes actually back what the components need to *do*, not about missing files.

## 2. Gaps found

### Gap 1 — AD-4's event enumeration has no distinct "playback failed" event, but EXPERIENCE.md needs one

AD-4's rule: "Rust emits an event on every playback/metadata state change (**play, stop, reconnecting, metadata updated**)." That is the complete enumerated list.

EXPERIENCE.md's State Patterns table has two visually and semantically distinct transport-bar states:
- **Stream reconnecting** — "Reconnecting…" replaces elapsed time, controls stay visible-but-disabled. (Covered — `reconnecting` event exists.)
- **Stream failed (after retry)** — "Couldn't play this station" in `DESIGN.md.colors.error` — explicitly called out as "the one place error red is appropriate."

As written, a failed-after-retry stream and a normal user-initiated pause both appear to collapse onto the same `stop` event. There is no `failed`/`error` event variant in AD-4's list, so nothing in the spine tells the frontend how to distinguish "stopped because the user paused" from "stopped because retries were exhausted and it should render in error-red with 'Couldn't play this station.'" Sleep Timer firing deliberately reuses the plain pause/stop state (EXPERIENCE.md says so explicitly), which shows the spine's authors *did* think about state-reuse — but the reconnect-exhausted case needs the opposite treatment (a distinguishable error state) and isn't accounted for.

**Fix needed:** either add a `playback-error` (or similar) event/payload variant to AD-4's rule, or extend the `stop` event payload with an optional reason/error field the frontend can branch on.

### Gap 2 — AD-5's event model only describes the success path for Info Tiles, not the degraded/failure path

AD-5's rule: "Now-playing metadata, Location info, Weather info, and Stream info are each their own Tauri event, emitted independently **as each becomes available**."

EXPERIENCE.md's State Patterns requires each tile to show quiet degraded copy ("Weather unavailable," "Location unknown," "Stream info unavailable") when its respective fetch fails — not indefinitely spin. AD-5 as written only commits to emitting an event when data *becomes available*; it says nothing about emitting on failure/timeout. Since AD-4/AD-5 both forbid polling ("never polled"), and there's no fallback timeout mechanism specified anywhere in the spine, a tile whose backend fetch fails (weather API down, geolocation fails, ICY metadata absent) has no defined mechanism to ever leave a loading state and reach the "degraded" copy EXPERIENCE.md requires — unless failure is also expected to emit an event, which the rule doesn't say.

**Fix needed:** AD-5's rule should explicitly state that each of the four event types is emitted exactly once per attempt regardless of outcome (success or degraded-with-reason), so tiles have a defined, non-polled path out of a loading/idle state into the degraded copy.

### Gap 3 — No Settings read-model store, but two separate UI locations need shared access to Settings data

The Structural Seed lists exactly two Pinia stores: `stations.ts` and `playback.ts`. There is no `settings.ts` (or equivalent).

But Settings data is consumed in more than one place:
- `SettingsModal.vue` — the primary editor (minimize-to-tray toggle, Sleep Timer default duration, theme).
- The Sleep Timer control (inside `TransportBar.vue`) — EXPERIENCE.md: the duration picker is "pre-filled from the Settings-panel default (FR-13)," meaning it needs to read the same default duration value.
- `App.vue` / global chrome — the dark/light theme toggle is global (`DESIGN.md`'s entire color system is theme-conditional), not scoped to the modal.

With only `stations.ts` and `playback.ts` declared, it's unclear whether Settings is (a) fetched ad hoc via a command every time each consumer needs it (contradicts the "populated by commands on load... never poll" read-model pattern used everywhere else), or (b) meant to live in one of the two existing stores despite not fitting either's stated purpose ("favorites + search results" / "now-playing live state, ephemeral"). This cross-cutting read-model is unaddressed.

**Fix needed:** add a `settings.ts` Pinia store (or explicitly fold Settings into an existing store with a stated reason) to the Structural Seed, populated on load via command and consumed by both `SettingsModal.vue` and `TransportBar.vue`.

### Gap 4 — No Station/Settings field-level data shape anywhere in the spine, and several EXPERIENCE.md/DESIGN.md behaviors imply fields that aren't confirmed to exist

The spine never enumerates the fields of `Station` or `Settings` — the only data-shape commitments are: `Station.id` = Radio-Browser `stationuuid`, and "every struct crossing the boundary uses camelCase" (AD-6). That leaves several EXPERIENCE.md/DESIGN.md-required capabilities unconfirmed:

- **Filter control** (genre/country/language selects + removable chips, `DESIGN.md.components.filter-chip`) requires `Station` (or the search-result shape) to carry genre, country, and language fields — not confirmed.
- **Favorites reorder** (Station row's up/down controls, FR-3) requires a persisted order/position value per favorite in `store.rs`'s local JSON — not confirmed as part of the persisted shape.
- **Volume persistence** ("Volume slider updates live on drag, persists on release (FR-12)") requires a persisted volume field somewhere in Settings/store — not confirmed.
- **Now-Playing artwork** (DESIGN.md's Transport bar note: "Sits directly beneath the Now-Playing artwork") implies a station artwork/favicon field feeding the dashboard — not confirmed to exist on the Station shape (Radio-Browser does expose a `favicon` field, but the spine doesn't commit to carrying it through).
- **Stream Info tile** content (codec/bitrate/country/resolved-IP per DESIGN.md's typography section) is explicitly ephemeral/event-only per AD-5, which is consistent — but the resolved-IP and codec/bitrate fields aren't named in any payload struct, so it's unverified whether `audio/player.rs`'s metadata event is actually shaped to carry them.

None of this necessarily requires a rearchitecture — it's plausible all of it fits cleanly under AD-1/AD-6 — but because the spine has zero explicit field-level data-shape section, there's nothing to check these against, and a builder could plausibly ship a `Station` struct missing genre/country/language or a `Settings`/store shape missing persisted order/volume without violating anything written down.

**Fix needed:** add a minimal explicit field list (or reference table) for `Station` and `Settings`/persisted store shape, covering at minimum: genre, country, language, favicon/artwork, favorite order/position, and persisted volume.

### Gap 5 (minor) — Filter control's file-comment doesn't name the select panel itself

`SearchPanel.vue`'s Structural Seed comment reads "search input + filter chips." EXPERIENCE.md's Component Patterns table has the Filter control as "a small panel (not a full page) with three independent selects — genre, country, language" that is distinct from the filter *chips* (the chips are the applied-filter display beneath the search field, described separately in both EXPERIENCE.md and DESIGN.md's `filter-chip` component token). The comment names only the chips, not the select panel that produces them. Almost certainly the same file is meant to hold both, but as literally written the Structural Seed doesn't name the Filter control's actual input UI (the three selects), only its output display (the chips).

**Fix needed:** trivial — reword the comment to "search input + filter selects + filter chips" (or similar) so the Filter control (a named EXPERIENCE.md component) is unambiguously represented.

## Summary

No component or IA surface is left without a plausible file home — the Structural Seed and Capability→Architecture Map cover all eight Component Patterns rows and all four IA surfaces. The real gaps are in depth, not placement:

1. AD-4's event list has no distinct failed/error event, but EXPERIENCE.md requires a visually distinct error state separate from ordinary stop/pause.
2. AD-5 only commits to emitting Info Tile events on success, leaving the required "degraded" per-tile states without a defined non-polled trigger.
3. No Settings Pinia store is declared despite Settings data being needed in at least three places (Settings modal, Sleep Timer control pre-fill, global theme).
4. The spine has no field-level Station/Settings data shape at all, so genre/country/language (filter), persisted order (reorder), persisted volume (FR-12), and artwork are all unconfirmed.
5. Minor: the Filter control's select-panel UI isn't named in the Structural Seed comment, only its resulting chips.
