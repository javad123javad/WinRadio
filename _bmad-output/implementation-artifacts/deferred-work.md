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

- source_spec: `_bmad-output/implementation-artifacts/spec-1-6-system-tray-presence.md`
  summary: The tray icon (`winradio/src-tauri/src/tray.rs`) carries no tooltip or state indicator — once minimized (now the default outcome of closing the window, not opt-in), there's no way to tell from the tray alone what's playing or even that the icon belongs to WinRadio.
  evidence: Code review confirmed no tooltip-setting API call anywhere in `tray.rs`. Not required by any stated AC (which only asks for play/pause exposure), but a reasonable enhancement now that tray-residency is the default experience for every user.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-6-system-tray-presence.md`
  summary: Tray "Quit" calls `std::process::exit(0)` directly, with no graceful audio-stream teardown or final store flush. This is more consequential now that minimize-to-tray defaults on, since the tray's Quit item becomes the primary/only exit path for most users instead of the window's X button.
  evidence: Code review confirmed `tray.rs`'s `"quit"` handler is an unconditional `std::process::exit(0)`. Pre-existing since Story 1.1's tray wiring, not introduced by this story's default-value change, but worth a graceful shutdown path given increased exposure.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-6-system-tray-presence.md`
  summary: No single-instance guard exists — since the app now commonly runs hidden with no taskbar entry by default, a user who forgets it's running and relaunches it risks a second process contending over the same `store.json` and audio device.
  evidence: Code review confirmed neither `Cargo.toml` nor `main.rs` registers any single-instance protection (e.g. `tauri-plugin-single-instance`). Pre-existing gap, made more likely to matter now that tray-residency is the default rather than opt-in.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-6-system-tray-presence.md`
  summary: `tauri.conf.json`'s `systemTray.iconAsTemplate: true` is a macOS-specific setting (template-image recoloring for the macOS menu bar) left enabled in a Windows-only app (NSIS-only bundle target, `main.rs`'s Windows-gated code) — likely a harmless no-op on Windows, but worth confirming the tray icon renders correctly and removing the setting if it's confirmed to do nothing here.
  evidence: Code review flagged this config value as unrelated to any Windows tray behavior. Pre-existing since the app's original scaffold, unrelated to this story's default-flip change; now exercised by every user by default rather than only opt-ins.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-7-sleep-timer.md`
  summary: There's no way for the frontend to query current sleep-timer status on demand — state only moves via one-shot push events (`sleep-timer-armed`/`sleep-timer-cleared`). Anything that starts listening after those events already fired (a late `initListeners`, a future secondary window) has no way to resync to "armed, N minutes" and will show unarmed indefinitely.
  evidence: Code review confirmed `SleepTimer` has no status-query command. Matches the same push-only architecture already used consistently by every other feature (play/stop/reconnecting/metadata all work this way too, per Story 1.1's event model) — not a gap unique to this story.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-7-sleep-timer.md`
  summary: No component-level test exists for the new `TransportBar.vue` sleep-timer UI itself (popover open/close, preset clicks, cancel row, outside-click dismissal, Escape dismissal) — only the Pinia store is tested.
  evidence: Code review confirmed the only new test coverage is in `playback.test.ts` (store logic), none touching the component's DOM/event behavior. Same class of gap already logged for Stories 1.4/1.5 (no component-level test tooling); a genuine but larger infrastructure investment beyond a single story.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-7-sleep-timer.md`
  summary: `armSleepTimer`/`cancelSleepTimer` failures are only `console.error`'d with no user-visible signal — combined with the popover closing immediately on click (before the round-trip event confirms anything), a failed arm/cancel is indistinguishable from a successful one to the user.
  evidence: Code review confirmed both actions' catch blocks are console-only. Matches the same optimistic-persistence pattern already deferred for Story 1.3's `playback.play()`, Story 1.4's `toggleFavorite`, etc. — a consistent codebase-wide pattern, not unique to this story.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-7-sleep-timer.md`
  summary: No guard against rapid double-clicks on different presets — two overlapping `set_sleep_timer` invokes have no ordering guarantee (the critical section that swaps the timer handle is properly mutex-protected, but the event `emit()` calls happen outside that lock, so under Tauri's multi-threaded async runtime a stale "armed" event could in principle arrive after a newer one).
  evidence: Code review (edge-case-hunter + blind-hunter) both raised variants of this. Same class of gap already deferred for Stories 1.4/1.5 (no coalescing/sequencing on rapid actions); low real-world impact since the popover closes immediately on click, requiring the user to reopen and click again within milliseconds to trigger it.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-7-sleep-timer.md`
  summary: The Sleep Timer icon button has `aria-expanded` and a dynamic `aria-label` but no `aria-haspopup`; opening the popover doesn't move focus into it, and closing via Escape/Cancel doesn't return focus to the trigger button — the disclosure pattern is only partially accessible.
  evidence: Code review confirmed the gaps directly in `TransportBar.vue`. Not required by any stated AC or by EXPERIENCE.md's accessibility floor (which specifies Tab order, Esc-closes-Settings, focus rings, no color-only state — not focus management for every new popover), but worth a future accessibility pass.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-7-sleep-timer.md`
  summary: The re-arm-supersedes-old-timer behavior (arming a new duration while one is already active correctly cancels the old one) is documented only by code comments, with no test verifying the superseded task actually goes inert rather than later calling `player.stop()`/emitting a second `sleep-timer-cleared`.
  evidence: Code review identified this as the most subtle new logic in the diff. A genuine test would need `tokio::time::pause()`/`advance()` fake-time infrastructure not currently used anywhere in this codebase — valuable but non-trivial effort beyond this story's scope.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: `SettingsModal.vue`'s overlay has no `role="dialog"`, `aria-modal="true"`, or `aria-labelledby` pointing at its "Settings" heading, so assistive tech doesn't announce it as a modal dialog tied to that title.
  evidence: Code review confirmed the overlay `<div>` carries none of these attributes. Pre-existing since Story 1.1's scaffolding built this modal; not required by this story's frozen ACs (Tab order, Esc-closes, visible focus rings — no ARIA-dialog-semantics requirement stated), but a real screen-reader gap.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: The Settings modal has no focus trap, no initial-focus-on-open, and no focus-restoration-to-the-trigger-button on close — a keyboard user can Tab out of the open modal into the nav/rail/transport bar behind the overlay, and focus is simply abandoned wherever it was after Esc/Close.
  evidence: Code review confirmed no focus-management code exists in `SettingsModal.vue`. Deliberately not treated as a violation of this story's own AC3, whose "Tab order follows nav→rail→transport→settings" wording describes exactly one linear whole-app sequence ending at Settings — which the current untrapped behavior actually satisfies, as directly observed live (Vite dev server, accessibility-tree inspection: Settings' Close/checkbox/selects/Close consistently appear last in tab order). Still a real general modal-accessibility gap worth a future pass.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: The "Sleep Timer default" and "Theme" rows wrap their `<select>` in a plain `<div>`+`<span>` instead of a `<label>` (unlike the "Minimize to tray" row just above, which correctly uses `<label>`), so those two selects' accessible names aren't programmatically associated with their visible row text.
  evidence: Code review confirmed the markup inconsistency directly in `SettingsModal.vue` (lines ~29-54 vs. ~19-27). Pre-existing since Story 1.1; a small, isolated a11y fix for a future pass.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: `saveSettings`/`loadSettings` failures are only `console.error`'d with no user-visible signal — given this story's own AC1 framing ("closing the modal is the only confirmation"), a toggle whose persistence silently failed (disk full, permissions) looks identical to one that succeeded.
  evidence: Code review confirmed both catch blocks in `settings.ts` are console-only, and `SettingsModal.vue`'s `onChange` doesn't await or surface the result. Matches the same optimistic-persistence pattern already deferred for Stories 1.3/1.4/1.7 — a consistent codebase-wide pattern, not unique to this story.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: No debounce or change-coalescing on the Sleep Timer default / Theme `<select>` elements — each `change` event (which some browsers can fire per keyboard arrow-step through options) round-trips synchronously to `save_settings`, which does a blocking `std::fs::write` on every call.
  evidence: Code review traced `onChange` -> `saveSettings` -> `store::save_settings` -> `Store::save()`'s synchronous file write. Low real-world impact (settings changes are infrequent, deliberate user actions) but a real, unbounded-repetition path.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: No component-level test exists for `SettingsModal.vue` (immediate-apply on toggle, Esc-closes, Close-button) or for `TransportBar.vue`'s consumption of `sleepTimerDefaultMinutes` for preset pre-fill/highlight — this story's three ACs are verified only by live manual testing, not by an automated regression test.
  evidence: Code review confirmed `settings.test.ts` covers only the Pinia store, nothing DOM/event-level. Same class of gap already logged for Stories 1.4/1.5/1.7 (no component-level test tooling in this project) — a genuine but larger infrastructure investment beyond a single story; this entry specifically flags that Story 1.8's own ACs are among the now-uncovered behaviors.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: The Sleep Timer default preset highlighted in `TransportBar.vue`'s popover is conveyed by color alone (`text-primary` class, no icon/text/`aria-current` marker), which runs against EXPERIENCE.md's stated accessibility floor ("no state conveyed by color alone").
  evidence: Code review confirmed the highlight is a single conditional Tailwind color class with no other differentiator. Pre-existing since Story 1.7 built this popover; EXPERIENCE.md's color-alone rule is a cross-cutting requirement, not specific to Story 1.8, but this story is the first to formally exercise the "pre-fill" AC that surfaces it.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: `App.vue`'s `applyTheme()` runs twice on startup — once reactively via `watch(() => settingsStore.theme, applyTheme)` firing when `loadSettings()` mutates `theme.value`, and again explicitly at the end of `onMounted` — redundant, and easy to get subtly wrong if load ordering ever changes.
  evidence: Code review traced both call sites in `App.vue` (the `watch` declaration and the final `onMounted` line). Harmless today since `applyTheme()` is idempotent, but worth simplifying to a single call site.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: There's a window between app mount and `loadSettings()` resolving during which the Settings modal is already reachable with in-memory default values; a change made in that window is silently overwritten once the load response replaces the refs.
  evidence: Code review traced `settings.ts`'s `loadSettings()` unconditionally overwriting `minimizeToTray`/`sleepTimerDefaultMinutes`/`theme` refs on resolution, with no check for an intervening user edit. Narrow timing window (load is typically near-instant against local disk), but a real race.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: No validation or clamping of `sleepTimerDefaultMinutes` on either side — the Rust `Settings` struct accepts any `u32`, and the frontend doesn't guard against a hand-edited `store.json` value outside {15, 30, 60, 90}; it degrades gracefully (no preset highlighted in the popover) but silently, with no snap-to-nearest-valid-preset.
  evidence: Code review confirmed neither `commands.rs`'s `Settings` struct nor `settings.ts`/`TransportBar.vue` constrain this value. Pre-existing since Story 1.7; low real-world likelihood (value is only ever written by the app's own `<select>`, whose options are fixed).

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: Two independent window-level Escape-key listeners exist (`SettingsModal.vue`'s and `TransportBar.vue`'s Sleep Timer popover's), neither calling `stopPropagation` — if both the Settings modal and the Sleep Timer popover were ever open simultaneously, one Escape press would close both instead of just the topmost.
  evidence: Code review confirmed both listeners independently check `event.key === 'Escape'` with no coordination. In practice the two can't currently both be open (opening Settings requires a click outside the popover's trigger, which already closes the popover via its own outside-click handler), making this a latent rather than currently-reachable issue.

- source_spec: `_bmad-output/implementation-artifacts/spec-1-8-settings-tray-behavior-sleep-timer-default-theme.md`
  summary: The Theme `<select>`'s "System" option gives no indication of which concrete theme (light or dark) it's currently resolving to based on the OS preference.
  evidence: Code review confirmed `SettingsModal.vue` only ever shows the literal stored value ("System"/"Light"/"Dark"), never the resolved one. Minor UX polish gap, not required by any stated AC.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: No component-level test exists for `App.vue`'s now-playing panel itself (blank subtext when no title, populated subtext when title/artist present) — this story's own acceptance criteria are verified only by live manual testing (Pinia state injected directly in a running dev server), not by an automated regression test.
  evidence: Code review confirmed the only new coverage is store-level (`playback.test.ts`'s `metadata-updated`/`play`/`stop`/`playback-error` tests). Same class of gap already logged for Stories 1.4/1.5/1.7/1.8 (no component-level test tooling in this project) — this entry specifically flags that Story 2.1's own template-level ACs are among the now-uncovered behaviors.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: `RadioPlayer::stop()` (`player.rs`) does not clear `self.metadata`, unlike `play()` which does — an inconsistency in the same struct, currently harmless only because nothing calls the `get_metadata` command (see next entry).
  evidence: Code review confirmed `play()` resets metadata internally but `stop()` does not. No observable effect today since the frontend never queries `get_metadata`, but worth fixing for consistency if that command ever gets a real caller.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: The `get_metadata` Tauri command (and its `player.rs` implementation) is dead code — nothing in the frontend ever calls `invoke('get_metadata')`. There is no way to resync the now-playing subtext to the current ICY state if a listener starts late (e.g. after a page reload or a future secondary window) short of waiting for the next tag change.
  evidence: Code review confirmed the command exists and is registered but has zero call sites in `src/`. Pre-existing since an earlier story wired the command speculatively; worth either using it for a resync-on-load path or removing it.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: `Station.category` is now orphaned dead data — its only UI consumer was the fallback subtext this story removed (it violated AC2's "simply blank" requirement). `stations.ts` still declares, seeds, and copies the field; `search.ts`'s `toPlayableStation` still derives it from Directory tags; nothing renders it anywhere.
  evidence: Code review confirmed via a full grep of `src/` that no template or computed value reads `.category` after this story's fix. Worth an explicit decision (remove the field, or document it as reserved for a future tile) rather than leaving it silently computed and persisted for no consumer.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: `Metadata.album`/`Metadata.artworkUrl` are unused, always-empty fields on both sides — Rust's `handle_metadata` hardcodes them to empty strings, and the frontend `Metadata` interface carries both but nothing reads or displays them.
  evidence: Code review confirmed neither field has a real producer or consumer. Pre-existing scaffolding from Story 1.1; harmless, but a reader could easily mistake their presence for "album art is already supported."

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: A raw ICY title like `"Artist - "` (trailing separator, empty remainder) parses to a non-empty `artist` but an empty `title`; since the frontend gates the entire subtext on `v-if="playbackStore.metadata.title"`, a genuine non-empty artist tag would be silently dropped instead of shown — arguably a violation of "show it when the stream provides it," not just a coincidental blank state.
  evidence: Code review traced `handle_metadata`'s `split_once(" - ")` in `player.rs` against `App.vue`'s title-only gate. Pre-existing gating logic, not touched by this story's fix (which only removed the no-metadata fallback branch); a narrow real-world edge case worth a future look.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-1-now-playing-metadata-display.md`
  summary: The `reconnecting` event leaves the last-known metadata in place with no explicit test or comment confirming that's intentional (as opposed to the newly-added `playback-error` handling, which now clears it).
  evidence: Code review flagged the asymmetry. Arguably correct by design — `reconnecting` means the same stream is momentarily interrupted, not abandoned, so keeping the last known title makes sense until it either resumes (new `metadata-updated`) or gives up (`playback-error`, now cleared) — but this reasoning wasn't previously written down anywhere.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: `store.rs`'s `parse_store_data` decodes the entire `stations` array as one `Vec<Station>` in a single `serde_json::from_value` call — one malformed field on any single station (e.g. a hand-edited `geoLat` sent as a string) fails the whole array and falls back to `Vec::default()`, silently wiping *every* favorited station, not just the bad one.
  evidence: Code review confirmed `parse_store_data` (`store.rs`) has no per-entry resilience, unlike `directory.rs`'s `parse_stations` (Story 1.2), which already uses `filter_map` to skip individually-malformed search results. Pre-existing since Story 1.1's original store shape — adding `country`/`geoLat`/`geoLong` in this story increases the surface area of fields that could be malformed, but the underlying whole-array-decode fragility predates this story.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: `LocationTile.vue` has no fallback text when a station has valid coordinates but a `null` country (Radio-Browser sometimes omits it) — the template interpolates `location.country` directly, rendering a blank line above the map instead of e.g. "Unknown country".
  evidence: Code review confirmed `location_info_for`'s own doc comment states country is "carried through even if absent (never blocks on it)", i.e. this combination is expected to occur, but no test exercises `country: null` with coordinates present, and the template has no `v-else`/fallback text for it.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: No caching of fetched OSM tiles across a session — replaying a favorite or switching back to a previously-played station re-fetches the identical tile from `tile.openstreetmap.org` every time, even though nothing about that station's map image ever changes.
  evidence: Code review confirmed neither the Rust command nor the Pinia store cache tile bytes by coordinate/station. OSM's tile usage policy expects local caching rather than repeated identical requests; the narrower "same station reconnects after a stream drop" case was fixed directly in this review round (see Spec Change Log), but the general cross-session-replay case is a larger, separate caching design this story didn't scope.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: Rapid station switching fires one live HTTP request per switch to the OSM tile server with no cancellation of the now-superseded in-flight request — `locationRequestSeq` correctly discards the stale *result* so no incorrect state is ever shown, but the underlying network request still runs to completion regardless.
  evidence: Code review confirmed `get_location_tile`'s `reqwest` call has no `AbortController`/cancellation wiring. State correctness is unaffected (verified by the existing supersession test), so this is a network-politeness/resource-waste concern only, not a correctness bug.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: `get_location_tile`'s success path (a real 2xx response's bytes correctly base64-encoded) and its non-2xx-status branch (e.g. a reachable server returning 404) are both untested — the only new test for this command exercises the connection-refused path alone.
  evidence: Code review confirmed via `directory.rs`'s test module that no test spins up a real HTTP responder for this command. Closing this gap would need a local-mock-HTTP-server test technique not currently used anywhere in this codebase (existing tests only simulate "connection refused" via an unreachable port) — a real but non-trivial test-infrastructure investment.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: No component-level test covers `App.vue`'s Info Tile grid wiring — nothing asserts the grid still renders exactly three tiles after this story replaced the reserved "Location" div with `<LocationTile />`, even though DESIGN.md calls out "exactly three, never a fourth" (SM-C1) as a deliberate counter-metric against scope drift.
  evidence: Code review confirmed no test touches `App.vue`'s template. Same class of gap already logged for Stories 1.4/1.5/1.7/1.8/2.1 (no component-level test tooling in this project).

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: The "© OpenStreetMap contributors" attribution in `LocationTile.vue` is plain unlinked text rather than a hyperlink to `openstreetmap.org/copyright`, which is the customary (though not strictly mandated) way tile consumers satisfy OSM's attribution expectation.
  evidence: Code review confirmed the caption is a plain `<p>` with no `<a>`. Minor polish gap; no other external link exists anywhere else in the app to match a styling convention against.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-2-location-tile.md`
  summary: The fetched OSM raster tile has no dark-theme treatment (no CSS filter, no themed placeholder) — OSM's standard tile style is light/white-background, which will read as a jarring bright rectangle against WinRadio's deliberately dark, flat "Metro" surface design.
  evidence: Code review flagged the visual mismatch; confirmed neither DESIGN.md nor the architecture doc addresses tile theming anywhere, and no alternative (dark-styled) tile source is sanctioned by AD-12. An open design question, not a coding gap — needs a human aesthetic call (a CSS filter hack vs. accepting the mismatch vs. a different tile provider) rather than a guessed fix.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-3-weather-tile.md`
  summary: No component-level test for `WeatherTile.vue` verifying the placeholder actually replaces the numeric content when status is `idle`/`unavailable`.
  evidence: Review of the diff found only store-level (`playback.test.ts`) coverage; `LocationTile.vue` has the identical gap, already accepted in Story 2.2.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-3-weather-tile.md`
  summary: No test exercises `fetch_weather`'s real HTTP success path (URL/query-string construction against a live-shaped response) — only the pure `parse_weather` and the "unreachable" failure path are tested.
  evidence: Review of `winradio/src-tauri/src/weather.rs`'s test module found no mocked-server success test; `directory.rs`'s `fetch_stations`/`fetch_names` have the same established gap in this codebase.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-3-weather-tile.md`
  summary: No integration test exercises `should_emit_weather_update` + `directory::location_info_for`'s `None` branch (a station with no coordinates) through `run_playback` itself, nor the `tokio::spawn`'d fetch's `is_current_generation` guard suppressing a stale emission from a superseded station (I/O matrix's "Rapid station switch mid-fetch" row).
  evidence: Verification-gap review confirmed no test drives two successive `play()` calls while an earlier station's weather fetch is still in flight; building one requires a test seam for captured `emit()` calls (no fake `AppHandle` harness exists in this codebase) — a real but non-trivial testability investment. I independently read `player.rs:344-414` and confirmed the guard's logic is correct by inspection and matches the spec's required design; the gap is coverage, not a known defect.

- source_spec: `_bmad-output/implementation-artifacts/spec-2-3-weather-tile.md`
  summary: `Math.round()` on a temperature just below zero can display as "-0°C" in `WeatherTile.vue`.
  evidence: Cosmetic display edge case noted in review; `Math.round(-0.4)` returns `-0` in JS, which template-interpolates as the string "-0".

- source_spec: `_bmad-output/implementation-artifacts/spec-2-3-weather-tile.md`
  summary: No request-cancellation/backoff for the Open-Meteo call when a user switches stations rapidly and repeatedly — each switch spawns a new HTTP request (suppressed only by the dedup-on-same-station check, not by rate-limiting distinct switches).
  evidence: Mirrors the analogous "no request cancellation on rapid switching" item already deferred for Location in Story 2.2; same shape, now also true for Weather's backend-spawned fetch.
