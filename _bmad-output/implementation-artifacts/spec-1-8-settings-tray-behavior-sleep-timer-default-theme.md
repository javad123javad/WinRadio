---
title: 'Settings: Tray Behavior, Sleep Timer Default, Theme'
type: 'chore'
created: '2026-09-04'
status: 'done'
route: 'one-shot'
---

# Settings: Tray Behavior, Sleep Timer Default, Theme

## Intent

**Problem:** Story 1.8's three acceptance criteria (immediate-apply with no Save button; live theme switching that persists and pre-fills Story 1.7's Sleep Timer picker; correct keyboard Tab order/Esc/focus rings) needed to be confirmed as actually satisfied, not assumed.

**Approach:** No new code was written. `SettingsModal.vue`, the settings Pinia store, the backend `Settings` struct/persistence, and the theme-token CSS were all already built during Story 1.1's foundation rescue as scaffolding required by Stories 1.5/1.6/1.7 (volume, `minimizeToTray`, `sleepTimerDefaultMinutes`, `theme` all needed to exist and round-trip long before this story). Verification traced the full chain for each AC and additionally exercised it live against a Vite dev server (accessibility-tree inspection, not screenshots): immediate-apply confirmed via an observed `save_settings` invoke firing on the very next tick after a toggle; live theme switching confirmed via `document.documentElement`'s `light` class flipping and the surface tokens changing on screen; Tab order confirmed via the accessibility tree showing nav → favorites rows → transport bar → Settings modal controls in that exact order; Esc-closes confirmed via a real keydown event closing the modal. All three ACs hold with no gaps. Findings surfaced during review that are real but don't violate these specific ACs (missing ARIA dialog semantics, no focus trap/restore, silent save/load failures, color-only default-preset highlighting, and others) were routed to `deferred-work.md` rather than fixed here, since fixing them isn't what this story asked for.

## Suggested Review Order

**Immediate-apply, no Save button (AC1)**

- `onChange` fires `saveSettings` on every toggle/select change — no batching, no explicit Save control anywhere in the modal.
  [`SettingsModal.vue:84`](../../winradio/src/components/SettingsModal.vue#L84)

- `saveSettings` always reconstructs and sends the full settings snapshot, confirmed live: a change dispatched an `invoke('save_settings', ...)` call on the next tick.
  [`settings.ts:45`](../../winradio/src/stores/settings.ts#L45)

**Live theme switch, persistence, and Sleep Timer default pre-fill (AC2)**

- `applyTheme` toggles the `.light` class reactively on every `settingsStore.theme` change — confirmed live via `document.documentElement.className`.
  [`App.vue:231`](../../winradio/src/App.vue#L231)

- The CSS token swap DESIGN.md specifies: `.light` inverts only the surface ramp, keeping both accents identical across themes.
  [`main.css:26`](../../winradio/src/styles/main.css#L26)

- Rust `Settings::theme` and its default, persisted verbatim through `store.rs`'s save/load round trip (tested).
  [`commands.rs:28`](../../winradio/src-tauri/src/commands.rs#L28)

- The Sleep Timer popover highlights the preset matching `sleepTimerDefaultMinutes` — this story's "pre-fill," per Story 1.7's own code map.
  [`TransportBar.vue:88`](../../winradio/src/components/TransportBar.vue#L88)

**Keyboard: Tab order, Esc, focus rings (AC3)**

- Esc closes the modal via a window-level listener attached only while open; confirmed live with a real keydown event.
  [`SettingsModal.vue:93`](../../winradio/src/components/SettingsModal.vue#L93)

- Global `:focus-visible` outline applies to every button/input/select/anchor in the app, not just Settings' controls.
  [`main.css:55`](../../winradio/src/styles/main.css#L55)

**Peripherals**

- Existing store-level test coverage for load/save/`lastStation` round-tripping, exercised as part of this verification.
  [`settings.test.ts:1`](../../winradio/src/stores/settings.test.ts#L1)

- Backend persistence and malformed-data resilience tests for `Settings` (including `theme`), exercised as part of this verification.
  [`store.rs:141`](../../winradio/src-tauri/src/store.rs#L141)
