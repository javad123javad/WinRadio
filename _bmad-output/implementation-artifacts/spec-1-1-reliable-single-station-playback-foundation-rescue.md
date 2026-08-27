---
title: 'Reliable Single-Station Playback (Foundation Rescue)'
type: 'refactor'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: '503be7839590da5f3b0ec45b15083f768f35a0b2'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The existing scaffold has a broken foundation: a duplicate crate root (`lib.rs` + `main.rs` compile every module twice), a hand-rolled ICY parser that blocks synchronously inside `Iterator::next()`, a serde casing bug (backend is snake_case, frontend expects camelCase, no rename layer), a tray that's built but never attached to the app, a dangling `<router-view/>` with no router, and unwanted EQ/recording subsystems (plus their `lame-sys`/`hound`/`cpal` deps). None of Epic 1's later stories can be built safely on top of this.

**Approach:** Delete `lib.rs` (make `main.rs` sole crate root), replace the hand-rolled streaming/ICY code with `stream-download` (bump 0.5→0.24) + `icy-metadata` (0.6, `default-features = false`) feeding `rodio`, delete EQ + recording subsystems and their commands/deps, add `#[serde(rename_all = "camelCase")]` to every boundary struct, fix tray registration, and rewrite the Vue frontend from scratch (new dark-theme shell, no router, Pinia stores as pure event-driven read-models) per DESIGN.md tokens.

## Boundaries & Constraints

**Always:** `main.rs` is the sole crate root. Every Tauri command keeps `Result<T, String>`. All boundary-crossing structs (`Station`, `Settings`, `Metadata`, event payloads) use `#[serde(rename_all = "camelCase")]`. At most one active stream at a time. Rust pushes state via `emit_all` events (`play`, `stop`, `reconnecting`, `playback-error`, `metadata-updated`) — frontend stores only mutate from `listen()`, never optimistically after `invoke()`. Failed stream: immediate retry, then 2s/5s/10s backoff, 3 attempts (~20s budget), then `playback-error`. Info Tile grid area is reserved in layout but left empty (Epic 2). Below min window width the (empty) tile grid would go 3→2 across; the Station List rail never collapses — enforce a minimum window width instead.

Out of scope for this story (correctly deferred to Story 1.5): persisting the last-played station/volume across restarts and restoring it idle on relaunch. Not one of this story's four ACs.

**Ask First:** Whether `Settings.start_minimized`/`show_notifications`/`compact_player` (not in the target field list: `minimizeToTray`/`sleepTimerDefaultMinutes`/`theme`/`volume`) should be dropped now or deferred — HALT and ask before deleting if any UI still references them.

**Never:** No `vue-router` install (already absent — just delete the dangling tag). No polling/setInterval for metadata or playback state. No EQ or recording code, commands, or UI. No direct `<img>`/host hotlinking (N/A this story — no Info Tiles yet). Don't reuse `PlayerControls.vue`/`SettingsModal.vue` markup as components — rewrite against the new store contract.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Play station | Click station row / play | `play` Tauri command → `stream-download`+`icy-metadata`+`rodio` starts audio; `play` event emitted | On failure, retry policy below |
| Stream drops mid-playback | Network interruption | Immediate retry, then 2s/5s/10s backoff (3 attempts, ~20s) | `reconnecting` event each attempt; `playback-error` if all fail |
| Window resized below min width | Any state | Tile grid area 3→2 across; rail unaffected | Min width enforced, not collapse-to-hidden |
| Settings struct round-trip | Any `Station`/`Settings` payload | Wire fields are camelCase, no manual mapping | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/lib.rs` -- DELETE (duplicate crate root; `main.rs` never consumes it)
- `winradio/src-tauri/src/main.rs:6-10,25-73,75-143` -- module decls, `.setup()` (tray built but unattached l.37), duplicated tray-event handler (l.75-108, `play_pause` no-op l.91-93), dead `sleep-timeout` listener (l.66-71), command registration (l.124-143)
- `winradio/src-tauri/src/audio/player.rs` -- `HttpStreamSource` hand-rolled ICY parser + `block_on` inside `Iterator::next()` (l.271-326, blocking call l.282-285); `EqSource`/`BiquadFilter` (l.367-479, DELETE); `Recorder` + start/stop_recording (l.21-25,179-209, DELETE); rewrite around `stream-download`+`icy-metadata`+`rodio`
- `winradio/src-tauri/src/commands.rs:10-40` -- `Station`/`Settings`/`AudioDevice`/`Metadata` structs, add `#[serde(rename_all = "camelCase")]`; l.62-98 EQ/recording commands (DELETE); l.116-134 `list_audio_devices`/cpal (DELETE)
- `winradio/src-tauri/src/tray.rs` -- correct tray impl, currently unused; wire into `main.rs` builder, remove `main.rs`'s inline duplicate
- `winradio/src-tauri/src/timer.rs:20-46` -- `SleepTimer::set`; needs armed-state event emission + explicit `cancel()` for Story 1.7 (this story: keep API stable, don't block on it)
- `winradio/src-tauri/src/store.rs:5,19-44,69-76` -- `Settings` fields to prune (`output_device`,`buffer_size`,`eq_enabled`,`recording_format`,`recording_bitrate`); add `sleepTimerDefaultMinutes`,`volume`; string-keyed `Store::get` inconsistency
- `winradio/src-tauri/Cargo.toml:17,19,21,22` -- bump `stream-download` 0.5→0.24, add `icy-metadata` 0.6 (`default-features = false`), remove `cpal`/`lame-sys`/`hound`
- `winradio/src/App.vue:26` -- delete `<router-view/>`; rebuild layout per DESIGN.md (nav rail, station list rail, transport bar, dashboard shell with empty tile grid area)
- `winradio/src/main.ts` -- unchanged (Pinia only, no router to remove)
- `winradio/src/stores/playback.ts`, `winradio/src/stores/stations.ts` -- rewrite as event-driven read-models (`listen()` on `play`/`stop`/`reconnecting`/`playback-error`/`metadata-updated`), drop EQ/recording/optimistic-mutation state
- `winradio/src/components/PlayerControls.vue`, `winradio/src/components/SettingsModal.vue` -- discard, rebuild against new stores + DESIGN.md tokens (Settings modal: only `minimizeToTray`/`sleepTimerDefaultMinutes`/`theme` this story, no volume control here — volume lives in transport bar)
- `winradio/tailwind.config.js`, `winradio/src/styles/main.css` -- populate `theme.extend` from DESIGN.md tokens (currently empty)

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/lib.rs` -- delete file -- eliminates duplicate crate root
- [x] `src-tauri/src/main.rs` -- attach `tray.rs`'s tray to builder, remove inline duplicate + dead `sleep-timeout` listener -- fixes untriggered tray bug
- [x] `src-tauri/Cargo.toml` -- bump `stream-download` to 0.24, add `icy-metadata` 0.6 (no default features), drop `cpal`/`lame-sys`/`hound` -- new audio stack
- [x] `src-tauri/src/audio/player.rs` -- rewrite `HttpStreamSource`/playback around `stream-download`+`icy-metadata`, delete `EqSource`/`BiquadFilter`/`Recorder` -- removes blocking-iterator anti-pattern and out-of-scope subsystems
- [x] `src-tauri/src/commands.rs` -- add `#[serde(rename_all = "camelCase")]` to `Station`/`Settings`/`Metadata`; delete EQ/recording/`list_audio_devices` commands; add `emit_all` calls for `play`/`stop`/`reconnecting`/`playback-error`/`metadata-updated` -- fixes casing bug, establishes event model
- [x] `src-tauri/src/store.rs` -- prune device/EQ/recording `Settings` fields, add `sleepTimerDefaultMinutes`/`volume` -- matches target data shape
- [x] `src/App.vue` -- delete `<router-view/>`, build new shell layout (nav, rail, transport, empty tile-grid area) with min-width enforcement -- AD-10 + UX-DR10
- [x] `src/stores/playback.ts`, `src/stores/stations.ts` -- rewrite as `listen()`-driven read-models, remove EQ/recording/polling-shaped state -- AD-4 event model
- [x] `src/components/` -- rebuild transport bar + settings modal against new stores and DESIGN.md tokens -- UX-DR1/DR5/DR7
- [x] `tailwind.config.js` -- populate `theme.extend` from DESIGN.md token set (dark default + light parallel) -- UX-DR1
- [x] Unit/integration test -- reconnect backoff sequence (immediate, 2s, 5s, 10s, then `playback-error`) -- covers I/O matrix retry scenario

**Acceptance Criteria:**
- Given the rewritten Vue shell and cleaned Rust backend, when Javad clicks play on a station, then audio plays via `stream-download`+`icy-metadata` (no hand-rolled blocking-in-iterator code)
- Given the Rust crate, when it's built, then `lib.rs` no longer exists, there's a single crate root, and no EQ/recording code, commands, or dependencies remain
- Given a `Station`/`Settings` struct crossing the Tauri boundary, when it serializes, then fields are camelCase with no manual mapping
- Given the base shell layout, when the window is resized below the minimum width, then the (reserved, empty) Info Tile grid area would collapse 3-across to 2-across and the Station List rail never collapses

## Spec Change Log

- **2026-08-27, Tasks & Acceptance Verification audit (step-03):** Found the frozen "Always" invariants and I/O matrix included "on relaunch, last-played station shows idle with last volume restored" — an epic-wide requirement copied in from `epic-1-context.md` during planning, but not one of Story 1.1's own four ACs in epics.md (that behavior is Story 1.5's AC). The implementer had correctly left it unbuilt, flagging the gap. Human confirmed (asked via checkpoint): remove it from Story 1.1's scope rather than implement it here. Amended: dropped the relaunch/idle-restore sentence from the "Always" bullet and removed its I/O matrix row. KEEP: everything else in Boundaries & Constraints and the remaining four matrix rows are unchanged and were satisfied as implemented.

- **2026-08-27, step-04 review (patch findings):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. 8 confirmed `patch` findings sent back to the implementer and fixed: (1) a stale/superseded-connection race in `try_connect_and_play` could install a stale sink into the shared slot and never stop it, violating the "at most one active stream" invariant — fixed with a generation-checked `install_if_current` guard + 3 new tests; (2) no read/idle timeout on the HTTP stream, only `connect_timeout` — fixed with `.read_timeout(30s)` (deliberately not `.timeout()`, which would kill healthy long-running streams); (3) unbounded rapid-reconnect loop on a flapping connection never reached `playback-error` — fixed with a consecutive-short-episode cap; (4) a `Settings` deserialization failure could wipe the user's saved stations too — fixed with `#[serde(default)]` + independent `stations`/`settings` decoding + 4 new tests; (5) volume slider changes were never persisted to disk — fixed by persisting on the slider's `change` event; (6) stale metadata survived `stop` — fixed by clearing it in the `stop` listener; (7) `listenersReady` could get stuck `true` after a failed `listen()` call — fixed by only setting it after all listeners succeed; (8) unclamped `prefetch_bytes` could exceed the storage provider's capacity on a bogus bitrate header — fixed with a `.min()` clamp. Re-verified: `cargo build`/`cargo test` (11/11 passing, up from 4) and `npm run build` all pass after the patch round. KEEP: the original implementation's architecture (generation-counter based supersession, event-driven stores, `retry_with_backoff` helper) was sound — all 8 fixes are localized hardening, not redesigns. 6 additional non-blocking findings (retryable-vs-permanent error distinction, HTTP status-code handling, no-audio-device panic, missing logging/tracing, orphaned export/clear-data commands, and the Story 1.5 scope note above) were routed to `deferred-work.md` rather than fixed now.

## Verification

**Commands:**
- `cd winradio/src-tauri && cargo build` -- expected: compiles clean, no `lib.rs`, no EQ/recording/cpal/lame-sys/hound references
- `cd winradio/src-tauri && cargo test` -- expected: reconnect-backoff test passes
- `cd winradio && npm run build` -- expected: type-checks and builds with no router import errors

**Manual checks (if no CLI):**
- Launch `tauri dev`: tray icon appears and play/pause from tray works; window resize below min width collapses tile-grid area, not the rail; closing window minimizes to tray instead of quitting

## Suggested Review Order

**Audio pipeline rewrite & reliability (the story's core)**

- Entry point: supersession-safe `play()` bumps the generation and tears down the prior sink before spawning a fresh session.
  [`player.rs:155`](../../winradio/src-tauri/src/audio/player.rs#L155)

- `run_playback`'s loop now caps consecutive short-lived reconnect episodes so a flapping connection reaches `playback-error` instead of looping forever.
  [`player.rs:218`](../../winradio/src-tauri/src/audio/player.rs#L218)

- `try_connect_and_play` builds the `stream-download`+`icy-metadata`+`rodio` pipeline, replacing the old hand-rolled blocking ICY parser.
  [`player.rs:312`](../../winradio/src-tauri/src/audio/player.rs#L312)

- Read-idle timeout (not a blanket request deadline) detects a stalled-but-connected stream.
  [`player.rs:333`](../../winradio/src-tauri/src/audio/player.rs#L333)

- Prefetch size is clamped under the bounded storage's capacity against a bogus `icy-br` header.
  [`player.rs:353`](../../winradio/src-tauri/src/audio/player.rs#L353)

- `install_if_current` closes the race window between a supersession check and a slow-to-build sink actually publishing itself.
  [`player.rs:414`](../../winradio/src-tauri/src/audio/player.rs#L414)

- The generic guard itself: only installs into the shared slot if the generation still matches at that exact moment.
  [`player.rs:452`](../../winradio/src-tauri/src/audio/player.rs#L452)

- Generic backoff helper driving the immediate/2s/5s/10s retry schedule.
  [`player.rs:469`](../../winradio/src-tauri/src/audio/player.rs#L469)

**Crate root & command surface cleanup**

- `main.rs` now attaches the real tray implementation instead of an inline, subtly-different duplicate.
  [`main.rs:41`](../../winradio/src-tauri/src/main.rs#L41)

- Command registration: EQ/recording commands are gone; the surviving surface is what Story 1.1 actually needs.
  [`main.rs:56`](../../winradio/src-tauri/src/main.rs#L56)

- `Station`/`Settings` now derive `#[serde(rename_all = "camelCase")]`, fixing the wire-format bug; `Settings` is pruned to the four target fields.
  [`commands.rs:9`](../../winradio/src-tauri/src/commands.rs#L9)

- `src/lib.rs` deleted entirely — `main.rs` is now the sole crate root (no line reference; the file is gone).

**Persistence & schema robustness**

- `stations` and `settings` decode independently so a malformed/future-shaped one can never wipe the other.
  [`store.rs:46`](../../winradio/src-tauri/src/store.rs#L46)

**Frontend event-driven read-model**

- `initListeners` only flips ready once every backend event is actually subscribed, and resets on partial failure instead of wedging silently.
  [`playback.ts:44`](../../winradio/src/stores/playback.ts#L44)

- `stop` now clears stale metadata so the now-playing panel doesn't show a dead track title.
  [`playback.ts:56`](../../winradio/src/stores/playback.ts#L56)

- `persistVolume` closes the gap where slider-only volume changes never reached disk.
  [`playback.ts:133`](../../winradio/src/stores/playback.ts#L133)

- Settings store: simple load-once/save-on-change config, no push event of its own.
  [`settings.ts:18`](../../winradio/src/stores/settings.ts#L18)

**UI shell rewrite**

- `App.vue` replaces the dangling `<router-view/>` with the real nav/rail/dashboard shell and wires startup loading, theme, and the `Space`-toggles-play accessibility shortcut.
  [`App.vue:167`](../../winradio/src/App.vue#L167)

- Transport bar: play/pause, mute-restores-exact-volume, and volume persisted on slider release rather than every drag tick.
  [`TransportBar.vue:69`](../../winradio/src/components/TransportBar.vue#L69)

- Settings modal trimmed to exactly this story's three fields (minimizeToTray/sleepTimerDefaultMinutes/theme) with no Apply/Save button.
  [`SettingsModal.vue:84`](../../winradio/src/components/SettingsModal.vue#L84)

- DESIGN.md tokens as Tailwind theme extensions, including the custom `tiles: 900px` breakpoint for the 3→2 Info Tile grid collapse.
  [`tailwind.config.js:35`](../../winradio/tailwind.config.js#L35)

**Peripherals**

- New regression tests: `install_if_current` supersession guard and the retry/backoff schedule.
  [`player.rs:410`](../../winradio/src-tauri/src/audio/player.rs#L410)

- New regression tests: independent `stations`/`settings` deserialization fallback.
  [`store.rs:99`](../../winradio/src-tauri/src/store.rs#L99)

- Dependency bump: `stream-download` 0.5→0.24, `icy-metadata` added, `cpal`/`lame-sys`/`hound` removed.
  [`Cargo.toml:17`](../../winradio/src-tauri/Cargo.toml#L17)
