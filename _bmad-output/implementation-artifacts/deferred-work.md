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

- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-search-browse-stations.md`
  summary: Add a frontend test runner (vitest) and cover `winradio/src/stores/search.ts`'s debounce/dedup/`requestSeq` race-guard logic — currently correct by direct code inspection, but with zero regression-safety net.
  evidence: Code review of Story 1.2 (verification-gap layer) found three distinct untested-but-correct-today mechanisms in `search.ts`: the zero-match/offline status mapping, the `{query,filters}` signature dedup (AD-13), and the stale-response `requestSeq` guard. Each has a concrete one-line regression that would ship silently since the repo has no frontend test infrastructure at all (confirmed: no vitest/jest dependency, no `*.test.ts`/`*.spec.ts` files anywhere).

- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-search-browse-stations.md`
  summary: Superseded in-flight `search_stations`/`get_filter_options` Tauri invocations aren't cancelled, only their results ignored — a fast sequence of filter changes can fire several real HTTP requests to Radio-Browser that are guaranteed to be discarded.
  evidence: Code review found `search.ts`'s `requestSeq` guard discards a stale *response* but the underlying `invoke()` call (and the Rust-side HTTP request it triggers) still runs to completion. Fixing this properly needs a cancellation mechanism plumbed through Tauri's IPC (e.g. an abort token passed to the Rust command), which is more design work than a trivial patch — and debouncing already covers the common typed-text case, so real-world impact is limited to rapid successive filter-select changes.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-search-browse-stations.md`
  summary: `search_stations`/`get_filter_options` build a fresh `reqwest::Client` per call instead of reusing one via Tauri managed state — minor connection-reuse inefficiency (no TLS/connection-pool reuse across searches).
  evidence: Code review of `winradio/src-tauri/src/directory.rs`'s `build_client()` found it's invoked fresh inside both `search_stations_at` and `get_filter_options_at`. Low real-world impact for a single-user desktop app's search frequency, but worth fixing if this pattern is reused for Epic 2's weather/location clients.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-2-search-browse-stations.md`
  summary: Filter dropdowns (genre/country/language selects in `SearchPanel.vue`) show no loading/error/retry state while `get_filter_options` is in flight or after it fails — they just render "(any)" indistinguishably from "nothing available."
  evidence: Code review found `loadFilterOptions` logs failures to console but the UI never reflects `filterOptionsLoading`/a failed load, and there's no retry affordance once the first mount's fetch fails. Minor UX polish, not a functional defect.
