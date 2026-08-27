# PRD Quality Review — WinRadio

## Overall verdict

This PRD is well-calibrated to its actual stakes: a hobby, single-operator desktop app with one deliberately-scoped user journey, an honest scope-delta note, and a well-built Assumptions Index. It holds up on thesis, scope honesty, and shape fit — the deliberate down-scoping of personas/UJs is a feature, not a gap, and the §6.1 delta note is a model of how to document an honest scope increase. The one dimension that needs tightening before this feeds story creation is done-ness clarity: NFR-2's performance bounds are adjectives ("a few seconds," "lightweight enough," "noticeable resource drain") rather than numbers, and several FRs lean on adjacent feature-level NFRs for their testability rather than stating it locally. Nothing here rises to critical or broken.

## Decision-readiness — strong

Trade-offs are named, not smoothed over. §6.1's delta note doesn't just announce the Info Tile scope increase — it names what that increase risks ("NFR-3... exists specifically to keep this addition from reintroducing the fragility... that caused the original circling problem") and points at the artifact of record (`.memlog.md`) rather than folding the change in silently. The `[NOTE FOR PM: revisit if this stops being purely personal-use]` at §6.2 sits at a real tension (English-only UI vs. the reference image's language toggle), not a safe checkpoint. §10's three Open Questions are genuinely unresolved and carry concrete evidence for why (the `lib.rs`/`App.vue`/component citations from brief.md) rather than being rhetorical questions answered in the next sentence.

No findings — this dimension does real work.

## Substance over theater — strong

One persona, one UJ, no differentiation/innovation section, no roadmap beyond "Javad actually uses it" (§1) — appropriately spare for a solo hobby tool, not padded to look thorough. The Vision statement (§1) is specific enough that it couldn't swap into another PRD unchanged: it names the exact thing being replaced (a pinned browser tab), the exact gap being closed (favorites/settings memory, a real now-playing view), and explicitly caps ambition ("no roadmap beyond..."). NFRs are mostly product-specific rather than boilerplate — NFR-3 and NFR-5 in particular name exact FR ranges and exact data categories rather than saying "the system must be secure." The one soft spot (NFR-2's vague thresholds) is a done-ness problem, not a theater problem — the *intent* behind NFR-2 is specific to this app (tray-resident, indefinite idle time), it just isn't bounded numerically. See Done-ness clarity.

No standalone findings — see Done-ness clarity for the related NFR-2 note.

## Strategic coherence — strong

The thesis is stated once and the feature set visibly serves it: replace the browser-tab-with-a-stream habit by adding memory (favorites/persistence) and presence (tray) without adding the bloat that killed prior attempts. Prioritization follows the thesis, not ease — the Info Tiles (§4.4), the one feature that *isn't* core to that thesis, get their own guardrail NFR (feature-specific NFR under §4.4) specifically so they can't compromise the core loop. Success Metrics validate the thesis rather than measuring activity: SM-1 is about the core loop replacing the browser tab, not raw usage counts, and SM-C1 is a genuine counter-metric that directly opposes the temptation the Info Tile addition creates ("Number of Info Tiles or dashboard features is not a target to maximize... more surface area is exactly the kind of scope drift that stalled the previous attempts"). MVP scope kind reads as experience/problem-solving, and the scope logic (§6.1/§6.2) matches that framing throughout.

No findings — this dimension does real work.

## Done-ness clarity — adequate

Most core-loop FRs (FR-1, FR-3, FR-5, FR-11) carry explicit "Consequences (testable)" blocks with verifiable conditions (error states, persistence behavior, retry behavior). But this pattern isn't applied evenly, and the one place it's genuinely missing teeth — NFR-2 — is exactly the dimension the rubric asks to be unforgiving about.

### Findings
- **medium** NFR-2 states bounds as adjectives, not numbers (§8) — "Search results and station playback should start within a few seconds," "the app should stay lightweight enough to sit in the tray indefinitely without noticeable resource drain." Neither "a few seconds" nor "lightweight"/"noticeable" is a number an engineer or a future story can test against, and FR-1's own consequence defers to this same vague bound ("within a reasonable wait (see NFR-2)"). *Fix:* Replace with concrete thresholds Javad would actually notice missing — e.g. "search results return within 3s on a normal connection," "idle RSS stays under Xmb while docked in the tray."
- **low** Several FRs (FR-2, FR-4, FR-6, FR-7, FR-8, FR-9, FR-10, FR-12, FR-13, FR-14) have no "Consequences (testable)" block, unlike their FR-1/3/5/11 siblings. For the simple toggle/click FRs (FR-4, FR-14) the one-line FR statement is arguably self-testable and this is fine. But FR-7/8/9 (Location/Weather/Stream Info Tiles) carry real failure modes — map fails to render, weather API unreachable, DNS resolution of the stream host fails — and their done-ness currently lives only in the shared feature-level NFR under §4.4 ("shows a quiet empty/placeholder state"), not per-FR. An engineer implementing FR-9 alone would need to cross-reference that shared note to know what "done" looks like on failure. *Fix:* Either give FR-7/8/9 their own one-line failure-state consequence, or add an explicit forward-reference from each FR to the §4.4 feature-specific NFR.

## Scope honesty — strong

§5 Non-Goals does real work with three specific, falsifiable bullets (no local files/podcasts, no accounts/sync, no plugin system) rather than a boilerplate disclaimer. §6.2 functions as the `[NON-GOAL for MVP]` mechanism the rubric asks for, and it's itemized rather than hand-waved (start-minimized, auto-start, media keys, custom-URL entry, per-station volume memory, each named individually). Eight `[ASSUMPTION]` tags are used precisely where the user didn't directly confirm something (API choices, map/weather providers, IP-resolution method, localization), all round-tripped into §11. The §6.1 delta note is the standout: it names the exact prior baseline (brief.md's scoped v1), the exact addition, the exact trigger (a shared reference screenshot), and where it's logged (`.memlog.md`) — de-scoping/re-scoping done honestly, not silently. Open-items density (3 Open Questions + 8 Assumptions + 1 NOTE FOR PM) is appropriate for a hobby/solo PRD per the rubric's own calibration guidance — nowhere near excessive for these stakes.

No findings — this dimension does real work.

## Downstream usability — strong

This PRD does feed downstream (§9 explicitly hands visual design to bmad-ux next; §10's Open Questions feed the architecture phase), so this dimension isn't waived. The Glossary (§3) is used consistently: Station, Directory, Favorite, Station List, Now-Playing Dashboard, Info Tile, Sleep Timer, and Tray all appear with matching capitalization across §§4, 7, and 8. FR IDs (FR-1–FR-14) and NFR IDs (NFR-1–NFR-5) are contiguous with no gaps or duplicates. The single UJ has a named protagonist (Javad) carrying context inline, and every feature section correctly cross-references it ("Realizes UJ-1"). A few minor mechanical nits (case drift, a numbering gap, SM-to-FR traceability) are noted below — none of them block clean extraction.

No dimension-level findings — see Mechanical notes.

## Shape fit — strong

The PRD matches its stated stakes deliberately rather than being forced into a template. Per §0 and the "Lighter scope dial" callout at §2.3, it consciously downscoled to one persona and one UJ instead of a formal multi-UJ set — exactly the "hobby/solo → rigor light, substance bar still applies" calibration the rubric asks for, and it isn't under-formalized either: Glossary, per-feature FRs, an Assumptions Index, and Cross-Cutting NFRs are all still present and doing work. Brownfield handling is accurate and specific — §10's Open Questions cite exact files (`lib.rs`, `App.vue`, `PlayerControls.vue`, `SettingsModal.vue`, two Pinia stores) rather than gesturing at "the existing codebase," and Q3 explicitly separates the reuse-vs-reference-only decision from the repair-vs-rewrite decision (Q1) instead of conflating them.

No findings — this dimension does real work.

## Mechanical notes

- **Section numbering gap**: §2 "Target User" jumps from "2.1 Jobs To Be Done" directly to "2.3 Key User Journeys" — there is no 2.2. This is almost certainly the artifact of removing a Persona subsection when the PRD downscaled to a single implicit persona (Javad), but the numbering wasn't closed up. Cosmetic only, but worth fixing so a reader doesn't go hunting for a missing 2.2.
- **Glossary case drift**: §3 defines **Sleep Timer** (capitalized), and §4.5's section title and FR-11 body both use "Sleep Timer" correctly — but the FR-11 heading itself reads "Sleep timer" (lowercase 't'). Trivial, but worth normalizing since downstream extraction may grep on the capitalized term.
- **SM-to-FR traceability gap**: §7's SM-1 states "Validates FR-1 through FR-5, FR-10" and SM-2 validates NFR-3 only. FR-6 through FR-9 (Now-Playing Dashboard / Info Tiles) and FR-12 through FR-14 (persistence, settings panel, theming) have no explicit Success Metric linkage. Likely fine for a hobby PRD where not every FR needs a metric, but worth a conscious call rather than a silent gap if this PRD is used as a checklist later.
- **Assumptions Index near-duplicate**: the inline `[ASSUMPTION: this feature set is the scope expansion agreed with Javad in this session — see §6.1 delta note]` at §4.4 doesn't get its own line in §11; its substance is covered by the §6.1 entry ("Location/Weather/Stream Info tiles are a deliberate v1 scope increase..."), so the roundtrip isn't literally broken, just merged. No action needed unless strict 1:1 tagging is required downstream.
- ID continuity otherwise clean: FR-1–FR-14 and NFR-1–NFR-5 have no gaps or duplicates; the single UJ-1 is referenced consistently from every feature section that realizes it.
