---
title: 'Weather Tile'
type: 'feature'
created: '2026-09-18'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: '8c9a4d7123d452c7ec3a6ec1be4badc142d210a0'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The dashboard's Weather tile is a reserved-but-empty placeholder; the station's current weather (a live, at-a-glance context signal) is missing.

**Approach:** Mirror Story 2.2's Location Tile pattern — an event-pushed `{ok, data, reason}` envelope (AD-5), fired once per playback attempt from `run_playback`, backed by a new `weather.rs` Open-Meteo client. Unlike Location, the whole payload (current temperature, condition, today's high/low) is small enough to ride inside the event itself — no follow-up command/image-fetch step is needed.

## Boundaries & Constraints

**Always:**
- Fetch only when the station has both `geoLat`/`geoLong` (reuse `directory::location_info_for`'s presence check); otherwise emit `{ok:false, reason:"Weather unavailable"}` immediately, no network call.
- The live Open-Meteo call happens off the playback-critical path: spawn it (`tokio::spawn`) rather than inlining the `.await` in `run_playback`, so a slow/unreachable weather API never delays reaching the retry loop's post-play logic (NFR-3).
- Guard the spawned task with `self.is_current_generation(gen)` before emitting — a station switch mid-fetch must not emit a stale tile's weather.
- Dedup on station id the same way Location does (`last_location_station_id`/`should_emit_location_update`): a drop/reconnect to the *same* station must not re-fetch; a switch to a *different* station always does. Add a parallel `last_weather_station_id` field + `should_emit_weather_update` helper (do not reuse Location's field — they track independent event streams).
- A failed weather fetch (network, non-2xx, bad body) must never fire `playback-error` — same scope isolation as Location/Stream Info.
- `weather-updated` reuses the exact `{ok, data, reason}` shape (`#[serde(rename_all = "camelCase")]`) already anticipated in `player.rs`'s `LocationUpdatedPayload` doc comment.
- Frontend: reset `weather` to idle in the same `play`/`stop`/`playback-error` handlers, at the same point as `location`/`metadata`.

**Ask First:** None — this story's shape (event, dedup, degrade-quietly) is fully determined by Story 2.2's precedent and the frozen epic context.

**Never:**
- No polling, no client-side timer/refresh — one event per attempt, per AD-4/AD-5.
- No new tile beyond the fixed three (SM-C1).
- Do not add weather to local storage (`store.rs`) — ephemeral only, per epic context.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Coords + reachable API | Station has geoLat/geoLong; Open-Meteo responds 200 | `weather-updated{ok:true, data:{temperatureC, condition, forecastHighC, forecastLowC}}`; tile shows conditions + today's high/low | N/A |
| No coordinates | Station missing geoLat or geoLong | `weather-updated{ok:false, reason:"Weather unavailable"}` fired immediately, no fetch | Tile shows "Weather unavailable" |
| Fetch fails | Network error / non-2xx / unparseable body | `weather-updated{ok:false, reason:"Weather unavailable"}` | Tile shows "Weather unavailable"; no `playback-error` |
| Same-station reconnect | Stream drops and reconnects to the same station id | No second `weather-updated` emitted | N/A |
| Rapid station switch mid-fetch | User switches stations while a fetch is in flight | Stale fetch's result is dropped (`is_current_generation` false); only the new station's weather (or its own no-coords/failure state) is shown | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/weather.rs` (new) -- `WeatherInfo { temperature_c, condition, forecast_high_c, forecast_low_c }` (`#[serde(rename_all = "camelCase")]`); `fn build_client() -> Result<reqwest::Client, String>` (own `USER_AGENT` const, mirrors `directory.rs`'s pattern, module-local since no shared-UA requirement exists for Open-Meteo); `async fn fetch_weather(client, lat, long) -> Result<WeatherInfo, String>` (GET `https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={long}&current_weather=true&daily=temperature_2m_max,temperature_2m_min&timezone=auto`, `.text().await`, then pure `parse_weather(&body)`); pure `parse_weather(body: &str) -> Result<WeatherInfo, String>` (serde_json, no reqwest -- unit-testable with canned JSON); pure `condition_from_weather_code(code: u32) -> &'static str` (WMO code -> short description, e.g. 0 => "Clear sky", 61 => "Rain", unknown => "Unknown")
- `winradio/src-tauri/src/main.rs:1` -- add `mod weather;`
- `winradio/src-tauri/src/audio/player.rs:52` -- new `WeatherUpdatedPayload { ok, data: Option<weather::WeatherInfo>, reason: Option<String> }`; new const `WEATHER_UNAVAILABLE: &str = "Weather unavailable"`
- `winradio/src-tauri/src/audio/player.rs:84` -- `RadioPlayer` += `last_weather_station_id: Mutex<Option<String>>` field, initialized `None` in `new()` (mirrors `last_location_station_id`)
- `winradio/src-tauri/src/audio/player.rs:322` -- right after the existing Location block (still inside the `Ok(join_handle)` arm, before `let episode_started = ...`), add the weather block: dedup check via `should_emit_weather_update`, then either an immediate no-coords emit or a `tokio::spawn`'d fetch-and-emit guarded by `self.is_current_generation(gen)`
- `winradio/src-tauri/src/audio/player.rs:541` (near `should_emit_location_update`) -- new pure `fn should_emit_weather_update(last_emitted_for: &mut Option<String>, station_id: &str) -> bool` (identical logic, separate fn/field so Location and Weather dedup independently)
- `winradio/src/stores/playback.ts:19` -- `WeatherEventData { temperatureC, condition, forecastHighC, forecastLowC }`, `WeatherState { status: 'idle'|'ok'|'unavailable', temperatureC, condition, forecastHighC, forecastLowC }` (all null when not `ok`), `emptyWeather()`
- `winradio/src/stores/playback.ts:81,112,132` -- reset `weather.value = emptyWeather()` alongside the existing `location.value = emptyLocation()` resets in the `play`/`stop`/`playback-error` listeners
- `winradio/src/stores/playback.ts:146` -- new `listen<{ok, data: WeatherEventData|null, reason: string|null}>('weather-updated', ...)` -- sets `weather.value` directly from the envelope (`ok` -> populated state, else `unavailable`); no follow-up `invoke` (unlike Location's tile-image fetch)
- `winradio/src/stores/playback.ts:301` -- export `weather` in the store's return object
- `winradio/src/components/WeatherTile.vue` (new) -- mirrors `LocationTile.vue`: `<InfoTile title="Weather" :placeholder="...">`, shows temperature + condition + today's high/low when `status === 'ok'`, else `InfoTile`'s placeholder "Weather unavailable"
- `winradio/src/App.vue:167,197` -- add `<WeatherTile />` next to `<LocationTile />`; narrow `remainingInfoTiles` to `['Stream Info']`

## Tasks & Acceptance

**Execution:**
- [x] `weather.rs` -- `WeatherInfo`, `build_client`, `fetch_weather`, `parse_weather`, `condition_from_weather_code` -- pure/testable Open-Meteo client, no shared state
- [x] `main.rs` -- register `mod weather;` -- makes the module available to `player.rs`
- [x] `player.rs` -- `WeatherUpdatedPayload`, `last_weather_station_id`, `should_emit_weather_update`, emit block in `run_playback` -- wires the event into the existing play lifecycle without blocking it
- [x] `playback.ts` -- `weather` state + listener + resets -- read-model for the new event
- [x] `WeatherTile.vue`, `App.vue` -- render the tile, replace the reserved placeholder -- the actual UI this story delivers
- [x] Unit tests -- `parse_weather` (valid/malformed body), `condition_from_weather_code` (known/unknown codes), `should_emit_weather_update` (first call / same-station suppress / different-station emit), `playback.ts` weather reset/populate/fail paths -- covers the I/O matrix

**Acceptance Criteria:**
- Given a station with coordinates and a reachable weather API, when it plays, then the tile shows current temperature, condition, and today's high/low
- Given a station with no coordinates, or a fetch failure, when the tile renders, then it shows a quiet "Weather unavailable" placeholder -- never blocks playback or the other two tiles
- Given a stream that drops and reconnects to the same station, when it resumes, then no duplicate `weather-updated` fetch/emit occurs

## Spec Change Log

- **2026-09-18, step-04 review (no code changes):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. Zero `patch`/`bad_spec`/`intent_gap` findings survived — the implementation matches the spec's required design exactly, including the `tokio::spawn` + `is_current_generation` guard for the async weather fetch. Rejected as noise or already-correct-by-design: `WeatherTile.vue`'s non-null assertions (safe by construction, mirrors `LocationTile.vue`), the Open-Meteo 200-with-error-body case (already resolves correctly to "Weather unavailable" via the existing parse-error path), NaN/boundary-code/reason-differentiation findings (intentional per the frozen "never a partial state" constraint), and both edge-case-hunter's synchronous-block race findings (mirrors Location's already-shipped identical pattern, negligible in practice). 5 non-blocking findings routed to `deferred-work.md`: no `WeatherTile.vue` component test, no mocked-success-path test for `fetch_weather`, no integration test for the `None`-coordinates branch or the rapid-switch generation guard through `run_playback` (I manually verified this guard's correctness by reading `player.rs:344-414` — the gap is coverage, not a known defect), a cosmetic "-0°C" rounding display edge case, and no request-cancellation/backoff for rapid repeated Open-Meteo calls (mirrors Location's already-deferred equivalent). KEEP: the architecture (event-pushed `{ok, data, reason}` envelope, payload riding entirely inside the event with no follow-up command, independent per-tile dedup state, `tokio::spawn` off the playback-critical path) was sound as specified — no redesign needed.

## Design Notes

Open-Meteo's `current_weather=true&daily=temperature_2m_max,temperature_2m_min&timezone=auto` gives everything needed in one call -- no second request for the daily range. WMO weather codes (the `weathercode` field) are a small fixed integer set; `condition_from_weather_code` only needs enough branches to cover the common ones (clear/cloudy/rain/snow/thunderstorm) plus an `_ => "Unknown"` fallback, same spirit as `tile_url`'s well-documented-formula note in spec-2-2 -- no need to enumerate all ~30 WMO codes here.

## Verification

**Commands:**
- `cargo test` (via the MSVC toolchain, not the default Bash-tool one) -- all existing + new tests pass
- `cargo build` -- clean build, `weather.rs` compiles and is wired into `main.rs`
- `npm run build` -- clean Vue/TS build
- `npx vitest run` -- all existing + new tests pass

**Manual checks (if no CLI):**
- Play a station with coordinates against the real backend (or Pinia-injected in the dev server) and confirm the Weather tile populates; play one without coordinates and confirm it shows "Weather unavailable".

## Suggested Review Order

**Backend: event emission, off the playback-critical path**

- Entry point — the dedup check, the no-coordinates immediate emit, and the `tokio::spawn`'d fetch guarded by `is_current_generation` before it emits.
  [`player.rs:347`](../../winradio/src-tauri/src/audio/player.rs#L347)

- Pure dedup decision, unit-tested in isolation — identical contract to Location's, deliberately not sharing state with it.
  [`player.rs:645`](../../winradio/src-tauri/src/audio/player.rs#L645)

- New per-tile mutex, parallel to but independent of `last_location_station_id`.
  [`player.rs:106`](../../winradio/src-tauri/src/audio/player.rs#L106)

- Shared `{ok, data, reason}` envelope struct — same shape as Location's, now with `weather::WeatherInfo` as its `data`.
  [`player.rs:77`](../../winradio/src-tauri/src/audio/player.rs#L77)

**Backend: the Open-Meteo client**

- The live network call, split from the pure parser so failure modes are unit-testable without a network dependency.
  [`weather.rs:127`](../../winradio/src-tauri/src/weather.rs#L127)

- Pure JSON→DTO mapping — the actual shape assumptions about Open-Meteo's response.
  [`weather.rs:83`](../../winradio/src-tauri/src/weather.rs#L83)

- WMO weather-code → short description mapping, deliberately partial with a fallback.
  [`weather.rs:66`](../../winradio/src-tauri/src/weather.rs#L66)

- Module registration.
  [`main.rs:12`](../../winradio/src-tauri/src/main.rs#L12)

**Frontend: read-model**

- The new `weather-updated` listener — sets state directly from the event, no follow-up `invoke()` unlike Location.
  [`playback.ts:207`](../../winradio/src/stores/playback.ts#L207)

- State shape and reset helper.
  [`playback.ts:36`](../../winradio/src/stores/playback.ts#L36)

**UI**

- The Weather Tile component — mirrors `LocationTile.vue`'s shell/placeholder conventions.
  [`WeatherTile.vue:1`](../../winradio/src/components/WeatherTile.vue#L1)

- Wired into the dashboard grid, narrowing the reserved-placeholder list.
  [`App.vue:168`](../../winradio/src/App.vue#L168)

**Tests**

- Dedup helper tests.
  [`player.rs:780`](../../winradio/src-tauri/src/audio/player.rs#L780)

- Weather store tests (populate/unavailable/resets).
  [`playback.test.ts:300`](../../winradio/src/stores/playback.test.ts#L300)
