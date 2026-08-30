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

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: UI state (Now-Playing card, `StationRow` highlight, transport bar) only updates from Rust-pushed `play`/`reconnecting`/`playback-error` events, never optimistically on click — so a slow connect, or a click landing while the previous station is mid-backoff, can make a click look like it did nothing until the event arrives; a superseded station's stale `reconnecting`/error state can also briefly show under the wrong station name.
  evidence: Code review (verification pass on the existing click-to-play mechanism, no new code this story) traced `playbackStore.play()` (`winradio/src/stores/playback.ts`) and confirmed it sets no local state before `invoke()` resolves via a backend event. Pre-existing since Story 1.1's event model; the underlying generation-based audio replacement is correct and immediate, this is purely a UI-feedback-latency gap.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: A superseded generation's `run_playback` retry loop can keep sleeping through its backoff schedule (up to ~17s) after being abandoned, purely wasting a background task, before its `is_current_generation` guard lets it exit; rapid switching between search results can leave several of these idle.
  evidence: Code review of `winradio/src-tauri/src/audio/player.rs`'s `retry_with_backoff`/`run_playback` found no generation check between retry attempts themselves, only at loop/attempt boundaries. Functionally harmless (the guard does eventually stop it, and the new station plays correctly), but worth a cheap fix (check the generation before each sleep) if this pattern is extended.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: Clicking an already-playing row restarts it from scratch (audible cut, elapsed timer reset, metadata cleared) instead of being a no-op; relatedly, `StationRow.vue`'s pause icon on the current row implies a play/pause toggle that doesn't exist — the row's `@click` always emits `play`, never `stop`.
  evidence: Code review found neither `StationRow.vue`, `searchStore.playResult`, nor `stationsStore.playStation` check `isCurrent` before re-triggering `play()`. Pre-existing since Story 1.1's row design (Story 1.2 only reused it as-is, correctly per this story's spec); a real UX inconsistency but not a violation of any stated AC.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: No click-guard/debounce on `StationRow` — rapid clicks (same or different rows) each open a real HTTP connection attempt in Rust before an earlier click's generation is invalidated, real network churn per stray click rather than just wasted CPU.
  evidence: Code review confirmed `try_connect_and_play` (`winradio/src-tauri/src/audio/player.rs`) reaches `HttpStream::new(...).await` before checking supersession again. Pre-existing since Story 1.1; low real-world impact for a single user's deliberate clicks, but worth a debounce if this ever becomes an issue.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: A search-result station played via `toPlayableStation` (`winradio/src/stores/search.ts`) never links back to a matching Favorites entry — Radio-Browser's `stationuuid` and a locally-generated Favorites `id` (e.g. `'soma-groove'`) differ even for the same physical station, so `isCurrent` never highlights the corresponding Favorites row, and `isFavorite` is hardcoded `false` regardless of whether an equivalent station is already saved.
  evidence: Code review traced `search.ts:176-185`'s `toPlayableStation` and `App.vue`'s `isCurrent(id)` check against `stations.ts`'s locally-generated ids. Not required by any stated AC (Story 1.2/1.3 never asked for Search/Favorites cross-highlighting), but a real, user-noticeable inconsistency worth a future story if Favorites ever need to match against Directory-sourced stations.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: `toPlayableStation` drops most of a search result's Directory metadata (`codec`, `bitrate`, `language`, `geoLat`/`geoLong`, `homepage`) and only keeps the first comma-separated tag as `category`, discarding the rest.
  evidence: Code review of `search.ts:176-185` confirmed the adapter maps only `id`/`name`/`url`/`favicon`/a single tag. Acceptable for now since nothing yet reads the dropped fields (Epic 2's Info Tiles will need codec/bitrate/geo but haven't been built), but worth revisiting once those tiles exist so a search-originated "now playing" station isn't missing data a Favorites-originated one has.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: `playbackStore.play()`'s catch block only logs to console on an IPC-level failure (as opposed to a backend-reported `playback-error` event) — no `errorMessage` or any other user-visible state is set, so a failed `invoke('play', …)` call itself produces zero feedback that the click failed.
  evidence: Code review of `winradio/src/stores/playback.ts`'s `play()` found the catch block is `console.error` only. A narrow, rare failure path (IPC serialization/dispatch failure, not a normal playback error), pre-existing since Story 1.1.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-3-preview-play-from-search-results.md`
  summary: `StationRow.vue` has no accessible "currently playing" signal (`aria-pressed`/`aria-current`, or an accessible-name change) beyond a background-color class and a swapped icon — invisible to screen-reader users.
  evidence: Code review confirmed no ARIA state attributes on the row button. Pre-existing since Story 1.1's original row markup, carried over unchanged through extraction in Story 1.2.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-favorites-add-remove-reorder.md`
  summary: `toggleFavorite`/`moveUp`/`moveDown` (`winradio/src/stores/stations.ts`) mutate in-memory state synchronously, then `await saveStations()` with no try/catch and no rollback — if persistence fails (disk error, IPC error), the UI and disk state can silently diverge with no user-visible signal.
  evidence: Code review confirmed none of the three actions catch a `saveStations()` failure. Matches the existing optimistic-persistence pattern used everywhere else in the codebase (e.g. `settings.ts`, `playback.ts`), not something newly introduced by this story — worth addressing project-wide rather than as a one-off fix here.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-favorites-add-remove-reorder.md`
  summary: No guard against overlapping/racing `saveStations()` calls — several fast clicks on the star or up/down arrows each kick off their own persist call, and nothing prevents an older in-flight write from resolving after (and clobbering) a newer one on disk.
  evidence: Code review of `stations.ts`'s `toggleFavorite`/`moveUp`/`moveDown` found each independently awaits its own `saveStations()` with no sequencing/queue. Low real-world impact for a single user's local disk writes, but worth a guard if this pattern is extended to something latency-sensitive.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-favorites-add-remove-reorder.md`
  summary: `StationRow.vue`'s row wrapper (needed to nest real `<button>`s for reorder/favorite) is a `role="button"` div that now contains 3 nested buttons, quadrupling tab stops per Favorites row versus the previous single-button design — worth reconsidering the interaction model (e.g. making the row itself non-focusable and relying on Tab landing on the play/star/reorder buttons directly) in a future accessibility pass.
  evidence: Code review noted the row-plus-3-nested-buttons structure is a real navigation cost for keyboard/screen-reader users, though functional. Not blocking this story's ACs, but a legitimate design question beyond a trivial patch.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-favorites-add-remove-reorder.md`
  summary: `favoriteOrder` values are never compacted after a removal (`nextFavoriteOrder` only computes `max + 1`) — gaps accumulate across repeated add/remove cycles and the counter only grows.
  evidence: Code review confirmed no renumbering/normalization logic exists. Harmless in practice (i64 range, realistic personal-favorites-list sizes), pure tidiness rather than a correctness concern.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-favorites-add-remove-reorder.md`
  summary: No component-level test tooling (`@vue/test-utils`, a DOM environment like jsdom/happy-dom) exists to test `StationRow.vue`'s actual DOM/event behavior directly (disabled states at reorder boundaries, `@click.stop` truly preventing the row's play handler, aria attribute correctness) — the new `vitest` suite only covers pure Pinia store logic.
  evidence: Code review confirmed `stations.test.ts` is the repo's only test file and exercises no DOM. Adding component-testing infrastructure is a reasonable next increment given vitest itself was only just added this story, but is a larger investment beyond this story's scope.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-4-favorites-add-remove-reorder.md`
  summary: `Station.isFavorite` is now provably always `true` for every persisted station, since `toggleFavorite` deletes rather than flipping the flag to `false` — the field is redundant given the current data model (the persisted collection only ever contains favorites).
  evidence: Code review of `stations.ts`'s `toggleFavorite` confirmed removal always deletes the entry rather than setting `isFavorite: false`. Worth reconsidering if/when the data model changes (e.g. if a unified "all known stations" list is ever introduced), not urgent now — removing the field has ripple effects (wire format, TS interface) not worth it for this alone.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-transport-controls-play-pause-volume-mute.md`
  summary: A successful reconnect re-emits the same `play` event for the *same* station (per `run_playback`'s retry loop), which redundantly re-triggers `setLastStation`/a full `save_settings` disk write even though nothing actually changed.
  evidence: Code review confirmed `RadioPlayer::run_playback` emits `play` on every successful connect, including reconnects after a drop, and `playback.ts`'s `play` listener unconditionally calls `setLastStation` on every such event. Correct but wasteful; could dedupe by comparing against the already-stored `lastStation.id` before writing.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-transport-controls-play-pause-volume-mute.md`
  summary: No coalescing/sequencing on the new `setLastStation`-triggered `save_settings` writes — rapidly switching stations fires overlapping, unawaited persist calls with no ordering guarantee, so the persisted `lastStation` could in principle end up reflecting an earlier click rather than the most recent one if writes complete out of order.
  evidence: Code review found each `play` event independently awaits its own `saveSettings()` call with no queue/sequencing token. Same class of gap already deferred for Story 1.4's `toggleFavorite`/`moveUp`/`moveDown`; low real-world impact for a single user's local IPC calls, which are fast enough that true reordering is unlikely, but worth a guard if this pattern is extended to something latency-sensitive.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-transport-controls-play-pause-volume-mute.md`
  summary: `Store::save()` (`winradio/src-tauri/src/store.rs`) writes `store.json` in place via plain `std::fs::write`, with no temp-file+rename and no fsync — a crash mid-write can corrupt the file. This story increases write frequency substantially (a full settings write now happens on every station play, not just on explicit settings/volume changes), raising the practical odds of hitting this pre-existing risk.
  evidence: Code review confirmed `Store::save()`'s implementation is unchanged since Story 1.1 but is now invoked far more often via the new `setLastStation` path. Worth an atomic write (write to a temp file, then rename) given the increased exposure.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-transport-controls-play-pause-volume-mute.md`
  summary: The persisted `lastStation` snapshot can silently diverge from a station's live record in `stationsStore` (if it's later favorited, renamed, reordered, or unfavorited after being played) — `restoreLastStation` never reconciles by id against the current Favorites list, it just displays the frozen snapshot verbatim.
  evidence: Code review traced `restoreLastStation` (`playback.ts`) and confirmed no reconciliation against `stationsStore.stations`. Currently harmless since the Now-Playing idle display only reads `name`/`category` (not `isFavorite`/`favoriteOrder`/`addedAt`), but worth revisiting if the display ever surfaces those fields.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-5-transport-controls-play-pause-volume-mute.md`
  summary: No frontend component/integration test harness exists to cover `App.vue`'s actual `onMounted` startup wiring (that `restoreLastStation` runs only after `loadSettings` resolves, with the right argument) — a regression in that ordering could ship with a fully green test suite, since the existing store-level tests only exercise `restoreLastStation`/`loadSettings` in isolation, never the real startup sequence together.
  evidence: Code review (verification-gap layer) confirmed no `App.test.ts`/component test exists and traced that neither `playback.test.ts` nor `settings.test.ts` touches `App.vue`. Same class of gap already logged for Story 1.4 (no component-level test tooling); a genuine but larger infrastructure investment beyond a single story.
