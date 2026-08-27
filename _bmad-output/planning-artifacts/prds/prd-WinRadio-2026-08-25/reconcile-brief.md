---
title: Brief → PRD Reconciliation
created: 2026-08-25
input: brief-WinRadio-2026-08-25/brief.md
downstream: prd-WinRadio-2026-08-25/prd.md
---

# Reconciliation: brief.md → prd.md

Scope of this pass: confirm every meaningful idea, constraint, or nuance from `brief.md` survived into `prd.md`, with special attention to qualitative content a mechanical FR-extraction pass would drop (tone, "why this exists," the relaxed-stakes framing, the out-of-scope list, the broken-scaffold context, the repair-vs-restart open question). The one known, intentional expansion (Location/Weather/Stream Info tiles, logged in PRD §6.1 and §11) is excluded from findings per instructions. Reverse-direction completeness (PRD covering brief word-for-word) was explicitly not required and was not checked as a bar — only substance.

## Gaps / Findings

### 1. Undocumented second scope expansion: "start minimized" pulled into v1 (FR-13)

Brief's Scope section bundles this as one **post-v1 (deferred)** item:
> "Start minimized / auto-start with Windows"

PRD FR-13 (§4.6, Settings & Persistence — which ships in v1 per §6.1) reads:
> "A dedicated settings view exposes tray behavior (minimize-to-tray, start-minimized) and Sleep Timer defaults."

`minimize-to-tray` correctly maps to the brief's v1 item (FR-10 mirrors it too). But `start-minimized` is the other half of the brief's explicitly-deferred bullet, and it has been folded into a v1 FR without comment. §6.2 (Out of Scope for MVP) does still list "auto-start with Windows" as deferred, so the brief's single bullet has been silently split — one half stayed deferred, the other half became v1 — with no delta note, unlike the one scope change the PRD does document (§6.1/§11 for the Info Tiles). This looks like exactly the kind of quiet scope creep the brief's whole premise (agents re-deciding scope mid-flight) was written to prevent.

**Suggested fix:** either move `start-minimized` back out of FR-13 into the deferred list, or add a delta note/assumption entry documenting it as a second deliberate expansion.

### 2. Brief's concrete "under a few clicks" usability bar has no PRD equivalent

Brief's Success Criteria:
> "Javad can search for a station, play it, and favorite it in under a few clicks"

This is a specific interaction-efficiency bar, distinct from latency (NFR-2 covers "within a few seconds") and distinct from retention (PRD §7 SM-1 covers "opens and plays... at least weekly"). Nothing in the PRD's Success Metrics (§7) or NFRs (§8) restates a click-count / interaction-friction bar for the core find-play-favorite flow. It's a minor, soft criterion, but it's a concrete, testable UX nuance from the brief that a pure FR-extraction pass would naturally drop (it's not a feature, it's a quality bar on existing features), and nothing in the PRD currently substitutes for it.

**Suggested fix:** fold into NFR-2 (performance) or add as a UJ-1 acceptance note — "favoriting a previewed station takes no more than N clicks" or similar.

### 3. Brief's specific broken-scaffold evidence is replaced, not carried forward

Brief's "The Problem" section grounds the "app doesn't compile" claim in specific evidence:
> "a `commands` module that exists but is never wired into `lib.rs`, a `router-view` with no router, components referenced that only partially exist"

This evidence doesn't appear anywhere in the PRD. PRD §10 Q3 references a different, non-overlapping set of specifics (`PlayerControls.vue`, `SettingsModal.vue`, "the two Pinia stores") in the context of the repair-vs-restart question — useful, but not the same evidence, and it doesn't restate *why* the original evidence mattered (i.e., that it's proof the prior agents were circling/re-deciding architecture rather than converging, which is the brief's central diagnosis). PRD §0 compresses this to one clause: "prior coding-agent attempts circled without a written spec." The *mechanism* of the circling (unwired commands module, router pointing nowhere, half-built components) — which is useful diagnostic detail for whoever picks up the repair-vs-restart decision in the architecture phase — is not preserved.

**Suggested fix:** low priority given Q1/Q3 already flag the decision is deferred to architecture, but worth a one-line pointer back to brief §"The Problem" for the specific evidence, since the architecture phase will need it and the PRD is the more likely document to be read next.

### 4. (Minor, low-confidence) "No onboarding flow needed" framing not restated

Brief's "Who This Serves" is explicit: "No secondary personas, no growth considerations, no onboarding flow needed — it opens straight into 'search or pick a favorite and hit play.'" PRD §2 covers the single-user/no-growth angle well (JTBD list, and the "hobby/solo, single operator" scope-dial note in §2.3), and UJ-1's narrative implicitly shows a no-onboarding flow. But the explicit "no onboarding flow needed" constraint itself isn't restated anywhere, so nothing in the PRD would stop a future FR from introducing a welcome/setup flow. Low severity — the JTBD and UJ framing make onboarding unlikely by implication — but it's the kind of explicit constraint that's cheap to keep and easy to lose.

## Non-findings (confirmed present, no action needed)

- Relaxed-project / no-roadmap-pressure tone: present via PRD §0 ("sole PM, sole user"), §1 ("no roadmap beyond... no ambition beyond that"), §2.1 JTBD ("no ads, no account, no bloat").
- Explicit out-of-scope list (recording, equalizer, casting, accounts/sync/mobile): fully carried in §5 Non-Goals and §6.2, explicitly marked "carried from brief.md, still true."
- Post-v1 deferred list (recently-played, custom groups/tags, media keys, manual stream URL, per-station volume): fully carried in §6.2, except the start-minimized split noted in Finding 1.
- Repair-vs-restart open question: explicitly carried forward verbatim as PRD §10 Q1, correctly still deferred to architecture.
- Station directory / Radio-Browser API assumption, sleep-timer auto-stop-only assumption: both carried forward with `[ASSUMPTION: ... carried from brief.md]` tags.
- "Reaches for WinRadio instead of a browser tab" success framing: carried in PRD §1 Vision and §7 SM-1.
