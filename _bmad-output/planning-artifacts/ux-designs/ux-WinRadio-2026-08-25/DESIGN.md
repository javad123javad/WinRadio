---
title: WinRadio Design
status: final
created: 2026-08-25
updated: 2026-08-25
sources:
  - ../../prds/prd-WinRadio-2026-08-25/prd.md
  - ../../briefs/brief-WinRadio-2026-08-25/brief.md
name: WinRadio
description: A flat, dark, tile-dashboard desktop radio player — Metro-era Windows aesthetic, not a modern glassy skin.
colors:
  surface-base: '#101012'
  surface-raised: '#18181b'
  surface-raised-high: '#212126'
  on-surface: '#f5f5f7'
  on-surface-variant: '#9a9aa2'
  outline: '#2a2a2f'
  primary: '#7c5cff'
  on-primary: '#ffffff'
  secondary: '#5aa9f0'
  on-secondary: '#ffffff'
  error: '#e5484d'
  on-error: '#ffffff'
  surface-base-light: '#fafafa'
  surface-raised-light: '#ffffff'
  surface-raised-high-light: '#f0f0f2'
  on-surface-light: '#1b1b1f'
  on-surface-variant-light: '#5c5c66'
  outline-light: '#e2e2e6'
typography:
  fontFamilyBase:
    fontFamily: "'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif"
  display:
    fontFamily: "{typography.fontFamilyBase.fontFamily}"
    fontSize: 32px
    fontWeight: '300'
    lineHeight: '1.2'
  headline:
    fontFamily: "{typography.fontFamilyBase.fontFamily}"
    fontSize: 20px
    fontWeight: '600'
    lineHeight: '1.3'
  body:
    fontFamily: "{typography.fontFamilyBase.fontFamily}"
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.5'
  label-link:
    fontFamily: "{typography.fontFamilyBase.fontFamily}"
    fontSize: 14px
    fontWeight: '500'
    lineHeight: '1.4'
  label-caps:
    fontFamily: "{typography.fontFamilyBase.fontFamily}"
    fontSize: 11px
    fontWeight: '600'
    lineHeight: '1.3'
    letterSpacing: 0.06em
  caption:
    fontFamily: "{typography.fontFamilyBase.fontFamily}"
    fontSize: 12px
    fontWeight: '400'
    lineHeight: '1.4'
rounded:
  sm: 2px
  DEFAULT: 2px
  md: 4px
  full: 9999px
spacing:
  unit: 8px
  '1': 4px
  '2': 8px
  '3': 12px
  '4': 16px
  gutter: 16px
  rail-width: 280px
components:
  nav-icon-button:
    shape: '{rounded.full}'
    size: 48px
    background: transparent
    border: "1px solid {colors.outline}"
    icon-color: '{colors.on-surface}'
    label: '{typography.caption}'
  station-row:
    height: 56px
    background: '{colors.surface-raised}'
    hover-background: '{colors.surface-raised-high}'
    text: '{typography.label-link}'
    text-color: '{colors.secondary}'
    radius: '{rounded.sm}'
    reorder-control-color: '{colors.on-surface-variant}'
    reorder-control-hover-color: '{colors.on-surface}'
  info-tile:
    background: '{colors.surface-raised}'
    radius: '{rounded.sm}'
    padding: '{spacing.3}'
    empty-state-text-color: '{colors.on-surface-variant}'
  transport-bar:
    background: '{colors.surface-raised}'
    accent: '{colors.primary}'
    radius: '{rounded.sm}'
    mute-icon-color: '{colors.on-surface}'
    mute-active-color: '{colors.primary}'
  search-input:
    background: '{colors.surface-raised}'
    border: "1px solid {colors.outline}"
    focus-border: '{colors.primary}'
    text: '{typography.body}'
    radius: '{rounded.sm}'
  settings-modal:
    background: '{colors.surface-raised}'
    overlay: 'rgba(0, 0, 0, 0.6)'
    radius: '{rounded.md}'
    row-divider: '{colors.outline}'
  sleep-timer-control:
    icon-color: '{colors.on-surface}'
    active-badge-background: '{colors.primary}'
    active-badge-text: '{colors.on-primary}'
    radius: '{rounded.full}'
  filter-chip:
    background: '{colors.surface-raised-high}'
    text: '{typography.caption}'
    text-color: '{colors.on-surface-variant}'
    value-color: '{colors.on-surface}'
    radius: '{rounded.full}'
---

## Brand & Style

WinRadio is deliberately unfashionable. It borrows its aesthetic from the Windows 8 Metro era — flat, dark, tile-based, no gradients, no glassmorphism, no drop shadows pretending to be depth. That's not nostalgia for its own sake: it's a good fit for a background-resident dashboard app that needs to be scannable at a glance and stay out of the way otherwise. The posture is **utilitarian dashboard**, not consumer-app polish — this app has exactly one user and doesn't need to sell itself.

## Colors

Dark is the primary, default surface — `{colors.surface-base}` is near-black, not pure black, so tiles (`{colors.surface-raised}`) read as distinct layers without needing a shadow to prove it. `{colors.surface-raised-high}` is reserved for hover/active tile states — one step lighter, never a border or glow.

Two accent colors carry different meanings, matching the reference image's own split: `{colors.primary}` (violet) marks *interactive/active* elements — the volume fill, focus rings, the active nav icon. `{colors.secondary}` (blue) is reserved for *station identity* — station-list link text specifically, nothing else. If everything used the same accent, favorites would stop reading as distinct from controls.

`{colors.error}` is used sparingly and only for things that are genuinely broken (a station that won't play at all) — never for an Info Tile that's simply degraded (see Do's and Don'ts). Light-mode tokens (`*-light` suffix) invert the surface ramp but keep both accents identical across themes — the app's identity shouldn't shift when the user flips the theme toggle.

`[ASSUMPTION]` Exact hex values are a starting point inferred from the reference screenshot, not confirmed pixel-picks — cheap to adjust once real screens exist.

## Typography

One family, `{typography.fontFamilyBase}` — Segoe UI Variable falling back to system Segoe UI, then generic `system-ui`. This is a Windows-only app; there's no reason to ship a custom webfont when the OS default already matches the reference image's look and loads with zero cost.

`{typography.display}` is reserved for the one moment it matters: the current station name on the Now-Playing Dashboard. Directly beneath it, the live track/title metadata (when the stream provides one) uses `{typography.body}` — a visibly smaller, calmer line than the station name, since it updates on its own and shouldn't compete with the thing the user actually chose. `{typography.headline}` marks section headers ("Favorites", search result counts). `{typography.label-link}` is exclusively the station-row text style — pairs with `{colors.secondary}`. `{typography.label-caps}` handles dense technical readouts, split by which Info Tile they belong to: the Location Tile's city/coordinates, and separately the Stream Info Tile's codec/bitrate/country/resolved-IP — same type style, distinct content, never blended into one list.

## Layout & Spacing

An 8px base unit (`{spacing.unit}`) throughout — every gap, padding, and tile dimension is a multiple of it, which is what makes a tile grid feel intentional instead of ad hoc. `{spacing.gutter}` (16px) separates tiles from each other and from the rail.

The Station List (the left-hand rail — see `EXPERIENCE.md.Foundation` for the shorthand) has a fixed width (`{spacing.rail-width}`, 280px) — it does not flex with window width; extra horizontal space goes to the Now-Playing Dashboard and Info Tile grid instead, since that's where the content variety actually lives.

## Elevation & Depth

No shadows. Depth is entirely tonal — `{colors.surface-base}` → `{colors.surface-raised}` → `{colors.surface-raised-high}` is the whole depth vocabulary. This is a deliberate constraint, not an oversight: shadows on a dark, flat-tile dashboard read as dated skeuomorphism, and the reference image itself uses pure tonal separation with hard edges.

## Shapes

Two tracks, used consistently and never mixed:
- **Tiles and rows** (Info Tiles, station rows, the transport bar): `{rounded.sm}` — barely-there, almost square. This is the Metro flatness.
- **Icon buttons and the play control**: `{rounded.full}` — fully circular, matching the reference image's nav icons exactly. Circles read as "controls you press"; squares read as "content you scan." Don't blur that distinction by rounding tiles more or squaring off buttons.

## Components

- **Nav icon-button** (`{components.nav-icon-button}`): circle, outlined not filled, icon + small caption label beneath (Favorites, Filter, Search, Settings). `[NOTE FOR UX]` The reference image's top-left "GER Home" / "ENG Home" buttons are that specific website's language-switcher links, not a generic navigation concept — WinRadio has no "Home" icon; Favorites already serves as the default/landing view. Active state fills with `{colors.primary}` at low opacity, not a solid fill — stays legible against both themes.
- **Station row** (`{components.station-row}`): play icon + station name in `{colors.secondary}`, left-aligned, full rail width. Hover lightens to `{colors.surface-raised-high}`; no border, no shadow. In the Favorites list only, a compact up/down reorder control (`reorder-control-color`) sits at the row's left edge, always visible (not hover-reveal, per the no-hover-only-affordances rule) — visually quiet until hovered/focused, then shifts to `reorder-control-hover-color`.
- **Info Tile** (`{components.info-tile}`): the shared shell for all three tile types (Location, Weather, Stream Info) — same background, radius, and padding regardless of content, so the dashboard reads as one grid, not three ad hoc widgets. Empty/degraded state (per PRD NFR-3) keeps the same shell and shows short muted text in `{colors.on-surface-variant}` — never a red error state, never an empty void.
- **Transport bar** (`{components.transport-bar}`): play/pause, scrub position (display-only — internet radio has no seek), a volume slider filled in `{colors.primary}`, and a separate mute icon-toggle (`mute-icon-color`, filling `mute-active-color` when engaged) immediately left of the slider. Mute is its own state, not "drag to zero" — muting and later unmuting restores the prior volume level exactly. Sits directly beneath the Now-Playing artwork, always visible whenever a station is loaded.
- **Search input** (`{components.search-input}`): single-line field, appears under the Search nav icon. Border shifts to `{colors.primary}` on focus — the only field in the app, so its focus state doubles as an unambiguous "you're now searching" signal.
- **Settings modal** (`{components.settings-modal}`): overlay dialog, one level deep only. Rows separated by `row-divider`: tray behavior, Sleep Timer defaults, and the dark/light theme toggle live here as simple label + control rows — this is the theme toggle's only location; it is deliberately not a nav-bar icon, keeping the nav bar reserved for content-filtering rather than app configuration.
- **Sleep Timer control** (`{components.sleep-timer-control}`): a small icon button on the transport bar (moon/clock icon) opens a short duration picker. Once armed, the icon itself carries an `active-badge` (small filled dot in `{colors.primary}`) — the *only* dashboard indication a timer is running, deliberately understated rather than a persistent countdown, matching the "utilitarian, not fussy" brand posture. Clicking the badged icon again offers cancel.
- **Filter chip** (`{components.filter-chip}`): pill-shaped, appears beneath the search input once a genre/country/language filter is applied — label in `{colors.on-surface-variant}`, the selected value in `{colors.on-surface}`, with an inline ✕ to remove it. Rounded `{rounded.full}` like the nav icons, not `{rounded.sm}` like tiles — a chip is closer kin to a control than to content.

## Do's and Don'ts

- **Do** keep the Info Tile grid to exactly three tiles (Location, Weather, Stream Info). Do not add a fourth without revisiting SM-C1 in the PRD — tile-count creep is the exact scope drift the PRD's counter-metric exists to prevent.
- **Do** let a degraded Info Tile look calm (muted text, same shell) — never let a weather-API outage look like the app itself is broken.
- **Don't** add drop shadows, blur/glass effects, or gradients anywhere. If a surface needs to look "elevated," lighten it — don't shadow it.
- **Don't** round tiles to match the circular nav buttons, and don't square off the nav buttons to match the tiles. The two shapes carry meaning.
- **Don't** introduce a second accent color for anything else. Violet is "interactive," blue is "station identity" — full stop.
