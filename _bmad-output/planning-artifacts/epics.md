---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories', 'step-04-final-validation']
inputDocuments:
  - '_bmad-output/planning-artifacts/prds/prd-WinRadio-2026-08-25/prd.md'
  - '_bmad-output/planning-artifacts/architecture/architecture-WinRadio-2026-08-25/ARCHITECTURE-SPINE.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-WinRadio-2026-08-25/DESIGN.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-WinRadio-2026-08-25/EXPERIENCE.md'
---

# WinRadio - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for WinRadio, decomposing the requirements from the PRD, UX design contract (DESIGN.md + EXPERIENCE.md), and Architecture spine into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: User can search the Directory by station name, and filter by genre, country, and/or language.
FR2: User can play any search result directly from the results list without first adding it as a Favorite.
FR3: User can add a Station to Favorites, remove it, and reorder the Favorites list.
FR4: Clicking any row in the Station List (Favorite or search result) starts playback of that Station, replacing whatever was previously playing.
FR5: User can play, pause, and resume the current Station, and adjust/mute volume.
FR6: Dashboard shows the current Station's name and, when the stream provides it, the live track/title metadata.
FR7: Dashboard shows an Info Tile with the current Station's city/country and a simple map centered on it.
FR8: Dashboard shows an Info Tile with current weather (and short forecast) for the Station's city, when location data is available.
FR9: Dashboard shows an Info Tile with technical stream details: codec, bitrate, country, and the resolved server IP of the stream.
FR10: Closing the main window minimizes WinRadio to the system tray instead of quitting; the tray icon exposes play/pause.
FR11: User can set a duration after which playback automatically stops (auto-stop only, not a wake/alarm feature).
FR12: Favorites, last-played station, volume, Sleep Timer duration, and theme preference persist locally across app restarts.
FR13: A dedicated settings view exposes tray behavior (minimize-to-tray), Sleep Timer default, and the dark/light theme toggle.
FR14: User can toggle between dark and light themes; the choice persists (FR12).

### NonFunctional Requirements

NFR1: Windows 10/11 desktop only, Tauri shell. No macOS/Linux/mobile in v1.
NFR2: Search results return within 3 seconds and playback starts within 2 seconds, both on a normal home connection. Idle memory footprint while docked in the tray stays under ~150MB resident over a multi-day session.
NFR3: Core playback (FR1-FR5, FR10, FR11) never depends on the Weather or Location services being reachable. A Directory, weather, or map outage degrades only the affected Info Tile, never blocks search or playback.
NFR4: With no network connection, the app shows a clear state (not a crash or silent hang) and recovers automatically once connectivity returns.
NFR5: No accounts, no telemetry, no data leaves the machine except Directory search queries, the Weather query (station coordinates only), and the stream connection itself.

### Additional Requirements

**No starter template — this is a brownfield hybrid rescue, not a greenfield scaffold.** Architecture AD-2 explicitly splits the existing codebase: the Rust backend (`audio/`, `store.rs`, `timer.rs`, `tray.rs`) is extended in place; the entire Vue frontend (`App.vue`, all components, both Pinia stores) is rewritten from scratch. Epic 1 / Story 1 must reflect this — it's foundational cleanup and re-wiring of the existing scaffold, not `npx create-*`.

Technical requirements from the Architecture Spine (AD-1 through AD-13), grouped by concern:

- **Command boundary & data contracts:** every network/filesystem op is a Tauri command (AD-1); commands distinguish "offline" from "zero results" in their error (AD-1/NFR4); every struct crossing the boundary uses `#[serde(rename_all = "camelCase")]` (AD-6, fixes a real existing bug); collection-mutating commands (favorites reorder) take the full resulting collection, never a delta (Consistency Conventions).
- **Audio streaming rework:** replace the hand-rolled, blocking-in-iterator ICY parser with `stream-download` (0.24) + `icy-metadata` (0.6, `default-features = false`) feeding `rodio` (AD-3); a build-compatibility spike for this three-crate pairing is recommended before deep implementation (unverified combination, flagged in Architecture's Deferred).
- **Event model:** Rust pushes state via Tauri events, never polled (AD-4) — `play`, `stop`, `reconnecting`, `playback-error` (with a defined retry policy: immediate retry, then 2s/5s/10s backoff, 3 attempts, ~20s budget), `metadata-updated`; Info Tile events (`location-updated`, `weather-updated`, `stream-info-updated`) share one `{ok, data, reason}` envelope (AD-5), each emitted exactly once per attempt whether it succeeds or fails; Stream Info's codec/bitrate/country and Location's coordinates read synchronously off the cached `Station` (no redundant re-fetch) — only the resolved IP and the tile image are actually fetched live.
- **Data shape changes:** `Station` gains `genre`/`country`/`language`/`favicon`/`codec`/`bitrate`/`geoLat`/`geoLong`/`favoriteOrder` (native Radio-Browser fields plus an explicit reorder position); `Settings` is pruned to just `minimizeToTray`/`sleepTimerDefaultMinutes`/`theme`/`volume`, dropping all EQ/recording/device-picker fields.
- **Scope removal:** delete the EQ subsystem (`EqSource`, `BiquadFilter`, related commands) and the recording subsystem (`Recorder`, `start_recording`/`stop_recording`, native save dialog) entirely, plus their now-unused dependencies `lame-sys`, `hound`, `cpal` (AD-9) — both are explicit PRD non-goals.
- **lib.rs / crate structure:** delete the vestigial `src-tauri/src/lib.rs` (duplicate, unconsumed lib target) so `main.rs` is the sole crate root (AD-8).
- **Router removal:** delete `App.vue`'s dangling `<router-view/>`; `vue-router` is never added — the IA is a single window with rail-content-swaps and a modal, not pages (AD-10).
- **Stack retained as-is:** Tauri 1.5 (not upgrading to v2 — verified deliberate choice, AD-7), Vue 3.4, Pinia 2.1, TypeScript 5.3, Vite 5.0, Tailwind 3.4.
- **Location Tile networking:** the OSM tile fetch goes through a Rust command with a proper `User-Agent` header — never a direct `<img>` hotlink from Vue — and the tile must carry an "© OpenStreetMap contributors" attribution caption (AD-12, both a policy-compliance requirement and consistency with AD-1's command-boundary rule).
- **Deployment:** no server component ever; distribution is `tauri build` → an unsigned Windows x86_64 installer, manual install; no auto-update, no CI pipeline, no code signing, no telemetry (AD-11) — explicitly out of scope for story-level work at this stage.

### UX Design Requirements

UX-DR1: Implement the DESIGN.md token system (colors, typography, `rounded`, `spacing`, `components` — dark as default, light as a full parallel set) and wire it through Tailwind rather than hand-picked one-off values.
UX-DR2: Build the Nav icon-button component — circular, outlined, active state fills at low opacity — for exactly four destinations: Favorites, Filter, Search, Settings. No "Home" icon (confirmed non-existent — the reference image's icon was a language-switcher artifact, not a nav concept).
UX-DR3: Build the Station row component: play-icon + station name (in the `secondary` accent color), a favorite-star toggle at the row's right edge (separate click target from row-click-to-play), and — Favorites list only — always-visible up/down reorder controls at the row's left edge (not hover-reveal).
UX-DR4: Build the shared Info Tile shell component (one component, three content variants: Location/Weather/Stream Info) with a consistent empty/degraded state (quiet muted text, same shell, never red/error styling).
UX-DR5: Build the Transport bar: play/pause toggle, volume slider, a separate mute icon-toggle (distinct from dragging volume to zero — mute/unmute must restore the prior level), and the Sleep Timer control (icon button opening a duration picker, with a small active-badge as the only "timer armed" indicator — no persistent countdown).
UX-DR6: Build the Search input plus the Filter control (three independent selects: genre/country/language) plus removable filter chips showing what's currently applied.
UX-DR7: Build the Settings modal: minimize-to-tray toggle, Sleep Timer default duration, and the dark/light theme toggle as simple label+control rows — changes apply immediately, no separate "Apply"/"Save" button.
UX-DR8: Build the Now-Playing Dashboard layout: artwork/disc visual, station name in the `display` type role, live track/title as a smaller subtext line beneath it, and the three-tile Info grid.
UX-DR9: Implement the accessibility floor: `Space` toggles play/pause when no text field is focused, `Tab` order follows nav→rail→transport→tiles, `Esc` closes the Settings modal, every interactive element shows a visible focus ring, and no state is conveyed by color alone (e.g. error text, not just red tint).
UX-DR10: Implement the one responsive rule: below a minimum window width, the Info Tile grid drops from 3-across to 2-across (Stream Info wraps beneath) before anything is hidden; the Station List rail never collapses — the app enforces a minimum window width instead.
UX-DR11: Apply the Voice and Tone microcopy rules throughout (plain, technical-neutral — "No stations found for 'x'", not "Oops!"; "Weather unavailable", not an error-styled technical dump).
UX-DR12: Implement all named State Patterns: cold load (Favorites from local persistence, no spinner), no-favorites-yet prompt, no-search-results with **two distinct messages** (genuine zero-match vs. offline/can't-reach-directory), stream reconnecting / stream failed (error-red, the one place it's used), Info Tile degraded (per-tile, independent), and settings-saved (modal close is the confirmation, no toast).
UX-DR13: Render the "© OpenStreetMap contributors" attribution caption on the Location Tile beneath the map image (required by AD-12 / OSM's tile usage policy).

### FR Coverage Map

FR1: Epic 1 - Search stations by name/genre/country/language
FR2: Epic 1 - Preview a search result by playing it directly
FR3: Epic 1 - Favorites: add/remove/reorder
FR4: Epic 1 - Play from Station List (Favorite or search result)
FR5: Epic 1 - Transport controls (play/pause/resume/volume/mute)
FR6: Epic 2 - Now-playing metadata display
FR7: Epic 2 - Location Tile
FR8: Epic 2 - Weather Tile
FR9: Epic 2 - Stream Info Tile
FR10: Epic 1 - Tray presence and control
FR11: Epic 1 - Sleep Timer
FR12: Epic 1 - Persistent state
FR13: Epic 1 - Settings panel
FR14: Epic 1 - Dark/light theme
NFR1: Epic 1 - Platform/build target (Tauri 1.5, Windows-only)
NFR2: Epic 1 - Core-loop performance budget (Info Tile latency is Epic 2's concern, not budgeted)
NFR3: Epic 2 - Info Tile independent degradation (this epic's delivery mechanism)
NFR4: Epic 1 - Offline handling at the command boundary
NFR5: Epic 1 - Privacy (no telemetry, foundational)

## Epic List

### Epic 1: Core Radio Experience
Search for a station, play it, favorite it, come back to it later — reliably, with the app quietly living in the tray and remembering everything between sessions. Includes the full brownfield foundation: deleting `lib.rs`, fixing the serde casing bug, removing EQ/recording, removing the dangling router, swapping in `stream-download`+`icy-metadata`, and building the entire new Vue frontend (nav bar, Station List rail with reorder, transport bar with mute + Sleep Timer, search+filter, Settings modal, dark/light tokens) plus the accessibility floor and microcopy rules.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR10, FR11, FR12, FR13, FR14
**NFRs covered:** NFR1, NFR2 (core loop), NFR4, NFR5

### Epic 2: Now-Playing Dashboard & Info Tiles
While something's playing, see live track info plus three at-a-glance context tiles — station location, local weather, and technical stream details — each degrading quietly on its own if its data source is unreachable.
**FRs covered:** FR6, FR7, FR8, FR9
**NFRs covered:** NFR3

## Epic 1: Core Radio Experience

Search for a station, play it, favorite it, come back to it later — reliably, with the app quietly living in the tray and remembering everything between sessions. Includes the full brownfield foundation: deleting `lib.rs`, fixing the serde casing bug, removing EQ/recording, removing the dangling router, swapping in `stream-download`+`icy-metadata`, and building the entire new Vue frontend plus the accessibility floor and microcopy rules.

### Story 1.1: Reliable Single-Station Playback (Foundation Rescue)

As Javad,
I want to press play on a station and have it play reliably without the app's underlying wiring bugs getting in the way,
So that I have a solid foundation to build everything else on.

**Acceptance Criteria:**

**Given** the rewritten Vue shell (new dark-theme token layout, no `router-view`) and the cleaned-up Rust backend
**When** Javad clicks play on a station
**Then** audio plays via the `stream-download`+`icy-metadata` pipeline (no hand-rolled blocking-in-iterator code)

**Given** the Rust crate
**When** it's built
**Then** `lib.rs` no longer exists, there's a single crate root, and no EQ/recording code, commands, or dependencies (`lame-sys`/`hound`/`cpal`) remain

**Given** a `Station`/`Settings` struct crossing the Tauri boundary
**When** it serializes
**Then** fields are camelCase on the wire with no manual mapping needed

**Given** the base shell layout
**When** the window is resized below the minimum width
**Then** the Info Tile grid area (reserved but empty this epic) would collapse from 3-across to 2-across, and the Station List rail never collapses — a minimum window width is enforced instead

### Story 1.2: Search & Browse Stations

As Javad,
I want to search the station directory by name and filter by genre/country/language,
So that I can find stations without knowing their exact stream URL.

**Acceptance Criteria:**

**Given** the search field
**When** Javad types a query
**Then** matching stations from Radio-Browser appear (debounced, no explicit "Go" needed)
**And** genre/country/language filters combine with the query or work standalone, shown as removable chips

**Given** no network connection
**When** a search is attempted
**Then** the message reads "Can't reach the station directory — check your connection" — distinct from a genuine zero-match "No stations found for 'x'"

### Story 1.3: Preview & Play From Search Results

As Javad,
I want to play any search result directly,
So that I can preview a station before deciding to favorite it.

**Acceptance Criteria:**

**Given** a search results list
**When** Javad clicks a result row
**Then** it plays immediately, replacing whatever was previously playing — no separate "select then play" step

### Story 1.4: Favorites: Add, Remove, Reorder

As Javad,
I want to save stations to Favorites and reorder them,
So that I can quickly get back to the ones I like.

**Acceptance Criteria:**

**Given** a station (favorite or search result)
**When** Javad clicks its star
**Then** it's added to/removed from Favorites and the change persists across restarts

**Given** the Favorites list
**When** Javad uses the always-visible up/down controls on a row
**Then** it moves one position and the new order persists

**Given** a fresh install with no favorites
**When** the app opens
**Then** the rail shows "No favorites yet — search to find a station" instead of an empty box
**And** on relaunch, the rail loads Favorites immediately from local storage — no spinner needed

**Given** the Favorites list
**When** Javad clicks a favorite row (not just search results, per Story 1.3's behavior)
**Then** it plays immediately, replacing whatever was previously playing — the same station-row click-to-play behavior applies regardless of which list the row is in

### Story 1.5: Transport Controls: Play/Pause/Volume/Mute

As Javad,
I want play/pause/volume/mute controls that behave predictably,
So that I can control playback without surprises.

**Acceptance Criteria:**

**Given** a station is loaded
**When** Javad drags the volume slider
**Then** it updates live and persists on release

**Given** audio is playing
**When** Javad clicks mute then unmute
**Then** it restores the exact prior volume (not "drag to zero")

**Given** a stream drops mid-playback
**When** WinRadio retries (immediately, then 2s/5s/10s backoff, up to 3 attempts within ~20s)
**Then** the transport bar shows "Reconnecting…"
**And** if all retries fail, it shows "Couldn't play this station" in error styling — a genuinely different state from a normal pause

**Given** the app relaunches
**When** the dashboard loads
**Then** the last-played station's info shows idle in the Now-Playing area (not auto-playing) with the last-set volume restored

### Story 1.6: System Tray Presence

As Javad,
I want WinRadio to minimize to the tray instead of quitting, with play/pause available from the tray icon,
So that it stays out of my way while still playing.

**Acceptance Criteria:**

**Given** minimize-to-tray is enabled
**When** Javad closes the main window
**Then** it hides instead of quitting, and the tray icon exposes play/pause

**Given** the app is in the tray
**When** Javad reopens it
**Then** the dashboard restores exactly as left

### Story 1.7: Sleep Timer

As Javad,
I want to set a timer that stops playback after a chosen duration,
So that I can fall asleep to the radio without it playing all night.

**Acceptance Criteria:**

**Given** the transport bar's Sleep Timer icon
**When** Javad picks a duration
**Then** it arms and the icon shows a small active badge — no persistent countdown

**Given** an armed timer
**When** it fires
**Then** playback pauses normally (no special modal)
**When** Javad clicks the badged icon before it fires
**Then** it offers cancel

### Story 1.8: Settings: Tray Behavior, Sleep Timer Default, Theme

As Javad,
I want a Settings panel for minimize-to-tray, my default Sleep Timer duration, and dark/light theme,
So that the app behaves the way I want without hunting for options.

**Acceptance Criteria:**

**Given** the Settings modal
**When** Javad toggles any of its three rows
**Then** it applies immediately — no Apply/Save button, closing the modal *is* the confirmation

**Given** a theme change
**When** applied
**Then** the DESIGN.md token set switches live and persists across restarts
**And** the Sleep Timer default set here pre-fills Story 1.7's duration picker

**Given** the modal is open
**When** Javad navigates by keyboard
**Then** `Tab` order follows nav→rail→transport→settings, `Esc` closes it, and focus rings are visible throughout the app

## Epic 2: Now-Playing Dashboard & Info Tiles

While something's playing, see live track info plus three at-a-glance context tiles — station location, local weather, and technical stream details — each degrading quietly on its own if its data source is unreachable.

### Story 2.1: Now-Playing Metadata Display

As Javad,
I want to see the current station's name and live track/title when the stream provides it,
So that I know what's actually playing.

**Acceptance Criteria:**

**Given** a station is playing
**When** it provides ICY metadata
**Then** the live track/title shows as a subtext line beneath the station name (display type)

**Given** a station with no ICY metadata
**When** it plays
**Then** the subtext is simply blank — not an error state

### Story 2.2: Location Tile

As Javad,
I want to see the station's city/country and a small map,
So that I get a sense of where I'm listening from.

**Acceptance Criteria:**

**Given** a station with location data
**When** it plays
**Then** the tile shows city/country and a single OSM tile image, fetched via a Rust command with a proper `User-Agent` header (never a direct browser hotlink), with an "© OpenStreetMap contributors" attribution caption

**Given** a station with no location data, or the tile fetch fails
**When** rendered
**Then** it shows a quiet "Location unknown" placeholder in the shared `{ok, data, reason}` envelope — never blocks playback or the other two tiles

### Story 2.3: Weather Tile

As Javad,
I want to see current weather for the station's location,
So that the dashboard feels a bit more alive and contextual.

**Acceptance Criteria:**

**Given** a station with location data
**When** it plays
**Then** the tile shows current conditions and a short forecast from Open-Meteo

**Given** no location data or a failed fetch
**When** rendered
**Then** it shows "Weather unavailable" independently — never blocks playback or the other tiles

### Story 2.4: Stream Info Tile

As Javad,
I want to see technical stream details (codec, bitrate, country, resolved IP),
So that I can see exactly what I'm connected to.

**Acceptance Criteria:**

**Given** a playing station
**When** the tile renders
**Then** codec/bitrate/country show immediately, read synchronously off the already-cached station data — no loading flash

**Given** the DNS lookup for the stream's IP
**When** it succeeds or fails
**Then** the IP field updates or shows "unavailable" independently — a failed IP lookup never triggers the "Stream failed" playback-error state, since audio itself is unaffected
