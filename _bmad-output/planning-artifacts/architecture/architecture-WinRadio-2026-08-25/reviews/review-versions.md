# Review: Technology Verification Lens — ARCHITECTURE-SPINE.md

**Target:** `_bmad-output/planning-artifacts/architecture/architecture-WinRadio-2026-08-25/ARCHITECTURE-SPINE.md`
**Lens:** Were committed technology decisions actually web-researched / reality-checked, or asserted from training data?
**Date:** 2026-08-25
**Method:** Live web fetches (crates.io API, docs.rs, lib.rs, operations.osmfoundation.org, open-meteo.com, tauri.app, radio-browser.info) plus direct inspection of the existing `winradio/` repo (`src-tauri/Cargo.toml`, `package.json`, `src-tauri/src/lib.rs`).

## Verdict: **FAIL — rework required before this spine can be treated as build-substrate**

Most version numbers quoted in the Stack table are real and (for the Vue/TS/Tauri-1.5 side) verified to match the existing repo exactly — that part looks genuinely checked, not hallucinated. But the crate pairing at the center of AD-3 (`stream-download` 0.24 + `icy-metadata` 0.6) was asserted with `[ADOPTED]`-level confidence without actually opening `icy-metadata`'s own manifest, and it fails on inspection: its default features silently pull a *second, incompatible* `reqwest` major line into the binary, contradicting the Stack table's own `reqwest 0.12` entry. AD-12's OSM claim has the same shape — technically-true-sounding, but wrong on the two mandatory conditions of the actual policy it cites, and it silently contradicts AD-1 elsewhere in the same document.

## Findings

### 1. [HIGH] `icy-metadata 0.6`'s default features pull `reqwest 0.13`, silently conflicting with the pinned `reqwest 0.12`
**Confirmed via `docs.rs/crate/icy-metadata/0.6.0/source/Cargo.toml.orig`:**
```
[dependencies]
reqwest = { version = "0.13", default-features = false, optional = true }
...
[features]
reqwest = ["dep:reqwest"]
default = ["reqwest"]
```
`icy-metadata`'s `reqwest` feature is **on by default**, and it depends on `reqwest 0.13` — a separate, incompatible major-lineage release from the `reqwest 0.12` the Stack table commits to (and which the existing `Cargo.toml` already pins). A bare `icy-metadata = "0.6"` add, exactly as AD-3 and the Stack table specify with no `default-features = false`, pulls both `reqwest` 0.12 *and* 0.13 into the dependency graph — two HTTP client stacks in a small tray-resident app whose own NFR-2 rule (AD-13) is explicitly about minimizing footprint bloat. This is not a training-data-plausible guess that happened to be wrong in a subtle way — it's the kind of thing that surfaces the moment you `cargo add icy-metadata` or open its manifest, which does not appear to have happened before the AD was written as settled fact.
**Fix:** either pin `icy-metadata = { version = "0.6", default-features = false }` (dropping its reqwest convenience helpers, which the design doesn't appear to need since `stream-download` — not raw `reqwest` — owns the HTTP fetch) or explicitly acknowledge and justify carrying two reqwest versions.

### 2. [MEDIUM] The `stream-download 0.24` + `icy-metadata 0.6` + `rodio 0.19` three-way pairing is unverified
`icy-metadata 0.6.0`'s own dev-dependencies (used for its test suite) pin `stream-download = "0.23.0"` and `rodio = "0.21.1"` — not the spine's `stream-download 0.24` / `rodio 0.19`. Individually every version number named in the Stack table does exist on crates.io (`stream-download` 0.24.3 and `icy-metadata` 0.6.0 are both real, current releases — confirmed via the crates.io API), so this isn't fabrication of version numbers. But the *combination* the AD commits to has not been shown to actually build together — 0.x crates are not guaranteed semver-compatible one minor version ahead of what they were tested against, and rodio 0.19 → 0.21 spans two minor releases the current pairing was never exercised against. This should be a build spike before AD-3 is marked adopted, not an assumption.

### 3. [HIGH] AD-12's OSM tile plan does not satisfy the tile usage policy it claims consistency with
Fetched `operations.osmfoundation.org/policies/tiles/` directly. The policy conditionally allows exactly this use case (single-tile hotlinking from a personal desktop app) but **requires**:
- a "clear, unique User-Agent string that names your app" — explicitly: "Do not use library default User-Agents or impersonate other applications."
- visible OSM attribution ("© OpenStreetMap contributors") on the map.

AD-12's actual mechanism — `<img src="https://tile.openstreetmap.org/{z}/{x}/{y}.png">` rendered directly by `LocationTile.vue` — sends whatever default User-Agent the Tauri/WebView2 webview emits, not an app-identifying string, and neither AD-12 nor the Stack table row nor the Structural Seed mentions attribution. The sentence "Personal, low-volume, single-tile-at-a-time usage is consistent with OSM's tile usage policy" is only half true: the *volume* profile is fine, but the *implementation as specified* fails the policy's two mandatory conditions. This reads as a claim checked against general awareness of "OSM tiles are free and hotlinkable at low volume," not against the actual policy text.
**Fix:** route tile fetches through a Rust command (reqwest with a custom `User-Agent: WinRadio/x.y (+contact)` header) and add attribution text/overlay to `LocationTile.vue`.

### 4. [MEDIUM] AD-12 silently contradicts AD-1's own command-boundary rule
AD-1 states: "Every network or filesystem operation is a Tauri command. The frontend has zero `fetch()`/`XMLHttpRequest` calls to external hosts," listing as a prevented failure mode "Vue code independently calling Radio-Browser/Open-Meteo/**filesystem** directly." AD-12's `<img>`-tag hotlink to `tile.openstreetmap.org` is Vue code calling an external host directly — the same failure class AD-1 is written to prevent, just via `<img src>` instead of `fetch()`. Neither AD notes or reconciles this exception. (Finding #3's fix — proxying tiles through a Rust command — would also resolve this inconsistency, which is further evidence the two ADs weren't cross-checked against each other.)

### 5. [LOW / informational] Tauri 1.5 retention is factually accurate about current state, but understates the maintenance-branch tradeoff
Verified against the actual repo: `src-tauri/Cargo.toml` already pins `tauri = "1.5"` / `tauri-build = "1.5"`, and `package.json` pins `@tauri-apps/cli: "^1.6.3"` — so AD-7's premise (this is the existing, working version) is correct, not asserted. Independently confirmed via web search that as of 2026 Tauri's active development line is v2 (2.10.x+), with v1 receiving only backported security/bug fixes rather than new development. AD-7's decision to stay on 1.5 for a Windows-only personal app is defensible, but the AD doesn't name this tradeoff explicitly (i.e., "retained" means depending on a maintenance-only branch going forward) — worth one added sentence, not a blocker.

### 6. [Confirmed accurate — no finding] Radio-Browser and Open-Meteo API claims
- Radio-Browser: confirmed free, open, unauthenticated, with `all.api.radio-browser.info` as the documented DNS round-robin entry point across community mirrors (`de1`/`nl1`/`at1`/etc.), and a community-recommended courtesy limit of ~2-3 requests/second. Matches the spine's "unauthenticated" claim.
- Open-Meteo: confirmed via `open-meteo.com/en/pricing` — free tier is explicitly "for non-commercial use, rate-limited to 10,000 calls/day," no stated key requirement for the free endpoint. Matches the spine's claim closely, including the exact "10k calls/day" figure.
These two look like they were actually checked against current sources, unlike findings 1 and 3.

### 7. [Confirmed accurate — no finding] AD-8's "correction of record" about `lib.rs`
Read `winradio/src-tauri/src/lib.rs` directly: it declares `pub mod audio; pub mod commands; pub mod store; pub mod tray; pub mod timer;` — all five modules, correctly, exactly as AD-8 claims after "direct re-inspection." This is a rare case in the document of a claim that cites verification and, on independent check, actually holds up.

## Summary Table

| Claim | Verified against | Result |
| --- | --- | --- |
| Tauri 1.5 / CLI 1.6.3 (existing) | repo `Cargo.toml`/`package.json` | Accurate |
| Vue 3.4 / Pinia 2.1 / TS 5.3 / Vite 5.0 / Tailwind 3.4 | repo `package.json` | Accurate (existing pins) |
| rodio 0.19 / tokio 1.38 / reqwest 0.12 (existing) | repo `Cargo.toml` | Accurate (existing pins) |
| `stream-download` 0.5 "currently unused" | grep of `src-tauri/src` | Accurate — no `stream_download` usage found |
| `stream-download` 0.24 exists | crates.io API | Accurate |
| `icy-metadata` 0.6 exists | crates.io API | Accurate |
| `icy-metadata` 0.6 pairs cleanly with `reqwest` 0.12 | icy-metadata's own Cargo.toml.orig | **False** — pulls reqwest 0.13 by default |
| `icy-metadata` 0.6 pairs cleanly with `stream-download` 0.24 / `rodio` 0.19 | icy-metadata's dev-dependencies | **Unverified** — tested against 0.23.0 / 0.21.1 |
| Radio-Browser unauthenticated, `all.api.*` pattern | web search of radio-browser.info ecosystem | Accurate |
| Open-Meteo free tier, 10k calls/day, non-commercial | open-meteo.com/en/pricing | Accurate |
| OSM tile hotlinking "consistent with" tile usage policy | operations.osmfoundation.org/policies/tiles/ | **Incomplete/misleading** — omits mandatory User-Agent + attribution conditions the current design doesn't meet |
| AD-8 lib.rs "verified" correction | repo `src-tauri/src/lib.rs` | Accurate |

## Recommendation
Do not let this spine pass the gate as-is. Findings 1 and 3 are concrete, reproducible, and would surface immediately at implementation time (a dependency-graph surprise and a policy-compliance gap), which is exactly what this lens exists to catch before commitment. Findings 2 and 4 should be resolved as part of fixing 1 and 3. Recommend the architect re-open AD-3 and AD-12 with the actual manifests/policy text in hand, adjust the Stack table accordingly, and re-submit.
