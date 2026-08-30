---
title: 'Sleep Timer'
type: 'feature'
created: '2026-08-30'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: '21ab39426ad2aed01c0aac9c1d731c94b0eb62d5'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `SleepTimer::set()` exists in Rust (cancel-then-arm via a oneshot channel, stops playback on fire) but emits no events at all — there's no way for the frontend to know a timer is armed, was cancelled, or fired. The frontend has zero sleep-timer UI: no icon, no duration picker, no state.

**Approach:** Add armed-state events to `SleepTimer` (mirroring `RadioPlayer`'s established `AppHandle`/`emit` pattern, per AD-4) so the frontend can track armed/cleared state. Build the transport bar's Sleep Timer icon + duration popover per DESIGN.md/EXPERIENCE.md: preset durations (15/30/60/90 min, pre-filled from `sleepTimerDefaultMinutes`), a small active-badge on the icon when armed (no persistent countdown), and clicking the badged icon reopens the same popover with a "Cancel timer" option instead of the preset list.

## Boundaries & Constraints

**Always:** No persistent countdown anywhere — only a small badge indicating armed/not-armed. Firing pauses playback via the existing `stop` mechanism with no special modal or notification. Duration options are exactly 15/30/60/90 minutes, pre-filled from `sleepTimerDefaultMinutes`. Clicking the badged icon reopens the same popover component with a "Cancel timer" option — not a separate menu, not a silent toggle-off.

**Ask First:** None identified — the icon/badge/popover/cancel interaction and preset durations are all fixed by DESIGN.md/EXPERIENCE.md.

**Never:** Don't add a countdown display of any kind. Don't add a "timer fired" modal or toast. Don't let manually pausing/playing elsewhere silently cancel an armed timer — the timer is independent, cleared only by explicit cancel or by firing.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Arm from idle | Click Sleep Timer icon, pick a duration | Icon shows active badge; picker closes | N/A |
| Fires | Duration elapses | Playback stops normally (existing `stop` event); badge clears | N/A |
| Cancel before firing | Click badged icon, click "Cancel timer" | Badge clears, no playback change | N/A |
| Re-arm while already armed | Click badged icon, pick a new duration instead of cancelling | Old timer is superseded; badge reflects the new duration | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/timer.rs` -- add `app_handle: Arc<Mutex<Option<AppHandle>>>` field + `set_app_handle()` (mirror `RadioPlayer`'s pattern, `audio/player.rs:57,102-104`); `set()` emits `sleep-timer-armed { minutes }` when arming (`minutes > 0`) and `sleep-timer-cleared` when cancelling (`minutes == 0`) or when the spawned task's sleep actually elapses (after calling `player.stop()`) -- no event needed on the interrupted/superseded branch of the internal `tokio::select!`, since the superseding call already emits its own event
- `winradio/src-tauri/src/main.rs` -- call `sleep_timer.set_app_handle(handle.clone())` in `.setup()`, alongside the existing `audio_player_for_setup.set_app_handle(...)` call
- `winradio/src/stores/playback.ts` -- add `sleepTimerArmed`/`sleepTimerMinutes` state; `initListeners` adds `sleep-timer-armed`/`sleep-timer-cleared` listeners; add `armSleepTimer(minutes)`/`cancelSleepTimer()` actions invoking `set_sleep_timer` (cancel = `minutes: 0`, matching the existing command's semantics)
- `winradio/src/components/TransportBar.vue` -- add the Sleep Timer icon button (moon/clock icon, DESIGN.md) with a small active-badge dot when `sleepTimerArmed`; add a popover: preset buttons (15/30/60/90 min, the one matching `settingsStore.sleepTimerDefaultMinutes` visually highlighted) when not armed, a single "Cancel timer" row when armed

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/timer.rs` -- add `AppHandle` field/setter, emit `sleep-timer-armed`/`sleep-timer-cleared` -- gives the frontend visibility into timer state
- [x] `src-tauri/src/main.rs` -- wire `sleep_timer.set_app_handle(...)` in setup -- required for the above to actually emit
- [x] `src/stores/playback.ts` -- add sleep-timer state, listeners, `armSleepTimer`/`cancelSleepTimer` actions -- frontend read-model for the new events
- [x] `src/components/TransportBar.vue` -- build the icon, badge, and popover (presets vs. cancel) -- the actual UI this story delivers
- [x] Unit/integration test -- `armSleepTimer`/`cancelSleepTimer` update `sleepTimerArmed`/`sleepTimerMinutes` correctly from the corresponding events -- covers the new store logic

**Acceptance Criteria:**
- Given the transport bar's Sleep Timer icon, when Javad picks a duration, then it arms and the icon shows a small active badge — no persistent countdown
- Given an armed timer, when it fires, then playback pauses normally (no special modal)
- Given an armed timer, when Javad clicks the badged icon before it fires, then it offers cancel

## Spec Change Log

- **2026-08-30, step-04 review (patch findings):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. 3 confirmed `patch` findings sent back to the implementer and fixed: (1) the popover, when armed, showed only "Cancel timer" with no way to re-arm directly — a direct violation of this spec's own frozen I/O matrix row ("Re-arm while already armed"), since the Rust backend already supersedes an active timer correctly but the UI never reached that call while armed — fixed by always showing the presets, with "Cancel timer" appended (not replacing them) when armed; (2) `timer_handle` was never reset to `None` when a timer fired naturally, a latent trap for any future status-query code — fixed by clearing it in the sleep-elapsed branch; (3) `SleepTimerArmedPayload`'s wire shape had no test pinning it, so a future field rename could silently break the frontend's `event.payload.minutes` read with zero test failure — fixed with a serde round-trip test. Re-verified: `cargo build`/`cargo test` (26/26 passing, up from 25) and `npm run build`/`npx vitest run` (24/24 passing, unchanged since these 3 fixes didn't touch the store) all pass after the patch round. KEEP: the original architecture (cancel-then-arm via oneshot channel, `AppHandle`/`emit` pattern mirroring `RadioPlayer`, read-model-only frontend store) was sound — all 3 fixes are localized hardening, not redesigns. 6 additional non-blocking findings (no status-query command, no component-level test tooling, optimistic actions with no user-visible error, no guard against rapid overlapping invokes, incomplete popover focus management, no fake-time test for the supersede race) were routed to `deferred-work.md` rather than fixed now.

## Verification

**Commands:**
- `cd winradio/src-tauri && cargo build` -- expected: compiles clean with the new events
- `cd winradio/src-tauri && cargo test` -- expected: all tests pass, no regressions
- `cd winradio && npm run build` -- expected: type-checks and builds
- `cd winradio && npx vitest run` -- expected: existing + new tests pass

**Manual checks (if no CLI):**
- Launch `tauri dev`: arm a short timer, confirm the badge appears; click the badged icon and cancel, confirm the badge clears; arm again and let it fire, confirm playback stops with no modal and the badge clears

## Suggested Review Order

**Rust timer: arm/cancel/fire and the event wiring**

- `set()`: cancel-then-arm via the oneshot channel, emitting `sleep-timer-armed`/`sleep-timer-cleared`.
  [`timer.rs:52`](../../winradio/src-tauri/src/timer.rs#L52)

- The sleep-elapsed branch: fires, stops playback, clears `timer_handle` (the patch fix), then emits cleared.
  [`timer.rs:75`](../../winradio/src-tauri/src/timer.rs#L75)

- `set_app_handle`, mirroring `RadioPlayer`'s established pattern.
  [`timer.rs:42`](../../winradio/src-tauri/src/timer.rs#L42)

- `main.rs`'s setup wiring for the above.
  [`main.rs:32`](../../winradio/src-tauri/src/main.rs#L32)

**Frontend: the re-arm-while-armed fix and store wiring**

- The popover: presets always shown, "Cancel timer" appended (not replacing) when armed — the fix for the frozen I/O matrix violation.
  [`TransportBar.vue:78`](../../winradio/src/components/TransportBar.vue#L78)

- `armSleepTimer`/`cancelSleepTimer`: not optimistic, both routes through the same `set_sleep_timer` command.
  [`playback.ts:199`](../../winradio/src/stores/playback.ts#L199)

- The `sleep-timer-armed`/`sleep-timer-cleared` listeners.
  [`playback.ts:100`](../../winradio/src/stores/playback.ts#L100)

**Peripherals**

- New Rust test pinning the payload's wire shape.
  [`timer.rs:99`](../../winradio/src-tauri/src/timer.rs#L99) (test module)

- New frontend tests: arm/clear/cancel behavior.
  [`playback.test.ts:1`](../../winradio/src/stores/playback.test.ts#L1)
