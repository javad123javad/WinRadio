---
title: Rubric Review — ARCHITECTURE-SPINE.md (WinRadio)
reviewed: ARCHITECTURE-SPINE.md, cross-checked against prd.md, .memlog.md, DESIGN.md, EXPERIENCE.md, and the live winradio/ codebase
date: 2026-08-25
---

# Rubric Review: WinRadio Architecture Spine

## Overall Verdict

Solid, well-evidenced spine — every brownfield claim it makes (serde casing bug, `lib.rs` dual-module correction, dangling `router-view`, unused `stream-download` pin, EQ/recording deps) checks out against the actual repo, and the ADs are concrete and enforceable; it has three real gaps worth closing before epics/stories: two NFRs silently absent from the Capability→Architecture Map, no bounded retry/backoff policy for the one stated reliability edge case (stream reconnect), and an OSM attribution requirement the spine claims policy-compliance on without providing.

## Findings

### 1. NFR-3 and NFR-5 are bound by ADs but missing from the Capability → Architecture Map

- **Where:** `ARCHITECTURE-SPINE.md` §"Capability → Architecture Map", last row: `Performance & platform (NFR-1, NFR-2, NFR-4)`.
- **Issue:** The map's own frontmatter (`binds:`) commits to all five NFRs, and the AD prose does bind NFR-3 (AD-5's Binds line: `FR-7, FR-8, FR-9, NFR-3`) and references NFR-5 (AD-11: "no telemetry (also required by PRD NFR-5)"). Neither NFR-3 nor NFR-5 appears anywhere in the Map table, so a reader using the Map as the coverage summary (which is exactly what this review task and any downstream epic-writer would do) would conclude they're unaddressed. This is the specific artifact the review brief asked to cross-check coverage claims against — and it currently under-reports its own coverage.
- **Fix:** Add NFR-3 and NFR-5 to the last row (or a new row), e.g. `Performance & platform (NFR-1, NFR-2, NFR-3, NFR-4, NFR-5)` governed by `AD-1, AD-5, AD-7, AD-11, AD-13`.

### 2. Stream reconnect has no bounded retry/backoff policy — a real cross-unit divergence point left undecided

- **Where:** AD-4's rule enumerates `reconnecting` and `playback-error (retries exhausted...)` as events but never states how many attempts, what backoff, or what timeout separates "reconnecting" from "exhausted." EXPERIENCE.md (line 70) requires the transport bar to show "Reconnecting…" with controls "disabled until it resolves or fails," and FR-5's consequence requires an automatic retry before surfacing an error — but the actual retry policy is the one piece of behavior that is simultaneously (a) explicitly called out as a UJ-1 edge case the PRD cares about, (b) split across two independently-built layers (`audio/player.rs` owns the retry loop; the Vue transport bar owns how long it sits in "Reconnecting…"), and (c) completely unspecified in the spine and absent from the Deferred list.
- **Why it matters:** Without a stated policy (e.g., "3 attempts, exponential backoff starting at 2s, total budget 30s"), a story implementing the backend retry loop and a story implementing the frontend's disabled-controls window have no shared contract — one could pick "retry forever" and the other could assume "fails within 5s," producing exactly the kind of two-independently-built-units divergence this rubric flags. This is a real divergence point AD-4 should fix but doesn't.
- **Fix:** Add a retry policy line to AD-4 (attempt count, backoff shape, total timeout) or explicitly add it to Deferred with a note that it's a story-level decision constrained by [these bounds].

### 3. AD-12's OSM policy-compliance claim is asserted, not verified, and the rule as written doesn't include OSM's actual attribution requirement

- **Where:** AD-12's Rule: renders the tile via a plain `<img>` in `LocationTile.vue`, no library, no key. AD-12's Prevents/rationale text asserts "Personal, low-volume, single-tile-at-a-time usage is consistent with OSM's tile usage policy."
- **Issue:** Per the review brief's instruction to check whether tech/service claims show verification evidence (version-with-date, citation) vs. bare assertion: `.memlog.md` shows a real verification trail for `stream-download`/`icy-metadata` (versions + dates, "Aug 2026"/"Jun 2026") and for the Tauri v1-vs-v2 decision, but contains zero entries about checking OSM's tile usage policy — the claim in AD-12 is a bare assertion. More concretely, OpenStreetMap's standard tile usage policy conditions any use of `tile.openstreetmap.org` on displaying the "© OpenStreetMap contributors" attribution notice; neither AD-12's Rule nor DESIGN.md's Info Tile component spec (which the Rule points to for "same background, radius, and padding") mentions rendering attribution text anywhere near the tile. As written, the enforceable rule ("compute a URL, render via `<img>`") does not actually keep the app compliant with the policy it claims to satisfy.
- **Fix:** Either add the attribution requirement to AD-12's Rule (and to `LocationTile.vue`'s spec in the Structural Seed / to DESIGN.md), or soften the claim to an open question pending an explicit attribution-placement decision.

### 4. AD-13 claims to enforce NFR-2 "at specific points," but only covers the memory/redundant-work sub-requirement, not the two latency SLAs

- **Where:** AD-13's Prevents/Rule text addresses debounce (prevents redundant search calls) and single-active-`Sink` (prevents memory creep), plus points to AD-8/AD-9 for idle footprint. NFR-2 itself has three independent numeric targets: search returns within 3s, playback starts within 2s, and idle RAM under ~150MB.
- **Issue:** AD-13's opening line — "Performance budget (NFR-2) is enforced at specific points, not left to hope" — reads as covering all of NFR-2, but the two latency SLAs (3s search, 2s playback start) have no corresponding mechanism anywhere in the spine (no timeout/cutoff, no loading-state budget, no explicit "if X takes longer than Ys, do Z"). Only the memory-related third of NFR-2 is actually operationalized.
- **Fix:** Either add a concrete mechanism for the latency SLAs (e.g., a client-side timeout after which the search/play-start shows a distinct "taking longer than usual" state) or narrow AD-13's framing to state explicitly that it covers only the memory/footprint dimension of NFR-2, leaving the latency SLAs as a QA/testing-strategy concern (consistent with "Automated test strategy" already being deferred).

## Checklist Walk (for traceability)

| Checklist item | Verdict |
| --- | --- |
| Fixes real divergence points for epics/stories, misses none | Mostly — retry/backoff policy (Finding 2) is a real miss. |
| Every AD's Rule is enforceable and prevents its stated divergence | Yes for AD-1,2,3,4,6,7,8,9,10,11,13; AD-12's rule under-delivers on its own compliance claim (Finding 3). |
| Nothing under Deferred lets two independently-built units diverge in a way that matters | Deferred list itself is fine (CI/CD, auto-update, code signing, logging, test strategy, multi-arch, PRD §6.2 items, codec breadth — all verified low-risk / correctly scoped-out). The risk is an *omission*, not a bad Deferred entry: retry policy (Finding 2) should be there (or in an AD) and isn't either place. |
| Named technology is verified-current | Good where it matters: `stream-download` 0.24.3 (Aug 2026) and `icy-metadata` 0.6.0 (Jun 2026) both carry version+date evidence in `.memlog.md`; Tauri 1.5-vs-2.10.1 (March 2026) decision is explicitly reasoned. Versions carried over unchanged from the existing `package.json`/`Cargo.toml` (Vue 3.4, Pinia 2.1, TS 5.3, Vite 5.0, Tailwind 3.4, rodio 0.19, reqwest 0.12, tokio 1.38) were verified against the live repo during this review and match exactly — they're inherited pins, not fresh adoptions, so absence of re-verification is reasonable. The one bare, unverified claim is OSM's tile usage policy (Finding 3). |
| Ratifies rather than contradicts the brownfield codebase | Yes, and unusually well-substantiated. Directly verified against the live repo during this review: `lib.rs` does declare all 5 modules (confirms AD-8's "correction of record" — the brief/PRD's missing-declaration claim is indeed false, the real issue is the redundant dual bin+lib target); `App.vue` does render a bare `<router-view/>` with no `vue-router` anywhere (confirms AD-10); `commands.rs` structs do use snake_case (`favicon_url`, `is_favorite`, `added_at`) with no `#[serde(rename_all)]` (confirms AD-6's bug claim); `Cargo.toml` does carry `lame-sys`, `hound`, `cpal`, and an already-declared-but-unused `stream-download = "0.5"` (confirms AD-9 and AD-3's version-bump framing); rodio's `Cargo.toml` entry already carries `features = ["symphonia-all"]` (confirms the Deferred section's codec-breadth claim). No contradictions found. |
| Covers the PRD's capabilities (FR/NFR cross-check) | All FR-1–FR-14 appear in the Map. NFR-1, NFR-2, NFR-4 appear in the Map; NFR-3 and NFR-5 are decided in AD prose but absent from the Map itself (Finding 1). |
| Every dimension the altitude owns is decided/deferred/open, especially deployment/environments/infra/operations | Deployment/environments/infra/ops is well covered and explicitly not left silent: AD-11 (no server, `tauri build` → unsigned installer, manual install, no CI/auto-update/telemetry) plus `.memlog.md`'s explicit note that the user required this not be left silent. The one dimension left partly to hope is the latency half of the performance envelope (Finding 4). |

## Summary of Recommended Actions

1. Add NFR-3, NFR-5 to the Capability → Architecture Map.
2. Add a concrete retry/backoff/timeout policy to AD-4 (or explicitly defer it with bounds).
3. Add an attribution mechanism to AD-12's Rule (or soften/flag the OSM-policy-compliance claim as unverified).
4. Either add a mechanism for NFR-2's two latency SLAs to AD-13, or narrow AD-13's stated scope to the memory/footprint dimension only.
