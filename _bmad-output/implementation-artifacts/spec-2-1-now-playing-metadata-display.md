---
title: 'Now-Playing Metadata Display'
type: 'bugfix'
created: '2026-09-04'
status: 'done'
route: 'one-shot'
---

# Now-Playing Metadata Display

## Intent

**Problem:** Story 2.1's two acceptance criteria — the live track/title shows as subtext when the stream provides ICY metadata, and the subtext is "simply blank — not an error state" when it doesn't — needed to be confirmed as satisfied. Station name display and ICY metadata subtext were already built during Story 1.1's foundation rescue, but the "no metadata" case rendered `station.category || 'Radio'` instead of being blank, directly contradicting the frozen AC.

**Approach:** Removed the `category`/`'Radio'` fallback branch in `App.vue`, leaving the subtext genuinely empty when `metadata.title` is unset. Verified both states live against a Vite dev server by injecting Pinia state directly (`window.__pinia`, temporarily exposed via a `main.ts` edit and reverted immediately after) — a station with an empty title renders no subtext at all, and a station with a title/artist renders "title — artist" correctly. Code review of the surrounding metadata lifecycle surfaced a related, in-scope gap: metadata was never cleared on `playback-error`, so a stale track title could linger after a genuine playback failure, misrepresenting what's actually playing. Fixed alongside the main change, with new store-level tests covering `metadata-updated`, and metadata resetting on `play`/`stop`/`playback-error`. Findings that are real but pre-existing or out of this story's scope (dead `get_metadata` command, orphaned `Station.category` field, an edge case where a title-less-but-artist-present ICY tag gets fully dropped, unused `album`/`artworkUrl` fields) were routed to `deferred-work.md`.

## Suggested Review Order

**The AC2 fix: blank subtext, not a fallback label**

- The fallback branch removed — subtext now renders nothing when there's no ICY title, instead of `category || 'Radio'`.
  [`App.vue:153`](../../winradio/src/App.vue#L153)

**Metadata lifecycle correctness (surfaced by review, fixed alongside)**

- `playback-error` now resets `metadata` to empty — a stale title no longer lingers after a genuine playback failure.
  [`playback.ts:89`](../../winradio/src/stores/playback.ts#L89)

- The existing `play`/`stop` resets this mirrors, already correct before this story.
  [`playback.ts:60`](../../winradio/src/stores/playback.ts#L60)

**Peripherals**

- New tests: `metadata-updated` sets metadata; `play`/`stop`/`playback-error` all reset it to empty.
  [`playback.test.ts:110`](../../winradio/src/stores/playback.test.ts#L110)
