---
title: 'Consolidate Info Tile dedup/payload/spawn duplication in player.rs'
type: 'refactor'
created: '2026-09-18'
status: 'done'
route: 'one-shot'
---

# Consolidate Info Tile dedup/payload/spawn duplication in player.rs

## Intent

**Problem:** Epic 2 retro action items 1-2 flagged that `winradio/src-tauri/src/audio/player.rs` grew from 650 to 1015 lines across Stories 2.2-2.4 with zero refactoring: three byte-for-byte identical `should_emit_*_update` dedup functions (each with its own 3-test suite), three near-identical `{ok, data, reason}` payload structs, and two near-identical `tokio::spawn`+generation-guard+emit blocks (Weather/Stream Info).

**Approach:** Pure internal refactor, zero behavior change, existing tests as the safety net. Replaced the three dedup functions with one generic `should_emit_tile_update` (consolidating 9 duplicate tests down to 3). Replaced the three payload structs with one generic `TileUpdatePayload<T>`, plus a small `TileUpdatePayload::<T>::unavailable(reason)` constructor added during review to remove the remaining turbofish-heavy `{ok:false, data:None, reason:Some(...)}` repetition. Extracted Weather's and Stream Info's spawn/generation-guard/emit block shape into one shared `spawn_tile_fetch_and_emit<T, F>` method. Net: 1015 → 913 lines, 56 → 50 Rust tests (6 duplicate tests removed, no coverage lost), zero compiler warnings.

## Suggested Review Order

- Entry point — the generic envelope struct plus its `unavailable()` constructor, replacing three near-identical structs.
  [`player.rs:64`](../../winradio/src-tauri/src/audio/player.rs#L64)

- The shared spawn/generation-guard/emit helper, replacing Weather's and Stream Info's near-identical `tokio::spawn` blocks.
  [`player.rs:193`](../../winradio/src-tauri/src/audio/player.rs#L193)

- The single dedup function, replacing three byte-for-byte identical copies (Location/Weather/Stream Info).
  [`player.rs:691`](../../winradio/src-tauri/src/audio/player.rs#L691)
