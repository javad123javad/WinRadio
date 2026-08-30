---
title: 'System Tray Presence'
type: 'feature'
created: '2026-08-30'
status: 'done'
route: 'one-shot'
---

# System Tray Presence

## Intent

**Problem:** Story 1.6's mechanics — the tray menu (Show/Play-Pause/Quit), the close-to-tray logic gated by `minimizeToTray`, and the dashboard-restores-exactly-as-left behavior (inherent to `window.hide()`/`show()` never destroying the webview) — were all already built in Story 1.1. But `Settings::default().minimize_to_tray` was `false`, meaning a fresh install's window X button would *quit* the app, contradicting this story's own framing ("minimize to tray **instead of quitting**") and the Settings modal mockup, which depicts the toggle already "on" as its canonical state.

**Approach:** Flip the default to `true` on the Rust side, update the frontend's matching initial ref so the two can never briefly disagree, and add a test locking in the new default. A one-shot review (blind-hunter) confirmed the fix and found no other blocking gaps — everything else raised was either pre-existing (tray's abrupt `process::exit`, no single-instance guard, a stray macOS-only `iconAsTemplate` config left on) or contradicted this project's own established conventions (e.g., a first-run toast, when the app deliberately avoids toasts everywhere else).

## Suggested Review Order

- The actual behavior change: `Settings::default()` now defaults `minimize_to_tray` to `true`.
  [`commands.rs:42`](../../winradio/src-tauri/src/commands.rs#L42)

- Frontend's matching initial ref, so the UI can never briefly show a stale "off" state before `loadSettings()` resolves.
  [`settings.ts:25`](../../winradio/src/stores/settings.ts#L25)

- New test locking in the default so it can't silently regress.
  [`store.rs:193`](../../winradio/src-tauri/src/store.rs#L193)

- Verified unchanged (already correct from Story 1.1): the close-to-tray gate reading this setting.
  [`main.rs:44`](../../winradio/src-tauri/src/main.rs#L44)

- Verified unchanged: the tray menu itself (Show/Play-Pause/Quit).
  [`tray.rs:5`](../../winradio/src-tauri/src/tray.rs#L5)
