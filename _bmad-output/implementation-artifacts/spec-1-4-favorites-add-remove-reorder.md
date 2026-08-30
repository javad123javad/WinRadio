---
title: 'Favorites: Add, Remove, Reorder'
type: 'feature'
created: '2026-08-29'
status: 'done'
review_loop_iteration: 0
context: []
baseline_commit: 'd86690fb3f05988a458f30bef10db8ff3a2d3ea7'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `StationRow` has no favorite-star toggle and no reorder controls — `stations.ts`'s own comment flags this as unbuilt ("Favorite toggling, reordering ... are Stories 1.2-1.4"). Every seeded station is hardcoded `isFavorite: true` with no way to add, remove, or reorder anything.

**Approach:** Add a `favoriteOrder` field to `Station` (Rust + TS), an always-visible star toggle (separate click target from row-click-to-play, working from both Favorites and Search rows), and always-visible up/down reorder controls (Favorites rows only). Reuse the existing `save_stations` full-collection-replacement command for every mutation (add/remove/reorder) — no new backend command, per the architecture's "mutating commands take the full resulting collection" convention.

## Boundaries & Constraints

**Always:** Star toggle is a separate click target from the row's play click (`@click.stop`), visible on every row (Favorites and Search), not hover-reveal. Reorder controls (▲▼) are always visible, Favorites rows only, never on Search rows. Every favorite/reorder mutation persists via the existing `save_stations` full-collection command — no new Tauri command. Favorites order is driven by an explicit `favoriteOrder` field, not array position, so a partial/reconstructed list can't silently scramble order. The existing "No favorites yet" empty-state message and cold-load-without-spinner behavior (already implemented) must keep working.

**Ask First:** None identified — the star/reorder interaction pattern, click-target separation, and persistence mechanism are all fixed by DESIGN.md/EXPERIENCE.md and the architecture's stated convention.

**Never:** No drag-to-reorder — up/down buttons only (EXPERIENCE.md explicitly rejects drag gestures here). No new Tauri command for favorite/reorder mutations. Don't add a confirm dialog for removing a favorite (matches the app's no-confirm, immediate-effect pattern everywhere else). Don't touch Search's own result list when a star is toggled from a Search row — starring only affects the separate Favorites collection.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Star an unfavorited station (from Search or an already-favorited Favorites row shown elsewhere) | Click star icon | Added to Favorites (or removed if already favorited), persists across restart | N/A |
| Reorder | Click ▲/▼ on a Favorites row | Row moves one position, persists across restart | Boundary row's inactive direction is disabled, not a no-op click |
| Fresh install, no favorites | App opens | Rail shows "No favorites yet — search to find a station" (already implemented) | N/A |
| Favorite a search result already in Favorites, or unfavorite one not present | Star clicked on a row whose `isFavorite` state is stale/racing | Toggle reflects `stationsStore`'s current membership check, not stale local state | N/A |

</frozen-after-approval>

## Code Map

- `winradio/src-tauri/src/commands.rs:9-20` -- `Station` struct: add `pub favorite_order: i64` with `#[serde(default)]` (back-compat with existing `store.json` files lacking the field)
- `winradio/src/stores/stations.ts` -- `Station` interface add `favoriteOrder: number`; `DEFAULT_STATIONS` (`:19-24`) add `favoriteOrder: 0..3`; new actions `isFavorite(id)`, `toggleFavorite(candidate)` (accepts either a `Station` or a search-result-shaped station to convert and append), `moveUp(id)`/`moveDown(id)` (swap `favoriteOrder` with the adjacent sorted neighbor); render/sort `stations.value` by `favoriteOrder` ascending
- `winradio/src/components/StationRow.vue:34-40` -- add props `isFavorite: boolean`, `showReorder: boolean`, `canMoveUp: boolean`, `canMoveDown: boolean`; add emits `toggle-favorite`, `move-up`, `move-down`; template: reorder ▲▼ at left edge (only if `showReorder`, per DESIGN.md `station-row`/`reorder-control-color` tokens), star at right edge (`★` filled `text-primary` when `isFavorite`, `☆` muted `text-on-surface-variant` otherwise) — both `@click.stop` so they never trigger the row's own play click
- `winradio/src/App.vue:90-98,109-117` -- Favorites usage: pass `is-favorite="true"`, `show-reorder="true"`, `can-move-up`/`can-move-down` from index, wire `@toggle-favorite`/`@move-up`/`@move-down` to `stationsStore`; Search usage: pass `is-favorite="stationsStore.isFavorite(station.id)"`, `show-reorder="false"`, wire `@toggle-favorite` to `stationsStore.toggleFavorite(station)` (converting the `DirectoryStation`)
- `winradio/src/stores/search.ts` -- reuse/export a station-shape converter (already has `toPlayableStation`) so `stationsStore.toggleFavorite` can accept a `DirectoryStation` directly without App.vue doing ad-hoc field mapping

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands.rs` -- add `favorite_order: i64` to `Station` with `#[serde(default)]` -- explicit persisted order, back-compat safe
- [x] `src/stores/stations.ts` -- add `favoriteOrder` to the TS interface and seed data; add `isFavorite`/`toggleFavorite`/`moveUp`/`moveDown`; sort by `favoriteOrder` before render -- core Favorites mutation logic
- [x] `src/components/StationRow.vue` -- add star toggle and conditional reorder controls, both separate click targets from row-click-to-play -- UX-DR3 / DESIGN.md station-row spec
- [x] `src/App.vue` -- wire the new props/emits at both `StationRow` usage sites (Favorites and Search) -- connects UI to store actions
- [x] Unit/integration test -- `moveUp`/`moveDown` swap adjacent `favoriteOrder` values correctly at both interior and boundary positions; `toggleFavorite` both adds and removes -- covers the reorder/star mutation logic

**Acceptance Criteria:**
- Given a station (favorite or search result), when Javad clicks its star, then it's added to/removed from Favorites and the change persists across restarts
- Given the Favorites list, when Javad uses the always-visible up/down controls on a row, then it moves one position and the new order persists
- Given a fresh install with no favorites, when the app opens, then the rail shows "No favorites yet — search to find a station" instead of an empty box, and on relaunch the rail loads Favorites immediately from local storage with no spinner
- Given the Favorites list, when Javad clicks a favorite row, then it plays immediately, replacing whatever was previously playing — same as any other station row

## Spec Change Log

- **2026-08-30, step-04 review (patch findings):** Three-layer review (blind-hunter, edge-case-hunter, verification-gap) against the diff since baseline, all findings independently re-verified against the actual code before triage. 5 confirmed `patch` findings sent back to the implementer and fixed: (1) reorder was a permanent no-op for any pre-existing favorite — `#[serde(default)]` backfills `favoriteOrder: 0` for every station in an old `store.json`, so swapping two tied `0`s changed nothing, forever — fixed with `normalizeFavoriteOrder` (detects a collision, backfills distinct sequential values from array order) called both on load and defensively inside `swapFavoriteOrder`, plus new tests using an all-tied list; (2) Enter on a nested reorder/favorite button also fired the row's `play`, since `@click.stop` doesn't stop `keydown` bubbling — fixed with `@keydown.stop` on each nested button; (3) Space no longer activated the row after the `<button>`→`<div role="button">` conversion — fixed with `@keydown.space.prevent`; (4) the favorite star lacked `aria-pressed` — added; (5) the new tests never asserted `saveStations`/`invoke` was actually called — added those assertions. Re-verified: `cargo build`/`cargo test` (20/20 passing) and `npm run build` all pass; the new `vitest` suite grew from 8 to 12 tests. KEEP: the original architecture (pure, independently-testable `sortByFavoriteOrder`/`swapFavoriteOrder` helpers, the `role="button"` row conversion to legally host nested buttons, reusing `save_stations` with no new command) was sound — all 5 fixes are localized hardening, not redesigns. 6 additional non-blocking findings (no rollback on save failure, no guard against overlapping saves, nested-button tab-stop cost, `favoriteOrder` never compacted, no component-level test tooling, `isFavorite` now provably always `true`) were routed to `deferred-work.md` rather than fixed now.

## Verification

**Commands:**
- `cd winradio/src-tauri && cargo build` -- expected: compiles clean with the new `favoriteOrder` field
- `cd winradio/src-tauri && cargo test` -- expected: all tests pass, no regressions
- `cd winradio && npm run build` -- expected: type-checks and builds with the new store logic/component props

**Manual checks (if no CLI):**
- Launch `tauri dev`: star a search result, switch to Favorites and confirm it appears; use ▲/▼ to reorder, restart the app, confirm the new order persisted; unfavorite the last remaining favorite and confirm the "No favorites yet" prompt returns

## Suggested Review Order

**Reorder logic & the migration-collision fix (the story's riskiest part)**

- `normalizeFavoriteOrder`: detects a `favoriteOrder` collision (every pre-existing favorite defaults to `0`) and backfills distinct sequential values.
  [`stations.ts:47`](../../winradio/src/stores/stations.ts#L47)

- `swapFavoriteOrder` calls the normalizer before swapping, so a migrated all-tied list has real values to exchange.
  [`stations.ts:58`](../../winradio/src/stores/stations.ts#L58)

- `loadStations` normalizes immediately on load, so the invariant holds as early as possible.
  [`stations.ts:92`](../../winradio/src/stores/stations.ts#L92)

- `toggleFavorite`: adds (converting a `FavoritableStation`, appended with the next order) or removes, based on current membership, not a caller's possibly-stale copy.
  [`stations.ts:129`](../../winradio/src/stores/stations.ts#L129)

**Keyboard-activation fixes on the shared row**

- The row wrapper's `<button>`→`<div role="button">` conversion (required to legally nest real buttons) and its Enter/Space handlers.
  [`StationRow.vue:12`](../../winradio/src/components/StationRow.vue#L12)

- Each nested button's `@keydown.stop` — the fix that stops Enter/Space on the star or a reorder arrow from also firing the row's own `play`.
  [`StationRow.vue:29`](../../winradio/src/components/StationRow.vue#L29)

- The favorite star: separate click target, filled/outline state, `aria-pressed` toggle semantics.
  [`StationRow.vue:58`](../../winradio/src/components/StationRow.vue#L58)

**Wiring & persisted shape**

- `App.vue`'s Favorites list: `sortedStations` iteration, index-derived `canMoveUp`/`canMoveDown`, and the new emits wired to the store.
  [`App.vue:90`](../../winradio/src/App.vue#L90)

- `App.vue`'s Search list: `isFavorite` cross-check against the Favorites store, and favoriting a search result via the shared `toPlayableStation` converter.
  [`App.vue:116`](../../winradio/src/App.vue#L116)

- `favorite_order` on the persisted `Station` struct, `#[serde(default)]` for back-compat.
  [`commands.rs:20`](../../winradio/src-tauri/src/commands.rs#L20)

**Peripherals**

- The first-ever frontend test suite: pure-helper tests (including the all-tied migration case) and store-level tests asserting both outcome and persistence-call.
  [`stations.test.ts:1`](../../winradio/src/stores/stations.test.ts#L1)

- Rust back-compat test for a `store.json` missing the new field entirely.
  [`store.rs:1`](../../winradio/src-tauri/src/store.rs#L1) (test module at file end)
