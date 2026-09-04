---
title: 'Location Tile'
type: 'feature'
created: '2026-09-04'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: 'dd91750d9c3b144819573ddc92778d86f3499d6a'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The Now-Playing Dashboard's Info Tile grid has a reserved, empty "Location" slot. There's no Location Tile: no station coordinates on the persisted `Station`, no OSM tile fetch, no event pushing location data to the frontend.

**Approach:** Extend `Station` with `country`/`geoLat`/`geoLong` (back-compat defaulted). On a successful play, Rust emits `location-updated` (shared `{ok, data, reason}` envelope, AD-5) from the station's cached coordinates — no network round-trip for data already in hand. On `ok:true`, the frontend calls a new `get_location_tile` command that fetches one static OSM raster tile via `reqwest` with an OSM-required `User-Agent` (AD-12), returned as a base64 data URL. Confirmed with the user: no city (Radio-Browser has no city field, and no reverse-geocoding API is in scope) — country only; the mockup's "Brussels, BE" is illustrative, not literal.

## Boundaries & Constraints

**Always:** `location-updated` fires exactly once per play attempt, right after the existing `play` event, mirroring `metadata`'s reset lifecycle (idle on `play`/`stop`/`playback-error`). Tile fetch failure and "no coordinates" both resolve to the exact same "Location unknown" placeholder — never a partial state (e.g. country shown but no map). The new `Station` fields are `#[serde(default)]` so existing `store.json` files without them still load. `LocationTile.vue` renders inside the existing shared `InfoTile.vue` shell (new, shared by Stories 2.3/2.4 too) so all three tiles share one visual language.

**Ask First:** None remaining — the city-vs-country question was resolved with the user before this spec was written (country only, no new reverse-geocoding API).

**Never:** No map library, no API key, no interactive/pannable map — a single fixed-zoom static tile. Don't fetch the tile speculatively when a station has no coordinates. Don't block the `play` event or playback on the tile fetch — it's fully async and independent.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Station has coordinates | Station plays, `geoLat`/`geoLong` present | `location-updated` fires `ok:true`; tile fetches and renders with country + "© OpenStreetMap contributors" caption | N/A |
| Station has no coordinates | Station plays, `geoLat`/`geoLong` absent | `location-updated` fires `ok:false, reason:"Location unknown"` | Tile shows the placeholder immediately, no tile fetch attempted |
| Tile fetch fails | Coordinates present but `get_location_tile` rejects (network/HTTP error) | Tile shows the same "Location unknown" placeholder as the no-coordinates case | Never a partial (country-only, no map) state |
| Playback stops or errors | `stop` or `playback-error` event | Location resets to idle (no stale previous station's tile lingers) | Mirrors `metadata`'s existing reset lifecycle |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/Cargo.toml` -- add `base64 = "0.22"` -- encodes the fetched tile as a data URL
- `winradio/src-tauri/src/commands.rs:11` -- `Station` += `country: Option<String>`, `geo_lat: Option<f64>`, `geo_long: Option<f64>` (each `#[serde(default)]`) -- new persisted fields this and future tiles read
- `winradio/src-tauri/src/store.rs:310` -- mirror `missing_favorite_order_defaults_instead_of_failing_the_whole_station` for the 3 new fields -- back-compat regression coverage
- `winradio/src-tauri/src/directory.rs` -- add `LocationInfo { country, geo_lat, geo_long }`; pure `location_info_for(station: &Station) -> Option<LocationInfo>`; `tile_url(lat, long, zoom) -> String` (standard XYZ slippy-map math, zoom 10); `#[tauri::command] async fn get_location_tile(lat, long) -> Result<String, String>` using a distinct `OSM_USER_AGENT = "WinRadio/0.1 (personal desktop app)"` (AD-12's exact string, separate from the existing Radio-Browser `USER_AGENT`)
- `winradio/src-tauri/src/audio/player.rs:262` -- right after `self.emit("play", ...)`, build the envelope from `directory::location_info_for(&station)` and `self.emit("location-updated", envelope)`
- `winradio/src-tauri/src/main.rs:72` -- register `directory::get_location_tile` in `invoke_handler!`
- `winradio/src/stores/stations.ts:6` -- `Station` interface += `country?: string`, `geoLat?: number`, `geoLong?: number`
- `winradio/src/stores/search.ts:44` -- `toPlayableStation` carries `country`/`geoLat`/`geoLong` through from `DirectoryStation` (already has them)
- `winradio/src/stores/playback.ts` -- add `location` ref `{status: 'idle'|'ok'|'unavailable', country: string|null, tileImage: string|null}`; reset to idle in the existing `play`/`stop`/`playback-error` handlers (same point as `metadata`); new `location-updated` listener sets `ok`/`unavailable`, and on `ok` invokes `get_location_tile`, setting `tileImage` on success or falling back to `unavailable` on failure
- `winradio/src/components/InfoTile.vue` (new) -- shared shell (DESIGN.md `{components.info-tile}`): title, default slot for populated content, placeholder text for the degraded state — same shell either way, per NFR-3
- `winradio/src/components/LocationTile.vue` (new) -- reads `playbackStore.location`; renders country + tile image + attribution caption when `ok` and `tileImage` is set, else `InfoTile`'s placeholder ("Location unknown")
- `winradio/src/App.vue:165` -- replace the reserved empty "Location" tile div with `<LocationTile />`; Weather/Stream Info divs stay untouched (Stories 2.3/2.4)

## Tasks & Acceptance

**Execution:**
- [x] `commands.rs`, `store.rs` -- extend `Station`, add back-compat test -- new data model, no regression on old `store.json`
- [x] `directory.rs`, `Cargo.toml` -- `location_info_for`, `tile_url`, `get_location_tile` command -- pure/testable location + tile-fetch logic
- [x] `player.rs`, `main.rs` -- emit `location-updated`, register the new command -- wires the event into the existing play lifecycle
- [x] `stations.ts`, `search.ts` -- carry the 3 new fields through `toPlayableStation` -- favorited search results keep their coordinates
- [x] `playback.ts` -- `location` state + listener + tile-fetch action, reset on `play`/`stop`/`playback-error` -- read-model for the new event
- [x] `InfoTile.vue`, `LocationTile.vue`, `App.vue` -- shared shell + Location variant wired into the grid -- the actual UI this story delivers
- [x] Unit tests -- `tile_url`'s XYZ math, `location_info_for`'s Some/None branches, `playback.ts`'s reset/populate/fail paths -- covers the I/O matrix

**Acceptance Criteria:**
- Given a station with coordinates, when it plays, then the tile shows country, a centered OSM tile image, and the OSM attribution caption
- Given a station with no coordinates, or a tile-fetch failure, when the tile renders, then it shows a quiet "Location unknown" placeholder — never blocks playback or the other two (still-empty) tiles

## Spec Change Log

- **2026-09-04, step-04 review (patch findings):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. 4 confirmed `patch` findings sent back to the implementer and fixed: (1) `stations.ts`'s `toggleFavorite` add-branch dropped `country`/`geoLat`/`geoLong` when constructing the persisted `Station` from a candidate — favoriting a search result silently lost its coordinates forever, permanently showing "Location unknown" for that favorite even though the same station played directly from search showed the correct tile; fixed by copying the three fields through, with a new regression test; (2) `player.rs`'s `run_playback` re-emitted `location-updated` (triggering a fresh OSM tile fetch) on every drop/reconnect to the *same* station, not just on a genuine switch — fixed via a `last_location_station_id`-tracking dedup, extracted as a standalone, unit-tested `should_emit_location_update` helper; (3) `fetch_tile_data_url` treated any 2xx status as success without checking for an empty body, which would have produced a bogus `data:image/png;base64,` URL — fixed with an explicit empty-body check. While independently re-verifying this fix's own new test, found and fixed a real bug in the test itself (not the production code): the hand-rolled local TCP test server wrote its canned response without first reading the client's request, which raced with reqwest/hyper's client state machine and made the test fail deterministically (`hyper::Error(UnexpectedMessage)`) — fixed by having the test's fake server read the incoming request before writing its response; (4) `App.vue`'s reserved Weather/Stream Info placeholders hand-duplicated `InfoTile.vue`'s exact shell markup instead of reusing the component this story introduced specifically so all three tiles share one visual language — fixed by using `<InfoTile>` for both. Re-verified: `cargo build`/`cargo test` (37/37 passing, up from 26) and `npm run build`/`npx vitest run` (36/36 passing, up from 24) all pass after the patch round. KEEP: the original architecture (event-pushed `{ok, data, reason}` envelope sourced synchronously from the cached `Station`, a separate on-demand tile-fetch command, country-only location text per the earlier user decision) was sound — all 4 fixes are localized hardening, not redesigns. 9 additional non-blocking findings (whole-array `Station` decode fragility, no fallback for a null country, no cross-session tile caching, no request cancellation on rapid switching, untested tile-fetch success/404 paths, no component-level test for the tile-count invariant, unlinked OSM attribution text, no dark-theme tile treatment) were routed to `deferred-work.md` rather than fixed now.

## Design Notes

`tile_url` uses the standard slippy-map XYZ conversion (lon/lat → tile x/y at a fixed zoom) against `tile.openstreetmap.org` — zoom fixed at 10 for a city-scale view matching "centered on it". Well-documented, standard formula; no need to reproduce it here.

## Verification

**Commands:**
- `cd winradio/src-tauri && cargo build` -- expected: compiles clean with the new fields/command/dependency
- `cd winradio/src-tauri && cargo test` -- expected: all tests pass, no regressions on existing `Station`/store tests
- `cd winradio && npm run build` -- expected: type-checks and builds
- `cd winradio && npx vitest run` -- expected: existing + new tests pass

**Manual checks (if no CLI):**
- Live-verify via Pinia state injection (as used in spec-2-1): a station with coordinates renders the tile + attribution; a station without renders "Location unknown"

## Suggested Review Order

**Data model: the new `Station` fields and their back-compat**

- `Station` gains `country`/`geoLat`/`geoLong`, each `#[serde(default)]` so old `store.json` files still load.
  [`commands.rs:11`](../../winradio/src-tauri/src/commands.rs#L11)

- The back-compat regression test mirroring the existing `favoriteOrder` pattern.
  [`store.rs:345`](../../winradio/src-tauri/src/store.rs#L345)

**Rust: the event/command split and the two patch fixes in this area**

- `location_info_for`: pure, synchronous read off the cached `Station` — no network round-trip for coordinates already in hand.
  [`directory.rs:286`](../../winradio/src-tauri/src/directory.rs#L286)

- `get_location_tile`: the one genuinely live piece — fetches and base64-encodes a single OSM tile, now rejecting an empty-but-2xx body (the patch fix).
  [`directory.rs:367`](../../winradio/src-tauri/src/directory.rs#L367)

- `tile_url`'s standard XYZ slippy-map conversion, independently verified against a manual calculation for Brussels.
  [`directory.rs:310`](../../winradio/src-tauri/src/directory.rs#L310)

- `should_emit_location_update`: the dedup fix — a drop/reconnect to the same station no longer re-triggers a tile fetch.
  [`player.rs:541`](../../winradio/src-tauri/src/audio/player.rs#L541)

- Where it's wired into the play loop, right after the existing `play` emission.
  [`player.rs:309`](../../winradio/src-tauri/src/audio/player.rs#L309)

**Frontend: the store's reset/populate/supersede lifecycle and the favoriting fix**

- The `location-updated` listener: sets `ok`/`unavailable`, invokes `get_location_tile` on success, and never shows country without a map.
  [`playback.ts:146`](../../winradio/src/stores/playback.ts#L146)

- `toggleFavorite`: the fix that stops a favorited search result from silently losing its coordinates.
  [`stations.ts:134`](../../winradio/src/stores/stations.ts#L134)

**Peripherals**

- `InfoTile.vue`/`LocationTile.vue` and their wiring into `App.vue`, including the patch that replaced the hand-duplicated Weather/Stream Info shell with `InfoTile` itself.
  [`App.vue:163`](../../winradio/src/App.vue#L163)

- New Rust tests: `location_info_for`'s branches, `tile_url`'s math, the dedup helper, and the tile-fetch failure/empty-body paths.
  [`directory.rs:480`](../../winradio/src-tauri/src/directory.rs#L480)

- New frontend tests: the full location I/O matrix plus the supersession race and the `toggleFavorite` regression.
  [`playback.test.ts:165`](../../winradio/src/stores/playback.test.ts#L165)
