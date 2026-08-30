---
title: 'Transport Controls: Play/Pause/Volume/Mute'
type: 'feature'
created: '2026-08-30'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: '8881955430e02dbfa3072dbb37d80359f67639d9'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Of this story's four ACs, three are already fully satisfied by Story 1.1's work (volume persist-on-release, mute/unmute exact-restore, reconnect/error-state UI) — confirmed by direct code inspection, not assumed. The fourth is genuinely unbuilt: on relaunch, the last-played station's info should show idle in the Now-Playing area (not auto-playing) with the last-set volume restored. Volume restore already works (seeded from `Settings.volume` at boot); showing the last station's *name/info* does not — nothing persists which station was last played, and nothing restores it into `currentStation` on mount.

**Approach:** Add `last_station: Option<Station>` to the existing `Settings` struct (Rust + TS) — reusing all existing `load_settings`/`save_settings` plumbing rather than adding new commands or a new persisted field elsewhere. Persist it as a full station snapshot, not just an id: a last-played station that was never favorited has no other persisted record once the app restarts (search results are never written to `store.json`), so an id-only reference would be unresolvable for that case. Update it from the frontend's `play` event listener (mirroring how `save_stations`/`save_settings` are already frontend-triggered persistence calls); restore it into `playbackStore.currentStation` on mount without calling `play()`, so the dashboard shows it idle.

## Boundaries & Constraints

**Always:** Restoring the last-played station on mount sets `currentStation` only — never calls `play()`, never sets `isPlaying`. Every `save_settings` call reconstructs the *complete* current in-memory settings snapshot (`minimizeToTray`, `sleepTimerDefaultMinutes`, `theme`, `volume`, `lastStation`) — never a partial object that could silently blank out fields it doesn't know about. Volume restore, mute/unmute, and reconnect/error states must keep passing their existing (already-correct) behavior unchanged.

**Ask First:** None — the persistence-shape decision (full station snapshot on `Settings`, not just an id) is justified above and doesn't require new commands or schema elsewhere.

**Never:** Don't add a new Tauri command for this — extend the existing `Settings` struct/`save_settings`/`load_settings` round-trip. Don't auto-play the restored station under any circumstance. Don't regress AC1-3's already-correct behavior while wiring AC4's cross-store update.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Relaunch after playing a Favorite | `Settings.lastStation` set to that station | Dashboard shows its name/category idle (not playing), last volume restored | N/A |
| Relaunch after playing a never-favorited search result | `Settings.lastStation` holds the full search-result snapshot (not just an id) | Same idle restore — works identically, no dependency on Favorites membership | N/A |
| Fresh install, nothing ever played | `Settings.lastStation` is `None` | Dashboard shows "No station selected" (existing default) | N/A |
| Settings modal changes theme while a station has been played this session | `saveSettings()` invoked by the modal | `lastStation` (and all other in-memory settings fields) round-trip unchanged — modal's save never blanks it | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/commands.rs:24-42` -- `Settings` struct: add `#[serde(default)] pub last_station: Option<Station>`; update `impl Default for Settings` accordingly
- `winradio/src/stores/settings.ts:7-12,18-56` -- `Settings` interface add `lastStation?: Station`; store add `lastStation` ref, populate it in `loadSettings`, add `setLastStation(station)` action that updates the ref then calls `saveSettings()` reconstructing the *full* current snapshot (not a partial object) -- update the file's existing "only one writer" comment to reflect that playback now also writes this one field
- `winradio/src/stores/playback.ts` -- in the `play` event listener (where `currentStation.value = event.payload` already happens), also call `settingsStore.setLastStation(event.payload)`; add a `restoreLastStation(station)` action that sets `currentStation.value = station` only (no `isPlaying`, no `play()` call, no metadata)
- `winradio/src/App.vue` -- in `onMounted`, after `settingsStore.loadSettings()` resolves, call `playbackStore.restoreLastStation(settingsStore.lastStation)` if present -- the existing dashboard template (`v-if="playbackStore.currentStation"`) already renders idle correctly once `currentStation` is set, no template change needed
- Verify (no code change expected, confirm only): `winradio/src/components/TransportBar.vue:52-53,70-81`, `winradio/src/stores/playback.ts` (`persistVolume`, `toggleMute`), `winradio/src-tauri/src/audio/player.rs` (`RETRY_DELAYS`, `run_playback`'s reconnecting/playback-error emission) -- AC1-3, already correct

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands.rs` -- add `last_station: Option<Station>` to `Settings` with `#[serde(default)]` -- back-compat safe, no new command
- [x] `src/stores/settings.ts` -- add `lastStation` state, `setLastStation`, ensure `saveSettings` always reconstructs the full snapshot -- core persistence logic
- [x] `src/stores/playback.ts` -- wire `play` event to `setLastStation`; add `restoreLastStation` (sets `currentStation` only, never plays) -- the actual behavior this story adds
- [x] `src/App.vue` -- call `restoreLastStation` after settings load in `onMounted` -- wires the restore into app startup
- [x] Verify (existing behavior, no changes expected) -- volume persist-on-release, mute/unmute exact-restore, reconnect/error-state UI -- confirm no regression while wiring the above
- [x] Unit/integration test -- `setLastStation`/`saveSettings` reconstructs a full snapshot without dropping other fields; `restoreLastStation` sets `currentStation` without setting `isPlaying` -- covers the new cross-store logic

**Acceptance Criteria:**
- Given a station is loaded, when Javad drags the volume slider, then it updates live and persists on release
- Given audio is playing, when Javad clicks mute then unmute, then it restores the exact prior volume
- Given a stream drops mid-playback, when WinRadio retries (immediate, then 2s/5s/10s backoff, ~20s budget), then the transport bar shows "Reconnecting…", and if all retries fail it shows "Couldn't play this station" in error styling — distinct from a normal pause
- Given the app relaunches, when the dashboard loads, then the last-played station's info shows idle in the Now-Playing area (not auto-playing) with the last-set volume restored

## Spec Change Log

- **2026-08-30, step-04 review (patch findings):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. 4 confirmed `patch` findings sent back to the implementer and fixed: (1) `setLastStation` read a stale, settings-store-local `volume` ref instead of the live playback volume — fixed by requiring the caller to pass the live value explicitly (removing the stale ref entirely), same as `persistVolume`/the settings modal already do; (2) a malformed nested `lastStation` value could reset the *entire* `Settings` record (theme/volume/minimizeToTray/sleepTimerDefault too, not just drop `lastStation`) since it was decoded as one unit — fixed by decoding it independently with its own fallback, same fix shape as the earlier stations/settings split; (3) `restoreLastStation` had no guard against clobbering an already-playing station's display — added a defensive `isPlaying` check, cheap insurance even though today's control flow can't currently trigger the race; (4) a Rust test named "round_trips" didn't actually round-trip (deserialize-only) — renamed accurately and added a genuine save→reload round-trip test via a new test-only `Store::new_at(path)` constructor. Re-verified: `cargo build`/`cargo test` (24/24 passing, up from 22) and `npm run build`/`npx vitest run` (21/21 passing, up from 19) all pass after the patch round. KEEP: the original design (full station snapshot on `Settings` rather than an id, frontend-triggered persistence mirroring `save_stations`/`save_settings` precedent, restore-without-play semantics) was sound — all 4 fixes are localized hardening, not redesigns. 5 additional non-blocking findings (redundant writes on reconnect, no write-coalescing on rapid switches, non-atomic `Store::save()`, potential snapshot/live-data divergence, no App.vue integration test) were routed to `deferred-work.md` rather than fixed now.

## Verification

**Commands:**
- `cd winradio/src-tauri && cargo build` -- expected: compiles clean with the new `last_station` field
- `cd winradio/src-tauri && cargo test` -- expected: all tests pass, no regressions
- `cd winradio && npm run build` -- expected: type-checks and builds
- `cd winradio && npx vitest run` -- expected: existing + new tests pass

**Manual checks (if no CLI):**
- Launch `tauri dev`, play a station (favorite or search result), quit, relaunch: confirm the dashboard shows that station's name idle (not playing) with the same volume; confirm dragging the volume slider updates live and only writes to disk on release; confirm mute then unmute restores the exact volume; confirm reconnect/error states still render correctly

## Suggested Review Order

**Last-played station persistence (the story's new behavior)**

- `Settings` gains `last_station: Option<Station>` — a full snapshot, not just an id, since a never-favorited station has no other persisted record.
  [`commands.rs:31`](../../winradio/src-tauri/src/commands.rs#L31)

- `parse_store_data` decodes `lastStation` independently from the rest of `Settings`, so a malformed snapshot can never reset the user's other settings.
  [`store.rs:59`](../../winradio/src-tauri/src/store.rs#L59)

- The `play` event listener persists the last-played station with the live volume, not a stale cached one.
  [`playback.ts:59`](../../winradio/src/stores/playback.ts#L59)

- `setLastStation` requires the caller to pass the live volume explicitly — the fix for the stale-volume patch finding.
  [`settings.ts:66`](../../winradio/src/stores/settings.ts#L66)

- `restoreLastStation`: sets `currentStation` only, guarded against clobbering an already-playing station.
  [`playback.ts:109`](../../winradio/src/stores/playback.ts#L109)

- `App.vue`'s startup wiring: restore runs only after settings have loaded.
  [`App.vue:267`](../../winradio/src/App.vue#L267)

**Verified unchanged (AC1-3, already correct from Story 1.1)**

- Volume slider: live update on `input`, persist on `change`.
  [`TransportBar.vue:52`](../../winradio/src/components/TransportBar.vue#L52)

- Mute/unmute exact-restore.
  [`playback.ts:140`](../../winradio/src/stores/playback.ts#L140)

- Reconnecting/error-state backoff schedule.
  [`player.rs:23`](../../winradio/src-tauri/src/audio/player.rs#L23)

**Peripherals**

- New Rust tests: independent `lastStation` decode failure, and a genuine save→reload round trip.
  [`store.rs:1`](../../winradio/src-tauri/src/store.rs#L1) (test module at file end)

- New frontend tests: live-volume assertion, defensive-guard behavior.
  [`playback.test.ts:1`](../../winradio/src/stores/playback.test.ts#L1)
