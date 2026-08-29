---
title: 'Search & Browse Stations'
type: 'feature'
created: '2026-08-29'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: '4bf35a74d92735a39cc6601f9a61f3ba3144ff97'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** WinRadio has no way to find a station without already knowing its stream URL — `stations.ts` only ever shows a locally-persisted Favorites list, and FR1 (Directory search) is entirely unbuilt. Search/Filter nav icons exist in `App.vue` as disabled "(coming soon)" placeholders.

**Approach:** Add a new Rust `directory.rs` module with a `search_stations` command hitting Radio-Browser's public API (`all.api.radio-browser.info`), returning search-result stations distinct from persisted `Station`s. Add a `get_filter_options` command that lazily fetches real genre/country/language values from Radio-Browser's own discovery endpoints. On the frontend, wire the Search/Filter nav icons to swap the rail from Favorites to live results (same row component, per AD-1's "same rail, same row" pattern) with a debounced search input and three filter selects shown as removable chips.

## Boundaries & Constraints

**Always:** All network access goes through new Tauri commands — the frontend never calls Radio-Browser directly (AD-1). Search debounces client-side and is not re-issued for an unchanged `{query, genre, country, language}` tuple (AD-13). A network/connect failure returns `Err("Can't reach the station directory — check your connection")`; a successful call with zero matches returns `Ok(vec![])` — these are never conflated (AD-1/NFR4). Filter selections combine with the text query (AND) or work standalone. Selections render as removable chips beneath the search field. Only one of Favorites/Search is the active rail content at a time; Filter refines whichever is currently showing rather than being its own destination.

**Ask First:** None identified — search-result `Station` shape, endpoint, and error contract are all fixed by the architecture spine.

**Never:** Don't wire click-to-play behavior specially for search-result rows — reusing the existing `StationRow` component (built in Story 1.1) carries click-to-play over automatically; formally verifying that behavior is Story 1.3's job, not this one. Don't add a "Go" button — Enter submits immediately, everything else is debounced. Don't persist search results or filter selections across sessions (ephemeral UI state only). Don't hardcode a static genre/country/language list — fetch real values from Radio-Browser.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Type a query | "jazz" typed into search field | Debounced call to `search_stations`; rail swaps to matching results | N/A |
| Genre filter only, no text | Genre="Jazz" selected, query empty | `search_stations` called with just the genre param; chip shown | N/A |
| Genuine zero match | Query "zzzznonexistent" | Rail shows "No stations found for 'zzzznonexistent'." | `Ok(vec![])`, not an error |
| No network | Any search attempted while offline | Rail shows "Can't reach the station directory — check your connection." | `Err("Can't reach the station directory — check your connection")` |
| Unchanged query re-triggered | User pauses typing mid-debounce, same text as last completed search | No redundant network call | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/directory.rs` -- NEW: `DirectoryStation` DTO (stationuuid→id, name, url, favicon, tags, country, language, codec, bitrate, geoLat, geoLong; camelCase), `search_stations(name?, genre?, country?, language?)` command (GET `https://all.api.radio-browser.info/json/stations/search`, `User-Agent: WinRadio/0.1`, `hidebroken=true`, `limit=100`), `FilterOptions` DTO + `get_filter_options()` command (GET `/json/tags`, `/json/countries`, `/json/languages`, each `?order=stationcount&reverse=true&limit=100`)
- `winradio/src-tauri/src/main.rs` -- add `mod directory;`, register `directory::search_stations`, `directory::get_filter_options` in `generate_handler!` (currently ends at `commands::get_metadata`, main.rs:68)
- `winradio/src-tauri/Cargo.toml` -- no new dependencies (`reqwest` 0.13 already present, per `audio/player.rs`'s existing client-builder pattern)
- `winradio/src/stores/search.ts` -- NEW store: `query`, `filters{genre,country,language}`, `results`, `status` ('idle'|'loading'|'ok'|'zero-match'|'offline'), `filterOptions` (lazy-loaded once), `runSearch()` (debounced, skips unchanged params), `loadFilterOptions()`
- `winradio/src/components/StationRow.vue` -- NEW: extract the existing inline row markup from `App.vue`'s Favorites `<ul>` (App.vue:64-83) into a shared component taking a `station` prop, so Search reuses the exact same click-to-play row Story 1.1 already built
- `winradio/src/components/SearchPanel.vue` -- NEW: search `<input>` (debounced, Enter submits immediately) + three genre/country/language `<select>`s + removable chips row, per DESIGN.md's filter-chip component and EXPERIENCE.md's Search field/Filter control rows
- `winradio/src/App.vue` -- add `activeView` ref ('favorites'|'search'); wire Search nav icon (App.vue:27-35, currently disabled) to switch view and Filter nav icon (App.vue:18-25, currently disabled) to toggle the filter selects' visibility within `SearchPanel`; rail conditionally renders Favorites list or search results, both via `StationRow`

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/directory.rs` -- create module with `DirectoryStation`, `FilterOptions`, `search_stations`, `get_filter_options` -- new Directory search client per AD-1/AD-13
- [x] `src-tauri/src/main.rs` -- declare module, register both new commands -- exposes them to the frontend
- [x] `src/components/StationRow.vue` -- extract shared row component from `App.vue`'s existing Favorites markup -- enables "same rail, same row" reuse for search results
- [x] `src/stores/search.ts` -- create store with debounced `runSearch`, param-change dedup, lazy `loadFilterOptions` -- AD-13 compliance
- [x] `src/components/SearchPanel.vue` -- build search input + filter selects + removable chips -- UX-DR6 / EXPERIENCE.md Search field & Filter control
- [x] `src/App.vue` -- wire Search/Filter nav icons, add `activeView` rail-swap logic -- removes "(coming soon)" placeholders
- [x] Unit/integration test -- `search_stations` error path distinguishes offline (`Err`) from genuine zero-match (`Ok(vec![])`) -- covers I/O matrix's two error-adjacent rows

**Acceptance Criteria:**
- Given the search field, when Javad types a query, then matching stations from Radio-Browser appear, debounced, with no explicit "Go" needed
- Given genre/country/language filters, when any combination is applied (with or without a text query), then results reflect all active filters, shown as removable chips
- Given no network connection, when a search is attempted, then the message reads "Can't reach the station directory — check your connection" — visibly distinct from "No stations found for 'x'"

## Spec Change Log

- **2026-08-29, step-04 review (patch findings):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. 6 confirmed `patch` findings sent back to the implementer and fixed: (1) a retry deadlock where `lastSignature` was committed before the network call resolved, permanently blocking retry of an identical query after an offline failure — fixed by clearing it in the catch block; (2) no UI feedback during `status === 'loading'` — added a "Searching…" branch; (3) `get_filter_options_at`'s `tokio::join!` propagated the first failing endpoint's error, blanking all three dropdowns on one partial failure — fixed with independent `unwrap_or_default()` per endpoint; (4) `parse_stations` failed an entire 100-station batch on one type-mismatched field — fixed by converting entries individually via `filter_map`, skipping only the bad ones; (5) `parse_names` had no dedup, risking duplicate Vue `:key`s — fixed with an order-preserving dedup pass; (6) `CONNECT_ERROR` had silently drifted between a Rust constant and a separately hardcoded frontend string — fixed by having the frontend display the actual thrown error message, eliminating the duplicate literal. Re-verified: `cargo build`/`cargo test` (19/19 passing, up from 17) and `npm run build` all pass after the patch round. KEEP: the original architecture (base_url-parameterized functions for deterministic offline testing, pure parse functions split from I/O, the `DirectoryStation`/persisted-`Station` shape separation) was sound — all 6 fixes are localized hardening, not redesigns. 4 additional non-blocking findings (frontend test coverage, in-flight request cancellation, shared HTTP client reuse, filter-dropdown loading/error state) were routed to `deferred-work.md` rather than fixed now.

## Verification

**Commands:**
- `cd winradio/src-tauri && cargo build` -- expected: compiles clean with the new `directory` module
- `cd winradio/src-tauri && cargo test` -- expected: offline-vs-zero-match distinction test passes
- `cd winradio && npm run build` -- expected: type-checks and builds with the new store/components

**Manual checks (if no CLI):**
- Launch `tauri dev`: type a real query (e.g. "jazz"), confirm live results appear debounced; apply a genre filter and confirm a chip appears and results narrow; disconnect network and confirm the distinct offline message renders in the same visual slot as zero-match

## Suggested Review Order

**Directory search client (the story's core)**

- Entry point: `search_stations` command — optional name/genre/country/language, all AND'd together.
  [`directory.rs:222`](../../winradio/src-tauri/src/directory.rs#L222)

- `search_stations_at` builds the query params, only including filters that are actually set.
  [`directory.rs:189`](../../winradio/src-tauri/src/directory.rs#L189)

- `parse_stations` converts entries individually now, skipping a single malformed station instead of failing the whole batch.
  [`directory.rs:125`](../../winradio/src-tauri/src/directory.rs#L125)

- `CONNECT_ERROR`: the single source of truth for the offline message, now actually enforced (frontend displays it verbatim rather than a second hardcoded copy).
  [`directory.rs:22`](../../winradio/src-tauri/src/directory.rs#L22)

- `get_filter_options_at`: each of the three discovery endpoints falls back independently so one hiccup doesn't blank all three dropdowns.
  [`directory.rs:232`](../../winradio/src-tauri/src/directory.rs#L232)

- `parse_names` dedupes discovered genre/country/language values.
  [`directory.rs:135`](../../winradio/src-tauri/src/directory.rs#L135)

- `main.rs` registers both new commands alongside the existing surface.
  [`main.rs:70`](../../winradio/src-tauri/src/main.rs#L70)

**Frontend search store & the retry-deadlock fix**

- `executeSearch`: the signature-guard/dedup (AD-13) and the request-sequence guard against stale out-of-order responses.
  [`search.ts:80`](../../winradio/src/stores/search.ts#L80)

- The retry-deadlock fix: clearing `lastSignature` on failure so retrying an identical query after going offline actually re-issues the call.
  [`search.ts:122`](../../winradio/src/stores/search.ts#L122)

- `errorMessage` now carries the backend's actual string instead of a separately hardcoded copy.
  [`search.ts:116`](../../winradio/src/stores/search.ts#L116)

**UI: rail swap, search panel, shared row**

- `App.vue`'s rail-swap logic: `activeView`/`showFilterControls` and the Filter-nav-icon interpretation ("refines whichever is currently showing" → switches to Search and toggles the filter selects).
  [`App.vue:184`](../../winradio/src/App.vue#L184)

- The search rail's status branches, including the new "Searching…" state and the now-dynamic offline message.
  [`App.vue:109`](../../winradio/src/App.vue#L109)

- `SearchPanel.vue`: debounced input, Enter-submits-immediately, three filter selects, removable chips.
  [`SearchPanel.vue:94`](../../winradio/src/components/SearchPanel.vue#L94)

- `StationRow.vue`: extracted unchanged from the original Favorites markup — Search results reuse it as-is, which is what carries click-to-play over "for free" (Story 1.3's formal AC).
  [`StationRow.vue:1`](../../winradio/src/components/StationRow.vue#L1)

**Peripherals**

- New/updated tests: offline-vs-zero-match, malformed-entry skip, name dedup, and the partial-failure fallback for filter options.
  [`directory.rs:1`](../../winradio/src-tauri/src/directory.rs#L1) (test module at file end)

- `Cargo.toml`: `reqwest`'s `"query"` feature, needed for `.query()`.
  [`Cargo.toml:23`](../../winradio/src-tauri/Cargo.toml#L23)
