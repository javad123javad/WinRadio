---
title: 'Stream Info Tile'
type: 'feature'
created: '2026-09-18'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: '982f51f0f313f6354581a9ed5438fa6a40feebce'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The dashboard's Stream Info tile is a reserved-but-empty placeholder; the technical details of what's actually playing (codec, bitrate, country, resolved server IP) are missing. This is the last of the three fixed Info Tiles (SM-C1).

**Approach:** Unlike Location/Weather, three of the four fields (codec, bitrate, country) are already sitting on the cached `Station` object the frontend already holds — they render directly from `playbackStore.currentStation`, synchronously, with no event round-trip and no loading flash (AC1). Only the DNS-resolved IP is genuinely live; it gets its own `stream-info-updated` event, following the exact same dedup/spawn/generation-guard shape as Location and Weather.

## Boundaries & Constraints

**Always:**
- Codec/bitrate/country render the instant a station is selected/playing, read straight off `playbackStore.currentStation` — never wait for any Rust event for these three fields.
- A station missing one of codec/bitrate/country renders a per-field blank/dash for that field only — this never collapses the whole tile to the shared placeholder. The shared `InfoTile` placeholder is reserved for the true idle case (nothing currently playing).
- The IP lookup is the tile's only fetched part: parse the host from `station.url`, resolve it via DNS, `tokio::spawn`'d off the playback-critical path exactly like Weather's fetch, guarded by `is_current_generation` before emitting.
- Dedup the IP lookup per-station via its own `last_stream_info_station_id` field + `should_emit_stream_info_update` helper — independent of Location's and Weather's, same contract (same-station reconnect suppressed, genuine switch always emits).
- A DNS failure (unresolvable host, malformed URL, lookup error) resolves the IP field to "unavailable" and must never fire `playback-error` — same scope isolation as Location/Weather.
- Add `codec: Option<String>`/`bitrate: Option<u32>` to the persisted `Station` (backend and frontend), `#[serde(default)]` for back-compat, threaded through `toggleFavorite`/`toPlayableStation` exactly like `country`/`geoLat`/`geoLong` were in Story 2.2 — get it right the first time, don't repeat that story's dropped-fields bug.

**Ask First:** None — shape fully determined by Stories 2.2/2.3's precedent and the frozen epic context.

**Never:**
- No polling; one `stream-info-updated` event per attempt, per AD-4/AD-5.
- Do not add stream info to local storage beyond the `codec`/`bitrate` `Station` fields themselves (the IP is ephemeral, per-session only).
- Do not add a 4th tile — this completes the fixed set of three (SM-C1).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Station playing, full data, DNS resolves | Station has codec/bitrate/country; host resolves | Codec/bitrate/country show immediately; IP populates once `stream-info-updated{ok:true}` arrives | N/A |
| Station missing one static field | e.g. station has no `bitrate` | That field renders blank/dash; codec/country/IP unaffected | N/A |
| DNS lookup fails | Malformed URL, no host, or resolution error | `stream-info-updated{ok:false, reason:"..."}`; IP field shows "unavailable"; no `playback-error` | Tile's other fields unaffected |
| Same-station reconnect | Stream drops and reconnects to the same station id | No second `stream-info-updated` emitted | N/A |
| Rapid station switch mid-lookup | User switches stations while a DNS lookup is in flight | Stale IP is dropped (`is_current_generation` false); only the new station's IP (or its own unavailable state) is shown | N/A |
| Nothing playing | `currentStation` is null | Shared `InfoTile` placeholder ("Stream info unavailable"), same as idle Location/Weather | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/commands.rs:11` -- `Station` += `codec: Option<String>`, `bitrate: Option<u32>` (`#[serde(default)]` each, mirrors `country`/`geo_lat`/`geo_long`)
- `winradio/src-tauri/src/store.rs:333` -- mirror `missing_location_fields_default_instead_of_failing_the_whole_station` for the 2 new fields
- `winradio/src-tauri/src/stream_info.rs` (new) -- pure `extract_host_and_port(url: &str) -> Option<(String, u16)>` (via `http::Uri`, already a direct dependency -- no new crate needed; defaults port 80/443 by scheme when absent); `async fn resolve_ip(url: &str) -> Result<String, String>` (host/port extraction, then `tokio::task::spawn_blocking` wrapping `std::net::ToSocketAddrs` -- the only DNS mechanism available without adding a `net`-feature/new dependency)
- `winradio/src-tauri/src/main.rs:12` -- add `mod stream_info;`
- `winradio/src-tauri/src/audio/player.rs:77` -- new `StreamInfoUpdatedPayload { ok, data: Option<String>, reason: Option<String> }` (the `data` is just the resolved IP string -- no struct needed for a single dynamic field, unlike Location's/Weather's multi-field `data`); new const `STREAM_INFO_UNAVAILABLE: &str = "unavailable"`
- `winradio/src-tauri/src/audio/player.rs:106` -- `RadioPlayer` += `last_stream_info_station_id: Mutex<Option<String>>` (mirrors `last_weather_station_id`)
- `winradio/src-tauri/src/audio/player.rs:411` (right after the Weather block, still inside the `Ok(join_handle)` arm) -- dedup via `should_emit_stream_info_update`, then `stream_info::extract_host_and_port` synchronously; on `None` (bad URL) emit `ok:false` immediately; on `Some`, `tokio::spawn` the DNS resolution guarded by `is_current_generation(gen)` before emitting, identical shape to Weather's block
- `winradio/src-tauri/src/audio/player.rs:645` -- new pure `fn should_emit_stream_info_update(last_emitted_for: &mut Option<String>, station_id: &str) -> bool` (identical logic to `should_emit_weather_update`, own field)
- `winradio/src/stores/stations.ts:6` -- `Station` += `codec?: string`, `bitrate?: number`
- `winradio/src/stores/stations.ts:138` -- `toggleFavorite`'s add-branch += `codec: candidate.codec, bitrate: candidate.bitrate,`
- `winradio/src/stores/search.ts:44` -- `toPlayableStation` += `codec: station.codec, bitrate: station.bitrate,` (both already on `DirectoryStation`)
- `winradio/src/stores/playback.ts:36` -- `StreamInfoState { status: 'idle'|'ok'|'unavailable', ip: string|null }`, `emptyStreamInfo()`; reset alongside `location`/`weather` in the `play`/`stop`/`playback-error` handlers
- `winradio/src/stores/playback.ts:207` -- new `listen<{ok, data: string|null, reason: string|null}>('stream-info-updated', ...)` -- sets `streamInfo.value` directly (`ok` -> `{status:'ok', ip:data}`, else `{status:'unavailable', ip:null}`)
- `winradio/src/stores/playback.ts:355` -- export `streamInfo`
- `winradio/src/components/StreamInfoTile.vue` (new) -- reads `playbackStore.currentStation` directly for codec/bitrate/country (per-field blank fallback, no loading flash) and `playbackStore.streamInfo` for the IP row; uses `InfoTile`'s placeholder only when `!playbackStore.currentStation` (true idle)
- `winradio/src/App.vue:167,197` -- replace the last `<InfoTile v-for="tile in remainingInfoTiles">` placeholder with `<StreamInfoTile />`; remove `remainingInfoTiles` (all three tiles now wired)

## Tasks & Acceptance

**Execution:**
- [x] `commands.rs`, `store.rs` -- extend `Station` with `codec`/`bitrate`, back-compat test -- new data, no regression on old `store.json`
- [x] `stream_info.rs`, `main.rs` -- `extract_host_and_port`, `resolve_ip` -- pure/testable host parsing + DNS lookup
- [x] `player.rs` -- `StreamInfoUpdatedPayload`, `last_stream_info_station_id`, `should_emit_stream_info_update`, emit block -- wires the event without blocking playback
- [x] `stations.ts`, `search.ts` -- carry `codec`/`bitrate` through `toggleFavorite`/`toPlayableStation` -- favorited/replayed stations keep their stream info
- [x] `playback.ts` -- `streamInfo` state + listener + resets -- read-model for the new event
- [x] `StreamInfoTile.vue`, `App.vue` -- render the tile (per-field fallbacks, idle placeholder), replace the last reserved placeholder -- the actual UI this story delivers; all three Info Tiles now live
- [x] Unit tests -- `extract_host_and_port` (valid URL, malformed URL, no host, default ports), `should_emit_stream_info_update` (first call / same-station suppress / different-station emit), `toggleFavorite`/`toPlayableStation` carrying `codec`/`bitrate`, `playback.ts` streamInfo reset/populate/fail paths -- covers the I/O matrix

**Acceptance Criteria:**
- Given a playing station, when the tile renders, then codec/bitrate/country show immediately, read synchronously off the already-cached station data -- no loading flash
- Given the DNS lookup for the stream's IP, when it succeeds or fails, then the IP field updates or shows "unavailable" independently -- a failed IP lookup never triggers the "Stream failed" playback-error state
- Given a stream that drops and reconnects to the same station, when it resumes, then no duplicate `stream-info-updated` fetch/emit occurs

## Spec Change Log

- **2026-09-18, step-04 review (no code changes):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. Zero `patch`/`bad_spec`/`intent_gap` findings survived. Two review layers (edge-case-hunter and verification-gap) independently converged on the same real bug — a stream drop/reconnect to the *same* station leaves the tile permanently blank for the rest of that station's playback — which I confirmed by reading `player.rs:335` (`self.emit("play", ...)` fires on every retry-loop iteration, not just genuine switches) combined with the frontend's unconditional idle-reset on every `play` event. This is classified `defer`, not `patch`: the exact same mechanism already exists for Location (Story 2.2) and Weather (Story 2.3) — Story 2.4 only inherited the pattern via its own dedup helper, so the root cause predates and is not caused by this story; fixing it properly needs a coordinated change across all three tiles. Rejected as noise or already-covered by existing precedent: `resolve_ip`'s internal re-parse of the URL (harmless duplication), discarded detailed error strings (already covered by the existing "no logging/tracing" deferred item), the tile's per-field vs. whole-tile placeholder distinction (intentional per spec), no manual-refresh affordance (out of scope), non-http(s) URL schemes (the app's audio pipeline can't play those at all, so `extract_host_and_port` never sees one), and the malformed-bitrate whole-array-decode risk (duplicate of the already-tracked Story 2.2 deferred item on whole-array `Station` decode fragility). 4 non-blocking findings routed to `deferred-work.md`: the cross-cutting reconnect bug (documented above), no timeout on `resolve_ip`'s blocking DNS lookup (unlike Weather's reqwest client), a cosmetic `bitrate: 0` falsy-check gap, and no `StreamInfoTile.vue` component test for AC1's synchronous-read path (mirrors the same already-accepted gap for Location/Weather). KEEP: the architecture (codec/bitrate/country read synchronously off the cached `Station`, only the IP genuinely fetched via the same event/dedup/spawn/generation-guard shape as Location/Weather, `codec`/`bitrate` threaded through `toggleFavorite`/`toPlayableStation` correctly from the start) was sound as specified — no redesign needed.

## Design Notes

`extract_host_and_port` uses `http::Uri` (already a direct dependency for the HTTP stack -- no new crate needed) rather than the fuller `url` crate; radio stream URLs are plain `http(s)://host[:port]/path` and don't need `url`'s userinfo/fragment handling. DNS resolution uses blocking `std::net::ToSocketAddrs` inside `tokio::task::spawn_blocking` rather than adding tokio's `net` feature -- keeps the dependency surface minimal, same spirit as Weather's and Location's "no new dependency beyond what's already justified" approach.

## Verification

**Commands:**
- `cargo test` (via the MSVC toolchain, not the default Bash-tool one) -- all existing + new tests pass
- `cargo build` -- clean build, `stream_info.rs` compiles and is wired into `main.rs`
- `npm run build` -- clean Vue/TS build
- `npx vitest run` -- all existing + new tests pass

**Manual checks (if no CLI):**
- Play a station and confirm codec/bitrate/country appear immediately (before the IP field populates); confirm the IP appears shortly after. Play a station with a malformed/unreachable URL and confirm only the IP field shows "unavailable" while codec/bitrate/country (if present) still render.

## Suggested Review Order

**Backend: event emission, off the playback-critical path**

- Entry point — the dedup check, the synchronous bad-URL emit, and the `tokio::spawn`'d DNS lookup guarded by `is_current_generation` before it emits.
  [`player.rs:439`](../../winradio/src-tauri/src/audio/player.rs#L439)

- Shared envelope struct — `data` is a bare string here, unlike Location's/Weather's multi-field `data`.
  [`player.rs:96`](../../winradio/src-tauri/src/audio/player.rs#L96)

- New per-tile mutex, parallel to but independent of Location's/Weather's.
  [`player.rs:128`](../../winradio/src-tauri/src/audio/player.rs#L128)

**Backend: host parsing and DNS**

- The only genuinely fetched field — pure host/port parse split from the blocking DNS lookup.
  [`stream_info.rs:23`](../../winradio/src-tauri/src/stream_info.rs#L23)

**Backend: data model**

- `Station` gains `codec`/`bitrate`, back-compat by `#[serde(default)]` — the fields the tile's other three lines read synchronously.
  [`commands.rs:38`](../../winradio/src-tauri/src/commands.rs#L38)

**Frontend: read-model**

- The new `stream-info-updated` listener — sets the IP directly, no follow-up `invoke()`.
  [`playback.ts:252`](../../winradio/src/stores/playback.ts#L252)

- State shape — just `{status, ip}`, since codec/bitrate/country never ride this event.
  [`playback.ts:63`](../../winradio/src/stores/playback.ts#L63)

- `Station` interface gains the same two fields, threaded through `toggleFavorite`/`toPlayableStation` correctly from the start (avoiding Story 2.2's dropped-fields bug).
  [`stations.ts:23`](../../winradio/src/stores/stations.ts#L23)

**UI**

- The Stream Info Tile — the only one of the three tiles whose main content reads synchronously off `currentStation` rather than a store field.
  [`StreamInfoTile.vue:1`](../../winradio/src/components/StreamInfoTile.vue#L1)

- Wired into the dashboard grid — all three fixed Info Tiles (SM-C1) now complete.
  [`App.vue:169`](../../winradio/src/App.vue#L169)

**Tests**

- `extract_host_and_port`/`resolve_ip` and dedup helper tests.
  [`stream_info.rs:56`](../../winradio/src-tauri/src/stream_info.rs#L56)

- Stream info store tests (populate/unavailable/resets).
  [`playback.test.ts:368`](../../winradio/src/stores/playback.test.ts#L368)
