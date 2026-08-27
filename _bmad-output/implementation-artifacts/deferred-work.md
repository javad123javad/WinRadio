- source_spec: `_bmad-output/implementation-artifacts/spec-1-1-reliable-single-station-playback-foundation-rescue.md`
  summary: Distinguish retryable connect errors (network/timeout) from permanent ones (malformed URL, unsupported codec) so a clearly-bad station fails fast instead of burning the full ~20s backoff schedule.
  evidence: Code review of Story 1.1 (`winradio/src-tauri/src/audio/player.rs`, `try_connect_and_play`/`retry_with_backoff`) found all connect failures are retried identically regardless of cause. Not required by the story's spec, but a reasonable reliability/UX improvement.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-1-reliable-single-station-playback-foundation-rescue.md`
  summary: Confirm/harden how a non-2xx HTTP response from a station URL (404/500) surfaces to the user — currently relies entirely on `stream-download`'s own error surfacing, unverified in this review.
  evidence: Code review of `winradio/src-tauri/src/audio/player.rs:303-305` (`HttpStream::new`) found no explicit status-code check, unlike the pre-rescue code which did check `response.status().is_success()`. Worth confirming the new pipeline still produces a clear "station unreachable" message rather than an opaque decode error.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-1-reliable-single-station-playback-foundation-rescue.md`
  summary: `RadioPlayer::new()` panics (crashing the whole app at startup) if no audio output device is available, instead of degrading gracefully with a "no audio device" state.
  evidence: Code review found `winradio/src-tauri/src/audio/player.rs:89` uses `.expect("Failed to create audio output stream")` on the audio-thread handoff channel. A real but edge-case scenario (no speakers/audio driver failure); fixing it well requires a UX decision on what a "no audio device" degraded state looks like, which is a design call beyond a trivial patch.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-1-reliable-single-station-playback-foundation-rescue.md`
  summary: Add `tracing`/`log` instrumentation to the Rust audio pipeline (connect attempts, retries, decode errors) for production diagnostics.
  evidence: Code review found zero logging/tracing anywhere in `winradio/src-tauri/src`; failures are only visible via one-line `String` errors returned to the frontend. Will make diagnosing real-world connection/reconnect issues difficult once shipped — directly relevant given this story's whole purpose is playback reliability.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-1-reliable-single-station-playback-foundation-rescue.md`
  summary: `export_stations`/`clear_all_data` Tauri commands remain registered (`winradio/src-tauri/src/main.rs`) but have no UI entry point after this story's Settings modal intentionally dropped the old Data section — either wire up a Data section in a later story or remove the now-dead commands.
  evidence: Code review confirmed `commands::export_stations`/`commands::clear_all_data` are still in `main.rs`'s `invoke_handler`, but the rewritten `SettingsModal.vue` (correctly, per this story's spec) only has minimizeToTray/sleepTimerDefaultMinutes/theme rows — no Data section calls these commands anymore.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-1-reliable-single-station-playback-foundation-rescue.md`
  summary: Story 1.5 should implement "on relaunch, last-played station shows idle with last volume restored" as its own acceptance criterion — this was mistakenly drafted into Story 1.1's spec and then correctly removed after the implementer flagged it as unbuilt.
  evidence: epics.md's Story 1.5 AC states this exact behavior; Story 1.1's own four ACs do not include it. Confirmed with the human during Story 1.1's build (see spec's Spec Change Log) and removed from Story 1.1's scope rather than implemented there.
