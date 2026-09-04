# Epic 2 Context: Now-Playing Dashboard & Info Tiles

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

While a station plays, the dashboard shows live track metadata plus three at-a-glance context tiles — station location (with a small map), local weather, and technical stream details (codec/bitrate/country/resolved IP). Each tile is enhancement-only: none of them may block, slow, or destabilize core playback, and each degrades quietly and independently if its own data source is unreachable. This is what makes the dashboard feel alive without reintroducing the fragility (flaky external dependencies, half-wired features) that stalled earlier attempts at this app.

## Stories

- Story 2.1: Now-Playing Metadata Display
- Story 2.2: Location Tile
- Story 2.3: Weather Tile
- Story 2.4: Stream Info Tile

## Requirements & Constraints

- Now-playing metadata (station name + live ICY track/title) must display when the stream provides it; absent ICY data is normal, not an error — the subtext line is simply blank.
- Location Tile: shows city/country and a single map image centered on the station, when the station has location data. No location data, or a failed fetch, → a quiet "Location unknown" placeholder, never an error state.
- Weather Tile: shows current conditions and a short forecast for the station's location, when coordinates are available. No location data or a failed fetch → "Weather unavailable," shown independently of the other tiles.
- Stream Info Tile: shows codec, bitrate, country, and the DNS-resolved server IP. Codec/bitrate/country must render immediately with no loading flash (they're already-known data). A failed/slow IP lookup only affects the IP field ("unavailable") — it must never be conflated with a playback failure.
- Hard reliability constraint (NFR-3): core playback (search, play, favorites, tray, sleep timer) never depends on Weather or Location services being reachable. An outage in the Directory, weather, or map service degrades only the affected tile.
- Tile count is fixed at exactly three (Location, Weather, Stream Info) — do not add a fourth; this is a deliberate counter-metric (SM-C1) against scope drift.
- Privacy: the only network calls this epic introduces are the Weather query (station coordinates only) and the OSM tile fetch — no other data leaves the machine.

## Technical Decisions

- **Event-pushed, never polled (AD-4/AD-5).** Rust emits one event per Info Tile category, each fired exactly once per attempt: `location-updated`, `weather-updated`, `stream-info-updated`. Now-playing metadata is a separate concern, carried by the existing `metadata-updated` event (AD-4), not by AD-5's tiles.
- **Shared envelope.** All three Info Tile events use one payload shape: `{ ok: boolean, data: T | null, reason: string | null }`. This is what the shared `InfoTile.vue` shell renders against — success sets `data`, failure sets `reason` and leaves `data` null. Never silence, never a bespoke per-tile shape.
- **No redundant fetching.** Stream Info's codec/bitrate/country and Location's `geoLat`/`geoLong` are read synchronously off the already-cached `Station` object — no network round-trip for data already in hand. Only the genuinely live parts are fetched: the DNS-resolved IP (Stream Info) and the OSM tile image (Location). Weather always requires a live Open-Meteo call.
- **Ephemeral only.** None of this Info Tile data is written to local storage (`store.rs`) — it exists only for the current playback session, reconstructed fresh each time a station starts playing.
- **Scope isolation from playback errors.** A failed IP lookup or tile/weather fetch must never fire `playback-error` — that event is reserved for audio-output failures only, even though the IP lookup lives in the same `player.rs` module.
- **Location Tile networking (AD-12).** The OSM tile is fetched via a Tauri command using `reqwest` with a custom `User-Agent: WinRadio/0.1 (personal desktop app)` header — never a direct `<img>` hotlink from Vue (violates both OSM's usage policy and the command-boundary rule). No map library, no API key; a single static XYZ raster tile at a fixed zoom.
- **Serde casing (AD-6).** Every event payload struct (including the shared envelope and its `data` variants) uses `#[serde(rename_all = "camelCase")]`.
- **Relevant modules:** `audio/player.rs` (metadata + IP lookup), `directory.rs` (location coords + OSM tile fetch, Stream Info's static fields), `weather.rs` (new — Open-Meteo client), `NowPlayingDashboard.vue`, `InfoTile.vue` + its three variants (`LocationTile.vue`, `WeatherTile.vue`, `StreamInfoTile.vue`).
- **External API:** Open-Meteo (`open-meteo.com`), unauthenticated, non-commercial free tier (10k calls/day) — no API key needed.

## UX & Interaction Patterns

- Info Tile shell: one shared component, three content variants, identical background/radius/padding so the dashboard reads as one grid rather than three ad hoc widgets.
- Degraded state styling: quiet muted text (`on-surface-variant` token) in the same shell — never red/error styling and never a blank void. Error red is reserved exclusively for genuine playback failure ("Couldn't play this station"), not for a degraded tile.
- Typography: station name uses the `display` type role; the live track/title subtext beneath it uses the smaller `body` role so it doesn't visually compete with the station name. Location's city/coordinates and Stream Info's codec/bitrate/country/IP both use the dense `label-caps` style, kept as visually distinct groups even though they share a type style.
- Location Tile renders an "© OpenStreetMap contributors" attribution caption beneath the map image — required alongside the tile fetch itself.
- Responsive rule: below a minimum window width, the Info Tile grid drops from 3-across to 2-across (Location + Weather on one row, Stream Info wraps beneath) before anything is hidden outright. The Station List rail never collapses.
- Accessibility: Info Tiles are informational, not currently interactive, but if any tile control becomes actionable later it must be reachable via `Tab` with a visible focus ring; no tile state may be conveyed by color alone.

## Cross-Story Dependencies

- All three Info Tiles and the metadata display only populate once a station is playing — they depend on Epic 1's playback pipeline (`stream-download`+`icy-metadata`+`rodio`, AD-3) already being wired up and emitting its base playback events.
- Stream Info and Location tiles both read their static fields off the cached `Station` object, so they depend on Epic 1/Architecture's `Station` data shape already carrying `genre`/`country`/`language`/`codec`/`bitrate`/`geoLat`/`geoLong` (AD-6's data-shape change).
- The Info Tile grid area is reserved (empty) by Story 1.1's base shell layout; this epic fills that reserved space rather than building new layout scaffolding.
