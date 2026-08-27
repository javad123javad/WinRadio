# Epic 1 Context: Core Radio Experience

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Deliver the full core radio loop — search a station, play it, favorite it, control it, and have it persist and quietly live in the system tray — on a rescued brownfield foundation. Not a greenfield build: it starts by cleaning up a broken existing scaffold (duplicate crate root, a serialization casing bug, a dangling unconfigured router, a hand-rolled blocking audio parser, unwanted EQ/recording subsystems) before any new UI is built. Epic 2's Info Tiles sit on top of the data model and event plumbing this epic establishes.

## Stories

- Story 1.1: Reliable Single-Station Playback (Foundation Rescue)
- Story 1.2: Search & Browse Stations
- Story 1.3: Preview & Play From Search Results
- Story 1.4: Favorites: Add, Remove, Reorder
- Story 1.5: Transport Controls: Play/Pause/Volume/Mute
- Story 1.6: System Tray Presence
- Story 1.7: Sleep Timer
- Story 1.8: Settings: Tray Behavior, Sleep Timer Default, Theme

## Requirements & Constraints

- Search by name, filterable by genre/country/language (combinable or standalone); results within 3s; unreachable Directory shows a distinct "can't reach directory" message, never conflated with a genuine zero-match.
- Any station row (search result or favorite) plays immediately on click, replacing whatever's playing — no select-then-play step.
- Favorites: add/remove/reorder (up/down, not drag), persisting immediately, no explicit save.
- Transport: play/pause/resume, live volume (persists on release), mute/unmute restores the exact prior volume rather than dragging to zero. Playback start budget: 2s.
- A dropped stream retries automatically (immediate, then 2s/5s/10s backoff, 3 attempts, ~20s budget) showing "Reconnecting…"; exhausted retries show a distinct error state ("Couldn't play this station"), never conflated with a normal pause.
- Closing the main window minimizes to tray instead of quitting; tray icon exposes play/pause; reopening restores the dashboard exactly as left.
- Sleep Timer: pick a duration, arms silently (small badge, no countdown); firing pauses normally; re-clicking the armed icon offers cancel.
- Settings: minimize-to-tray, Sleep Timer default, theme — every change applies immediately, no Apply/Save button.
- Favorites, last-played station, volume, Sleep Timer duration, and theme persist locally across restarts; on relaunch the last-played station shows idle (not auto-playing) with last volume restored.
- Platform: Windows 10/11 only, Tauri shell. Offline states must show clearly (never crash/hang) and recover automatically once connectivity returns. No telemetry/accounts; only network traffic is Directory search and the stream itself.

## Technical Decisions

- **Brownfield split:** `audio/`, `store.rs`, `timer.rs`, `tray.rs` extended in place; `App.vue`, all Vue components, and both Pinia stores rewritten from scratch — no existing markup reused.
- **Command boundary:** every network/filesystem op is a Tauri command; frontend never calls external hosts directly. Commands distinguish "offline" from "zero results." Every command returns `Result<T, String>`; the `String` is the exact user-facing message.
- **Audio pipeline:** `stream-download` (0.24) + `icy-metadata` (0.6, `default-features = false`) feeding `rodio` — no hand-rolled ICY parser, no `block_on` inside `Iterator::next()`. This crate pairing is unverified to compile together; a short build spike is warranted early. At most one active stream at a time — a new station tears down the previous first.
- **Event model, not polling:** Rust pushes `play`, `stop` (plain pause), `reconnecting`, `playback-error` (reason string, follows the retry policy above), `metadata-updated` (ICY track/title; absence is normal). Pinia stores are a read-model only — populated by commands on load, updated by events — never poll.
- **Serialization:** every boundary-crossing struct (`Station`, `Settings`, event payloads) derives `#[serde(rename_all = "camelCase")]`; internal Rust stays snake_case.
- **Data model:** `Station` gains `genre`/`country`/`language`/`favicon`/`codec`/`bitrate`/`geoLat`/`geoLong`/`favoriteOrder`; `id` is the Radio-Browser `stationuuid`. `Settings` pruned to `minimizeToTray`/`sleepTimerDefaultMinutes`/`theme`/`volume` only. Collection-mutating commands (reorder) take the full resulting collection, never a delta.
- **Removed entirely:** `lib.rs` (single crate root), EQ subsystem, recording subsystem, their commands, and `lame-sys`/`hound`/`cpal`. `vue-router` never added; no `<router-view/>` — Favorites/Search/Settings are rail-content-swaps and a modal, not routes.
- **Stack (retained as-is):** Tauri 1.5, Vue 3.4, Pinia 2.1, TypeScript 5.3, Vite 5.0, Tailwind 3.4, rodio 0.19, reqwest 0.12, tokio 1.38.
- Frontend files this epic touches stay flat (`components/`, `stores/`), no feature folders. `directory.rs` covers search only here — the OSM tile fetch is Epic 2. The Info Tile grid area is reserved by layout but left empty this epic.
- Search calls debounce client-side and aren't re-issued for an unchanged query.

## UX & Interaction Patterns

- Implement DESIGN.md tokens (dark default, light as a parallel set) via Tailwind, not one-off values. Two accents only: violet for interactive/active state, blue exclusively for station-identity text.
- Nav icon-button: circular, outlined, four destinations (Favorites, Filter, Search, Settings) — no "Home." Active state fills at low opacity.
- Station row: play icon + station name (secondary accent); favorite-star at the row's right edge as a separate click target from row-click-to-play; in Favorites only, always-visible up/down reorder controls at the row's left edge.
- Transport bar: single play/pause toggle, live volume slider persisting on release, separate mute toggle restoring prior volume exactly. Sleep Timer icon opens a duration picker; armed state shows only a small badge, no countdown.
- Search input plus a Filter panel (genre/country/language selects) with removable chips beneath the field.
- Settings modal: minimize-to-tray, Sleep Timer default, theme as label+control rows; closing the modal is the only confirmation, no toast.
- Copy is plain and technical-neutral ("No stations found for 'x'", "Reconnecting…" — never apologetic). Cold load shows Favorites from local storage with no spinner; fresh install shows "No favorites yet — search to find a station."
- Accessibility floor: `Space` toggles play/pause when no text field focused; `Tab` order is nav → rail → transport → settings; `Esc` closes Settings; visible focus rings everywhere; no state conveyed by color alone.
- Responsive rule: below a minimum width the (still-empty) Info Tile grid would drop 3-across to 2-across; the rail never collapses — a minimum window width is enforced instead.

## Cross-Story Dependencies

- Story 1.1 (foundation rescue) underlies every other story in this epic.
- Story 1.4 depends on Story 1.2/1.3's search results as a source of stations to favorite; both share the same row click-to-play mechanism.
- Story 1.8's Sleep Timer default pre-fills Story 1.7's duration picker.
- Story 1.6 (tray) depends on Story 1.5's play/pause state being exposed for tray control.
- Epic 2 depends on this epic's `Station` data model (`geoLat`/`geoLong`/`codec`/`bitrate`) and the event-push model; the Info Tile grid area reserved here is populated by Epic 2.
