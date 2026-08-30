---
title: 'Preview & Play From Search Results'
type: 'chore'
created: '2026-08-29'
status: 'done'
route: 'one-shot'
---

# Preview & Play From Search Results

## Intent

**Problem:** Story 1.3's acceptance criterion — clicking a search result row plays it immediately, replacing whatever was previously playing, with no separate select-then-play step — needed to be confirmed as actually satisfied, not assumed.

**Approach:** No new code was written. Story 1.2's spec deliberately scoped search results to reuse the exact `StationRow` component Story 1.1 built for Favorites, specifically so this story's click-to-play behavior would fall out for free rather than being built twice. Verification traced the full chain — `StationRow.vue`'s unconditional `@click` emit, `searchStore.playResult`'s adapter into `playbackStore.play()`, and `RadioPlayer::play()`'s generation-bump-and-teardown in Rust — and confirmed it satisfies the AC with no gaps. A live `tauri dev` run also directly observed a search result (a real Radio-Browser station) playing on click. Findings surfaced during verification that are real but don't violate this specific AC (UI-feedback latency, re-click-restarts, missing Search/Favorites identity linking, and others) were routed to `deferred-work.md` rather than fixed here, since fixing them isn't what this story asked for.

## Suggested Review Order

- `StationRow.vue`'s click handler emits `play` unconditionally — no selection state, no confirm step.
  [`StationRow.vue:7`](../../winradio/src/components/StationRow.vue#L7)

- `search.ts`'s `playResult` adapts a search-result station and calls the same `playbackStore.play()` Favorites uses — no search-specific play path exists.
  [`search.ts:187`](../../winradio/src/stores/search.ts#L187)

- `App.vue` wires the search results list's `@play` to `searchStore.playResult` — identical wiring pattern to the Favorites list just above it.
  [`App.vue:116`](../../winradio/src/App.vue#L116)

- `RadioPlayer::play()` is what actually guarantees "replacing whatever was previously playing": it bumps the generation and tears down the prior sink synchronously, before anything else runs.
  [`player.rs:155`](../../winradio/src-tauri/src/audio/player.rs#L155)
