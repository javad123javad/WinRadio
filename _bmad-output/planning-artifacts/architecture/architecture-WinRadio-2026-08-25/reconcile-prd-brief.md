---
title: Input Reconciliation — PRD & Brief vs. Architecture Spine
created: 2026-08-25
sources:
  - '../../prds/prd-WinRadio-2026-08-25/prd.md'
  - '../../briefs/brief-WinRadio-2026-08-25/brief.md'
  - './ARCHITECTURE-SPINE.md'
---

# Reconciliation: PRD/Brief → Architecture Spine (WinRadio)

## Method

For each FR-1..FR-14 and NFR-1..NFR-5, checked whether the spine gives it an explicit home: an AD's `Binds:` line, a Consistency Convention, a Capability→Architecture Map row, or a Deferred entry. Separately checked whether the brief's qualitative constraints (relaxed personal-project framing, the specific scaffold-bug evidence, the repair-vs-restart open question) survived into the spine intact and consistent.

## FR coverage (FR-1 through FR-14)

All 14 FRs have an explicit home. Every FR appears in the frontmatter `binds:` list and in the Capability → Architecture Map:

| FR | Map row | ADs |
| --- | --- | --- |
| FR-1, FR-2 | Station Discovery & Search | AD-1, AD-6 |
| FR-3, FR-4 | Favorites & Station List | AD-1, AD-6 |
| FR-5 | Playback & Transport | AD-3, AD-4 |
| FR-6–FR-9 | Now-Playing Dashboard & Info Tiles | AD-4, AD-5 |
| FR-10, FR-11 | System Tray & Sleep Timer | AD-4 |
| FR-12, FR-13 | Settings & Persistence | AD-6, AD-9 |
| FR-14 | Theming | — (DESIGN.md tokens) |

No FR gap. Two sub-FR items have thinner-than-expected homes worth flagging (see Gaps 4–5 below): FR-7's map-rendering mechanism and FR-9's DNS-IP-resolution mechanism don't get the same explicit crate/module assignment that other integrations (Radio-Browser, Open-Meteo, audio stack) received.

## NFR coverage (NFR-1 through NFR-5)

Grepped the spine for `NFR-1` .. `NFR-5` by ID. Results:

- **NFR-3** (Reliability/graceful degradation) — explicit: `AD-5` binds it directly, and its rule text is called out as "the concrete mechanism behind NFR-3's tiles degrade independently promise." AD-1 also references the "NFR-3 degradation contract." **Covered.**
- **NFR-5** (Privacy — no telemetry) — explicit: AD-11 cites it ("no telemetry (also required by PRD NFR-5)"). **Covered.**
- **NFR-1** (Platform: Windows 10/11, Tauri only) — **no explicit binding anywhere.** Implicitly satisfied by AD-7 (Tauri 1.5 retained) and the Stack table, but the spine never cites NFR-1 by ID and there's no AD/convention/Deferred entry that names it as the thing being satisfied.
- **NFR-2** (Performance: search ≤3s, playback start ≤2s, idle memory <~150MB resident) — **no home at all.** No AD, convention, map row, or Deferred entry addresses latency budgets or memory footprint. Nothing in the spine (e.g., connection pooling/caching for Radio-Browser calls, bounding event-payload retention, tray-idle resource behavior) speaks to this NFR even implicitly.
- **NFR-4** (Offline handling: clear state on no network, auto-recovery) — **no explicit home.** AD-3's reconnect-on-drop logic (via `stream-download`) covers an active stream dying mid-session (which overlaps FR-5's edge case), but general "no network at all" detection/recovery for search (FR-1) and the rest of the app isn't named anywhere as an architectural concern.

The frontmatter `binds:` field itself only lists FR-1..FR-14 — no NFR IDs appear there at all, which is consistent with the gaps found above (the spine's own self-declared scope of "what this document binds" omits NFRs entirely, even the two it does address in prose).

## Brief's qualitative constraints

- **"Relaxed personal project" framing** — landed correctly. The spine explicitly invokes it ("Six features on a solo project doesn't earn that ceremony," single-user/single-machine language, AD-11's deferral of CI/CD/auto-update/code-signing, Deferred section's "not this altitude's call" for test strategy). Consistent with the brief's and PRD's tone throughout.
- **Repair-vs-restart open question** (brief §Open question / PRD §10 Q1) — resolved, not left open. AD-2 gives a nuanced hybrid answer ("extend the Rust core, rewrite the Vue frontend") rather than a flat repair-or-restart pick. This is a legitimate, well-reasoned resolution of the deferred question and is internally consistent with the rest of the spine (AD-8, AD-9, AD-10 all describe specific Rust-side deletions/fixes — i.e., "extend," not "restart from zero"; the Structural Seed marks Vue files as rewritten and Rust files as existing/extended).
- **PRD Open Question 3** (reuse `PlayerControls.vue`, `SettingsModal.vue`, the two Pinia stores, or reference-only?) — answered by the same AD-2 rule: components and both Pinia stores are rewritten from scratch, "no existing component markup reused as a starting point." Consistent.
- **PRD Open Question 2** (codec breadth) — explicitly closed out in Deferred, citing the question by number and resolving it via `rodio`'s `symphonia-all` feature. Consistent.

## Gaps found

**1. NFR-2 (Performance/memory budget) has no architectural home.** No AD, convention, Capability→Architecture Map row, or Deferred entry addresses the PRD's explicit 3-second search / 2-second playback-start / ~150MB idle-memory targets. This is the single clearest FR/NFR with zero home in the spine — not even a Deferred acknowledgment that it's out of scope for this altitude.

**2. NFR-1 (Windows/Tauri-only platform) and NFR-4 (offline handling) are never cited by ID.** Both are arguably satisfied in spirit (NFR-1 via AD-7 + Stack table's Windows/Tauri commitments; NFR-4 partially via AD-3's stream-reconnect behavior), but neither is named or bound anywhere, unlike NFR-3 and NFR-5 which get explicit call-outs. NFR-4's coverage is also only partial: AD-3 covers an active stream dropping mid-play, not a general "no network at all" state for search/Directory calls.

**3. Corrected-but-unflagged discrepancy in the scaffold-bug evidence (the specific item the task asked to check).** The brief states the evidence is "a `commands` module that exists but is never wired into `lib.rs`." The PRD restates it as "`lib.rs` never declares the `commands` module even though other files depend on it" — both describing a *missing declaration* that would break compilation. The spine's AD-8, which is clearly informed by an actual code read during architecture, describes a **different** bug: "the dual bin+lib compilation found in the current code (`lib.rs` duplicating `main.rs`'s module tree with nothing consuming the lib target)" — i.e., `lib.rs` *does* declare the modules (redundantly, duplicating `main.rs`), and the real problem is an orphaned/unconsumed lib compilation target, not a missing declaration. This is a legitimate correction of the upstream artifacts' evidence, but the spine doesn't flag it as a correction anywhere — a reader comparing brief/PRD to spine would see two different bug descriptions with no note that the later one supersedes the earlier one. Recommend either a one-line note in AD-8 acknowledging the correction, or flagging it back to the PRD/brief for an update.

**4. FR-7's map-rendering mechanism is unspecified.** PRD's assumption is "a lightweight embedded map (OpenStreetMap-based, no paid API key)." The spine names `LocationTile.vue` in the Structural Seed and covers FR-7 under AD-4/AD-5 in the Capability map, but never names a library/technique for the OSM embed (e.g., a tile-image approach vs. a JS map library) in the Stack table, unlike every other external integration (Radio-Browser, Open-Meteo, `stream-download`+`icy-metadata` all get explicit crate/version pins). Minor gap — worth either a Stack table entry or an explicit Deferred note.

**5. FR-9's DNS-resolved stream IP has an ambiguous "lives in" home.** The Capability→Architecture Map lists FR-6–FR-9 as living in `audio/player.rs` (metadata) and `directory.rs`/`weather.rs` (tile data), but the DNS lookup of the stream host (PRD's FR-9 assumption) fits none of these cleanly — it's not ICY metadata, not Directory catalog data, and not weather data. Minor ambiguity in an otherwise precise map.

## No gaps in

FR-to-architecture coverage overall (all 14 FRs homed), NFR-3 and NFR-5 coverage, the "relaxed personal project" tone, and the repair-vs-restart / component-reuse open-question resolutions.
