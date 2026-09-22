---
title: 'Tray: Dynamic Play/Pause Label'
type: 'feature'
created: '2026-09-22'
status: 'done'
route: 'oneshot'
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The system tray's "Play/Pause" menu item (`winradio/src-tauri/src/tray.rs:8`) is a static label that never changes — it always reads "Play/Pause" regardless of whether the app is actually playing, paused, reconnecting, or has hit a playback error, so the tray gives the user no indication of current state.

**Approach:** Update the tray menu item's title live from the same central point `RadioPlayer` already uses to notify the frontend of state changes (`emit()` in `winradio/src-tauri/src/audio/player.rs`) — every trigger source (tray click, transport bar button, media key) already funnels through this one function, so hooking the tray-title update in there covers all of them uniformly with no per-call-site duplication. Label reads "Pause" once actually playing (the `play` event), and "Play" for every non-playing state (`stop`, `reconnecting`, `playback-error`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Playback starts (fresh play or reconnect succeeds) | `play` event emitted | Tray item reads "Pause" | N/A |
| User stops playback | `stop` event emitted | Tray item reads "Play" | N/A |
| Stream drops and is retrying | `reconnecting` event emitted | Tray item reads "Play" (not yet actually playing) | N/A |
| All retries exhausted | `playback-error` event emitted | Tray item reads "Play" | N/A |
| Fresh app launch, nothing ever played | No station loaded | Tray item reads "Play" (initial label, matches the not-playing state) | N/A |

</frozen-after-approval>

## Implementation Notes

Implemented as designed: `winradio/src-tauri/src/tray.rs` gained a `PLAY_PAUSE_ITEM_ID` const (initial label "Play"); `winradio/src-tauri/src/audio/player.rs`'s `emit()` now calls `sync_tray_label`, which delegates to a pure `tray_label_for(event) -> Option<&'static str>` function (extracted during review so the event->label mapping is unit-testable without a live `AppHandle`/tray). 3 new tests, `cargo build`/`cargo test` clean (53/53, up from 50).

## Review Triage Log

- `patch` — Event->label mapping had no unit test; a future edit to the match arms could silently drift. Fixed by extracting `tray_label_for` as a pure free function with 3 new tests.
- `patch` — No comment explaining the `stop`/`reconnecting`/`playback-error` -> "Play" collapse would need revisiting if a genuine (non-stop) pause state is ever added. Fixed with a doc comment on `tray_label_for`.
- `patch` — No cross-reference noting `sync_tray_label` duplicates the event->state knowledge `playback.ts`'s Vue store already encodes independently (two runtimes, no shared source of truth). Fixed with a doc comment on `sync_tray_label`.
- `low`, rejected — `set_title`'s `Result` is discarded with `let _ =`. Matches this file's and `tray.rs`'s existing convention throughout (e.g. `window.show()`/`emit_all()` results are already discarded the same way) — not a new inconsistency.
- `maybe-false`, deferred — Theoretical tray-label ordering race across overlapping generations. On inspection, `play()`/`stop()` bump the generation with no `await` before their `emit` call, and the `play` emit is itself gated on `is_current_generation` immediately before firing — same event-ordering assumption the frontend's own listeners already rely on. Narrow-to-nonexistent in practice; recorded in `deferred-work.md` rather than fixed since it isn't verified reproducible.
- `medium`, deferred (pre-existing, not caused by this change) — Tray item isn't disabled when no station has ever been loaded; clicking it silently no-ops. The more accurate dynamic label makes this pre-existing gap more noticeable, so it's recorded in `deferred-work.md` even though this spec didn't cause it.
