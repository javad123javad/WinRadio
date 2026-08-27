# Adversarial Review — ARCHITECTURE-SPINE.md (WinRadio)

**Reviewer lens:** attack the spine as an adversary. For each finding below, two units (stories/features) are constructed that each satisfy every applicable AD/Rule/Convention *to the letter*, yet are structurally incompatible when integrated. Each pair is evidence of a hole: an AD or Rule that sounds like it pins the decision down but actually leaves a degree of freedom wide enough to build two non-interoperable things.

**Verdict: FAIL — send back for tightening.** The spine is strong on layering, ownership-of-state, and dependency choices, but it is silent or ambiguous on **wire-level contracts** (exact event names/payload shapes, exact command signatures) in exactly the places where two independently-built units are most likely to diverge: Info Tile failure payloads, the "now-playing metadata" event identity, Stream Info Tile event bundling, mutating-command signatures, and cross-AD failure-domain ownership. Five concrete divergence pairs are documented below, roughly in order of how likely they are to actually bite during implementation.

---

## Pair 1 — AD-5's own "or" produces two incompatible degraded-payload shapes

**Spine text (AD-5):** *"Each event type is emitted exactly once per attempt, on success or on failure — a failed fetch emits the same event with a degraded/empty payload **(or** an explicit `ok: false` + reason**)**, never silence."*

The Rule itself offers a binary choice and never says which branch applies to which tile, or whether all tiles must pick the same branch.

- **Unit A — `WeatherTile.vue` + `weather.rs`.** Implements the "degraded/empty payload" branch literally: on failure, `weather-updated` fires with `{ temperature: null, condition: null, description: "" }` — same struct shape as success, fields blanked. Fully satisfies AD-5 ("same event... degraded/empty payload"), AD-6 (camelCase), and NFR-3 (tile shows "Weather unavailable" by checking for `null`).
- **Unit B — `LocationTile.vue` + `directory.rs`.** Implements the other literal branch: on failure, `location-updated` fires with `{ ok: false, reason: "No coordinates for this station" }` — a wrapper struct that does not even contain the success-case fields (`lat`/`lon`/`tileUrl`). Equally satisfies AD-5 to the letter.

**The clash:** a shared frontend abstraction (`InfoTile.vue`, described in the Structural Seed as "shared shell for the three tile variants") cannot be written generically against both wire shapes — one tile's failure is "same struct, blank fields," the other is "different struct, wrapper discriminant." A dev writing `InfoTile.vue` to the spec of one tile breaks silently (TypeScript `undefined` access, or a type union that has to special-case every tile) when wired to the other. Nothing in AD-5 or the Consistency Conventions table forces a single discriminant shape across all three Info Tile events.

**Recommended fix:** Pin one shape for all Info Tile events, e.g. a single generic envelope `{ ok: boolean, data: T | null, reason: string | null }` used identically by `weather-updated`, `location-updated`, and `stream-info-updated`. Strike the "or" from AD-5's Rule.

---

## Pair 2 — "Now-playing metadata" (AD-5) vs `metadata-updated` (AD-4): same event or two?

**Spine text:**
- AD-4: *"Rust emits an event on every playback/metadata state change: `play`, `stop`, ..., and `metadata-updated`."*
- AD-5: *"Now-playing metadata, Location info, Weather info, and Stream info are each their own Tauri event."*
- Capability Map: Now-Playing Dashboard & Info Tiles is governed by **both** AD-4 and AD-5.

AD-4 already names a concrete event, `metadata-updated`, for "metadata state change." AD-5 separately lists "Now-playing metadata" as one of *four* Info-Tile-style events that must each be "its own Tauri event," implying (by parallel construction with Location/Weather/Stream) a fourth, tile-scoped event distinct from AD-4's playback-lifecycle one.

- **Unit A — built strictly off AD-4.** `audio/player.rs` fires `metadata-updated` with ICY track/title whenever it changes, and that is the *only* event `NowPlayingDashboard.vue` listens to for title/artwork/station-name. No second event exists — "Now-playing metadata" in AD-5's list is read as just a cross-reference to the same `metadata-updated` event, not a new one.
- **Unit B — built strictly off AD-5's enumeration.** Because AD-5 requires each of the four to be independently emitted "exactly once per attempt, on success or failure" with a degraded-payload path, and `metadata-updated` (AD-4) has no failure/degraded variant defined anywhere, a second engineer introduces a distinct `now-playing-updated` event carrying `{ ok, stationName, favicon, reason }` specifically to give the dashboard a degraded path independent of ICY metadata timing.

**The clash:** Unit A's frontend has one listener and one payload type; Unit B's backend emits an event Unit A never subscribes to, and Unit A's `metadata-updated` never carries the `ok:false` shape Unit B's dashboard logic expects for its "degraded" rendering. The dashboard either double-renders or never receives the failure state it was built to show. The spine cites the same phrase ("now-playing metadata") under two ADs with two different implied contracts and never states they are the same event.

**Recommended fix:** Add an explicit line to AD-4 or AD-5: "`metadata-updated` *is* the 'Now-playing metadata' event referenced in AD-5; no separate event exists for it," and define its degraded/failure payload shape explicitly (does ICY absence — the common case, not a failure — get conflated with a genuine fetch failure? Currently unaddressed).

---

## Pair 3 — Stream Info Tile: one bundled event or two, and sync-from-store vs async-from-event

**Spine text (Structural Seed, Ephemeral shape):** *"Stream info tile data — codec/bitrate/country (mirrors the playing Station's own fields — **no re-fetch needed**) plus the DNS-resolved IP, which *is* genuinely ephemeral and computed by `audio/player.rs` at play-time."* Meanwhile AD-5's Rule says "Stream info" is "its own Tauri event," singular.

- **Unit A — sync-then-async split.** `StreamInfoTile.vue` reads codec/bitrate/country directly and synchronously off the already-loaded `Station` object in `stations.ts` (per "no re-fetch needed" — no event required for these fields), and separately subscribes to a `stream-ip-resolved` event for just the IP once DNS completes. Two data paths, one tile.
- **Unit B — single bundled event.** `audio/player.rs` waits until DNS resolution finishes (or fails) and then emits **one** `stream-info-updated` event carrying `{ codec, bitrate, country, ip }` together, per AD-5's "its own Tauri event" (singular, one event per Info Tile category, same pattern as Weather/Location). `StreamInfoTile.vue` in this build shows nothing at all until that single event fires — including codec/bitrate/country, which are already known instantly from the `Station` struct.

**The clash:** Unit B measurably violates the spirit of "no re-fetch needed" by gating already-known data behind a network-bound event, producing a visible loading-state regression Unit A doesn't have — yet Unit B is a literal, defensible reading of AD-5's "its own Tauri event" (one event, not two). Unit A never emits `stream-info-updated` at all, so a `StreamInfoTile.vue` built against Unit B's contract renders nothing under Unit A's backend. The spine gives two contradictory signals (the seed implies two data paths / two timings, AD-5 implies one event) and never says how many events "Stream info" actually is or whether the static fields are event-sourced or store-sourced.

**Recommended fix:** State explicitly in AD-5 or the Structural Seed: "Stream info's codec/bitrate/country render synchronously from the `Station` object already in `stations.ts`; only the IP is event-sourced, via a dedicated `stream-ip-resolved` event separate from Weather/Location's single-event pattern." Or, if one bundled event is intended, say so and accept the codec/bitrate/country loading-flash it introduces.

---

## Pair 4 — Mutating command signatures are entirely unpinned (illustrated via FR-3 reorder)

**Spine text:** AD-1 pins *where* commands live (the boundary) and their error-shape distinction (network-down vs zero-results). The Consistency Conventions table pins the *return* type (`Result<T, String>`) and wire casing (AD-6). Nothing in the spine pins command **parameter** shapes for any mutating operation — reorder, favorite/unfavorite, volume set, sleep-timer set, settings update.

- **Unit A — incremental reorder command.** `store.rs`/`commands.rs` expose `reorder_favorite(stationId: string, newOrder: number) -> Result<(), String>`. `store.rs` owns shifting every other favorite's `favoriteOrder` to keep them contiguous (consistent with "Rust is the sole owner and mutator of... persisted state"). `StationRow.vue`'s drag handler computes the single target index and calls this per-drop.
- **Unit B — whole-list reorder command.** `commands.rs` instead exposes `reorderFavorites(orderedStationIds: string[]) -> Result<(), String>`, and `StationList.vue`'s drag-drop library (a common pattern for list reorder UIs) hands back the complete new order on every drop, which Rust re-numbers wholesale.

**The clash:** both satisfy AD-1 (command-boundary crossing), AD-6 (camelCase param names), the `Result<T, String>` convention, and "Rust is sole owner/mutator of persisted state" — the spine gives zero basis for preferring one signature over the other. If `StationList.vue` is built to Unit B's contract against a backend built to Unit A's, the call fails to compile/type-check at the Tauri `invoke()` boundary — exactly the kind of integration break AD-1 was supposed to prevent, except AD-1 only governs *that a command exists*, not its shape. This is a systemic gap, not unique to reorder: the same ambiguity applies to every other mutating command in the Capability Map (favorite toggle, volume set, sleep-timer set, settings save) — none of their signatures are specified anywhere in the spine.

**Recommended fix:** Either add a convention row ("mutating commands that operate on collection order/membership always take the full resulting collection, never a delta") or defer explicitly to a companion doc (e.g., a commands.md contract) and say so in "Deferred," rather than silently leaving it to whoever writes each command first.

---

## Pair 5 — Failure-domain ownership clash: is a `player.rs`-side IP-resolution failure a `playback-error` (AD-4) or a degraded Stream Info payload (AD-5)?

**Spine text:** AD-4: *"Rust emits an event on every playback/metadata state change... `playback-error` (retries exhausted — carries a user-facing reason string)."* AD-5: *"a failed fetch emits the same event with a degraded/empty payload... never silence"* and *"This is the concrete mechanism behind NFR-3's 'tiles degrade independently' promise."* Both ADs claim jurisdiction over failures that originate inside `audio/player.rs`, since AD-5's Structural Seed explicitly assigns the IP lookup to `audio/player.rs` ("it already owns the stream URL when `play()` runs — no new module needed for this lookup").

- **Unit A — IP failure is tile-scoped, per AD-5/NFR-3.** DNS resolution for the display-only IP fails, but audio keeps playing fine (e.g., using an already-open socket). `player.rs` emits `stream-info-updated` with a degraded IP field and takes no other action; `play`/`metadata-updated` continue normally. Dashboard shows full playback, Stream Info tile alone shows "unavailable."
- **Unit B — any `player.rs` failure is a playback failure, per AD-4.** Because AD-3/AD-4 assign `audio/player.rs` responsibility for playback-state events and explicitly call out "a failed-after-retry stream silently collapsing into the same visual state as a user-initiated pause" as the exact bug AD-4 prevents, a second implementation treats *any* unrecoverable `player.rs`-internal error — including the IP lookup — as material to playback health, and routes it through the shared error-retry path, eventually firing `playback-error` with a reason string derived from the DNS failure. Audio may still be playing, but the dashboard now shows the "Stream failed" error state (per EXPERIENCE.md / `DESIGN.md.colors.error`) even though the stream itself is fine.

**The clash:** identical failure, same originating module, two contradictory user-visible outcomes — Unit A never blocks/alarms anything outside the one tile; Unit B surfaces a false "Stream failed" banner over a perfectly good stream. Both are literal, good-faith readings of their respective AD. Nothing in the spine states a precedence rule for `player.rs`-originated failures that are non-critical to audio output vs. ones that are.

**Recommended fix:** Add a rule scoping AD-4's `playback-error` strictly to failures that stop or degrade *audio output itself* (decode/connect/retry-exhausted), and explicitly carve out ancillary/display-only lookups performed inside `player.rs` (like the resolved IP) as always routing through AD-5's per-tile degradation path regardless of which module computes them.

---

## Minor / secondary gaps noted but not written up as full pairs

- **Settings live-sync:** AD-4's named event list (`play`, `stop`, `reconnecting`, `playback-error`, `metadata-updated`) has no settings-change event, yet the general rule says Pinia stores are "populated by commands on load, updated by events on change." Two builds could diverge on whether `settings.ts` is updated optimistically from the command's own `Result` (no event) or via a `settings-updated` event nobody is obligated to emit.
- **`stop` and `reconnecting` payload shapes are unspecified** — whether `stop` distinguishes user- vs. system-initiated in its payload (TransportBar arguably needs this to decide UI reset behavior) is left open, similar in kind to Pair 1's discriminant-shape gap.
- **Location Tile source-of-truth timing:** the Capability Map says Location info is "sourced from `directory.rs` (station coordinates, if present)" even though `geoLat`/`geoLong` are already native fields on the persisted/cached `Station` struct. It's not pinned whether the Location Tile reads coordinates already in memory (`stations.ts`) or triggers a fresh `directory.rs` lookup at play-time — the latter would be a redundant per-play network call the spine doesn't explicitly forbid (AD-13's debounce rule is scoped only to "Directory search calls").

---

**File:** `D:\workspace\LLMs\WinRadio\_bmad-output\planning-artifacts\architecture\architecture-WinRadio-2026-08-25\reviews\review-adversarial.md`
