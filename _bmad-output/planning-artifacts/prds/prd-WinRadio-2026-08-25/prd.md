---
title: WinRadio PRD
status: final
created: 2026-08-25
updated: 2026-08-25
---

# PRD: WinRadio

## 0. Document Purpose

This PRD is for Javad (sole PM, sole user, sole future reader of downstream architecture/epics) and for whatever agent builds it next. It builds on [brief.md](../../briefs/brief-WinRadio-2026-08-25/brief.md), which already fixed the core problem (prior coding-agent attempts circled without a written spec) and locked a tight MVP scope. This PRD adds the detail the brief intentionally left out — concrete FRs, a visual/functional reference, and one deliberate scope expansion driven by that reference (see §6.1 and the memlog for the full rationale). Vocabulary is Glossary-anchored (§3); FRs are numbered globally and nested under features; every inferred decision carries an inline `[ASSUMPTION]` tag, indexed in §11.

## 1. Vision

WinRadio is a Windows desktop app that plays internet radio the way a dedicated device would: pick a station, hit play, it just works, and it stays out of the way in the system tray until needed again. It replaces a browser tab pinned to a stream — with the added benefit of memory (favorites, settings persist) and a proper now-playing view instead of a bare `<audio>` element in a browser.

It exists for one person. There's no roadmap beyond "Javad actually uses it instead of the browser tab," and no ambition beyond that holding true for a long time without the app breaking, or without needing another rebuild.

## 2. Target User

### 2.1 Jobs To Be Done

- As the builder and sole user, I want a radio player that's *mine* — no ads, no account, no bloat from features I don't use.
- I want to find a station without knowing its exact stream URL.
- I want to get back to a station I liked without re-searching for it.
- I want the audio to keep playing while I do other things on my PC, and to stop automatically if I fall asleep to it.

Single user, no onboarding flow needed — the app opens straight into "search or pick a favorite and hit play" (carried from brief.md).

### 2.2 Key User Journeys

*Lighter scope dial per PRD Discipline (hobby/solo, single operator) — one narrative journey instead of a formal multi-UJ set.*

- **UJ-1. Javad finds a station and comes back to it later.**
  Javad opens WinRadio from the tray, searches "jazz," previews a couple of results by hitting play on each, favorites the one he likes, and closes the window — playback keeps going, the app minimizes to tray. That evening he reopens it, sees the favorite at the top of the list, and hits play again without searching. **Edge case:** the stream drops mid-session (station's server hiccup); WinRadio retries the connection automatically rather than silently going quiet.

## 3. Glossary

- **Station** — A single internet radio stream entry (name, stream URL, genre, country, language, codec/bitrate, and geographic location) sourced from the Directory.
- **Directory** — The external station catalog WinRadio searches against. `[ASSUMPTION: Radio-Browser API — carried from brief.md]`.
- **Favorite** — A Station the user has explicitly saved for quick access, shown in the Station List ahead of/separate from search results.
- **Station List** — The persistent left-hand list of Favorites (and, during search, matching results), each row playable directly.
- **Now-Playing Dashboard** — The main panel shown while a Station plays: artwork, transport controls, and the Info Tiles.
- **Info Tile** — One of the small supplementary panels on the Now-Playing Dashboard: Location Tile, Weather Tile, or Stream Info Tile. Enhancement-only — see NFR-3.
- **Sleep Timer** — A user-set duration after which playback auto-stops.
- **Tray** — The Windows system tray icon and its play/pause affordance, active while WinRadio is minimized.

## 4. Features

### 4.1 Station Discovery & Search

**Description:** Javad searches the Directory by name, genre, country, or language and browses results in the Station List. Realizes UJ-1. `[ASSUMPTION: Directory = Radio-Browser API]`.

#### FR-1: Search stations
User can search the Directory by station name, and filter by genre, country, and/or language.

**Consequences (testable):**
- A text query returns matching stations from the Directory within 3 seconds on a normal home connection (see NFR-2).
- Genre/country/language filters can combine with a text query or be used alone to browse.
- No network/Directory unreachable → a clear inline error, not a silent empty list or crash.

#### FR-2: Preview a result
User can play any search result directly from the results list without first adding it as a Favorite.

### 4.2 Favorites & Station List

**Description:** The persistent left-hand rail (mirroring the reference image's station list) holding Favorites, with quick play per row. Realizes UJ-1.

#### FR-3: Manage favorites
User can add a Station to Favorites, remove it, and reorder the Favorites list.

**Consequences (testable):**
- Adding/removing a Favorite updates the Station List immediately and persists (FR-11) without an explicit "save" step.

#### FR-4: Play from the list
Clicking any row in the Station List (Favorite or search result) starts playback of that Station, replacing whatever was previously playing.

### 4.3 Playback & Transport

**Description:** Standard transport controls, always visible while something is loaded.

#### FR-5: Transport controls
User can play, pause, and resume the current Station, and adjust/mute volume.

**Consequences (testable):**
- Volume level persists across restarts (FR-11).
- If the stream drops, WinRadio automatically retries the connection before surfacing an error to the user (realizes UJ-1 edge case).

### 4.4 Now-Playing Dashboard

**Description:** The reference-image-driven centerpiece: current Station's artwork/visual, transport controls, live stream metadata, and three Info Tiles. `[ASSUMPTION: this feature set is the scope expansion agreed with Javad in this session — see §6.1 delta note]`.

#### FR-6: Now-playing metadata
Dashboard shows the current Station's name and, when the stream provides it, the live track/title metadata.

#### FR-7: Location Tile
Dashboard shows an Info Tile with the current Station's city/country and a simple map centered on it. `[ASSUMPTION: a lightweight embedded map (OpenStreetMap-based, no paid API key), not a full interactive map widget]`.

**Consequences (testable):** If the Station has no location data or the map fails to render, the tile shows its empty/placeholder state (§4.4 feature-specific NFR) rather than an error or blank space.

#### FR-8: Weather Tile
Dashboard shows an Info Tile with current weather (and short forecast) for the Station's city, when location data is available. `[ASSUMPTION: Open-Meteo — free, no API key, fits a personal project with no ops burden]`.

**Consequences (testable):** If Open-Meteo is unreachable or the Station has no location data, the tile shows its empty/placeholder state (§4.4 feature-specific NFR) without retrying indefinitely or blocking the rest of the Dashboard.

#### FR-9: Stream Info Tile
Dashboard shows an Info Tile with technical stream details: codec, bitrate, country, and the resolved server IP of the stream (matching the reference image's info tile). `[ASSUMPTION: codec/bitrate/country come from Directory metadata; IP is resolved via DNS lookup of the stream host]`.

**Consequences (testable):** If DNS resolution of the stream host fails or times out, the IP field shows its empty/placeholder state (§4.4 feature-specific NFR) while codec/bitrate/country (sourced from Directory metadata, not DNS) still display normally.

**Feature-specific NFRs:**
- Info Tiles are enhancement-only: if Location, Weather, or Stream Info data is unavailable or the backing service is down, the tile shows a quiet empty/placeholder state — it never blocks or degrades playback. This directly guards against reintroducing the fragility that sank the earlier attempts.

### 4.5 System Tray & Sleep Timer

**Description:** Background-friendly behavior — the app is meant to live in the tray more than on screen.

#### FR-10: Tray presence and control
Closing the main window minimizes WinRadio to the system tray instead of quitting; the tray icon exposes play/pause.

#### FR-11: Sleep Timer
User can set a duration after which playback automatically stops. `[ASSUMPTION: auto-stop only, not a wake/alarm feature — carried from brief.md]`.

### 4.6 Settings & Persistence

**Description:** Everything the user sets should survive a restart, with no separate "save" action anywhere in the app.

#### FR-12: Persistent state
Favorites, last-played station, volume, Sleep Timer duration, and theme preference persist locally across app restarts.

#### FR-13: Settings panel
A dedicated settings view exposes tray behavior (minimize-to-tray) and Sleep Timer defaults. Start-minimized and auto-start-with-Windows are deferred together (see §6.2) — neither is in this panel for v1.

### 4.7 Theming

#### FR-14: Dark/light theme
User can toggle between dark and light themes; the choice persists (FR-12).

## 5. Non-Goals (Explicit)

- WinRadio is not a general media player. It does not play local files, podcasts, or anything outside internet radio streams.
- It will not grow accounts, sync, or any server-side component. Everything is local to one machine.
- It is not a platform: no plugin system, no theming beyond dark/light, no multi-window layout.

## 6. MVP Scope

### 6.1 In Scope

Everything in §4 (FR-1 through FR-14) ships in v1, including the Now-Playing Dashboard's three Info Tiles.

**Delta note vs. brief.md:** the finalized brief scoped v1 as core search/play/favorites/now-playing + sleep timer + tray + theme + persistence, with richer additions explicitly deferred. During this PRD's discovery, Javad shared a reference screenshot (a Windows-8-era "Internet Radio" Metro app) and confirmed he wants its Location, Weather, and Stream Info tiles as real v1 features, not just visual inspiration. That's a genuine scope increase over the brief, logged in this workspace's `.memlog.md` rather than silently folded in. NFR-3 (below) exists specifically to keep this addition from reintroducing the fragility (external dependencies, half-wired features) that caused the original circling problem.

### 6.2 Out of Scope for MVP

- Recording streams to file, equalizer/audio effects, casting to other devices, mobile companion app — carried from brief.md, still true.
- Recently-played history, custom station groups/tags, start minimized / auto-start with Windows, media key support, manual custom-stream-URL entry, per-station volume memory — deferred, same as brief.md.
- UI localization/multi-language — the reference image shows a language toggle; `[ASSUMPTION: not needed, single-user English UI]`. `[NOTE FOR PM: revisit if this stops being purely personal-use.]`

## 7. Success Metrics

**Primary**
- **SM-1**: Javad opens and plays through WinRadio at least weekly, in place of a browser tab, without abandoning it — the core loop (search → play → favorite) takes a handful of clicks, no dead ends, carried from brief.md's usability bar. Validates FR-1 through FR-5, FR-10.

**Secondary**
- **SM-2**: Zero unrecoverable crashes during normal week-to-week use (a stream error or Info Tile outage should degrade gracefully, not crash the app). Validates NFR-3.

**Counter-metrics (do not optimize)**
- **SM-C1**: Number of Info Tiles or dashboard features is not a target to maximize. The Now-Playing Dashboard exists to serve UJ-1, not to accumulate tiles — more surface area is exactly the kind of scope drift that stalled the previous attempts. Counterbalances SM-1, and is the deliberate check on FR-6 through FR-9.

FR-12 through FR-14 (persistence, settings, theming) are table-stakes infrastructure the metrics above assume rather than measure directly — no separate SM for them by design.

## 8. Cross-Cutting NFRs

- **NFR-1 (Platform):** Windows 10/11 desktop only, Tauri shell. No macOS/Linux/mobile in v1.
- **NFR-2 (Performance):** Search results return within 3 seconds and playback starts within 2 seconds, both on a normal home connection. Idle memory footprint while docked in the tray stays low enough not to show up as a top process in Task Manager (target: under ~150MB resident) over a multi-day session.
- **NFR-3 (Reliability / graceful degradation):** Core playback (FR-1 through FR-5, FR-10, FR-11) never depends on the Weather or Location services being reachable. A Directory, weather, or map outage degrades only the affected Info Tile, never blocks search or playback.
- **NFR-4 (Offline handling):** With no network connection, the app shows a clear state (not a crash or silent hang) and recovers automatically once connectivity returns.
- **NFR-5 (Privacy):** No accounts, no telemetry, no data leaves the machine except Directory search queries, the Weather query (station coordinates only), and the stream connection itself.

## 9. Aesthetic and Tone

Visual and functional north star: the Windows-8-era "Internet Radio" Metro app Javad shared — dark background, tile-based dashboard, a station list rail on the left with a live count, a large now-playing panel (artwork/visual centerpiece + transport controls), and small supporting Info Tiles around it. `[ASSUMPTION: this is a structural/functional reference — tile layout and the specific info surfaced — not a pixel-exact visual clone; exact visual design is bmad-ux's job next, not this PRD's.]` Dark theme is default (FR-14 keeps light as a toggle, not the primary design target).

## 10. Open Questions

1. Repair the existing broken Tauri/Vue/Rust scaffold, or start the codebase fresh? Carried from brief.md, explicitly deferred to the architecture phase. The concrete evidence behind this question (from brief.md's Problem section): `lib.rs` never declares the `commands` module even though other files depend on it, `App.vue` renders a `router-view` with no router configured, and several referenced components only partially exist.
2. Which stream codecs must play reliably (MP3 only, or also AAC/OGG/others the Directory commonly returns)? Affects the audio backend choice in `winradio/src-tauri/src/audio/`.
3. Should the existing partially-built pieces (`PlayerControls.vue`, `SettingsModal.vue`, the two Pinia stores) be reused as a starting point, or treated as reference-only? Feeds the same architecture-phase decision as Q1.

## 11. Assumptions Index

- §3, §4.1 — Directory = Radio-Browser API.
- §4.4 FR-7 — Location Tile uses a lightweight OpenStreetMap-based embed, not a full interactive map.
- §4.4 FR-8 — Weather Tile backed by Open-Meteo.
- §4.4 FR-9 — Stream Info Tile's IP is resolved via DNS from the stream host; other fields come from Directory metadata.
- §4.5 FR-11 — Sleep Timer is auto-stop only, not a wake/alarm feature.
- §6.1 — Location/Weather/Stream Info tiles are a deliberate v1 scope increase over brief.md, driven by the reference image.
- §6.2 — No UI localization; single-language (English) interface.
- §9 — Reference image is a structural/functional guide, not a literal visual spec.
