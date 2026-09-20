use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use icy_metadata::{IcyHeaders, IcyMetadataReader};
use parking_lot::Mutex;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::sync::mpsc as std_mpsc;
use serde::{Deserialize, Serialize};
use stream_download::http::HttpStream;
use stream_download::source::DecodeError;
use stream_download::storage::bounded::BoundedStorageProvider;
use stream_download::storage::memory::MemoryStorageProvider;
use stream_download::{Settings as DownloadSettings, StreamDownload};
use tauri::{AppHandle, Manager};

use crate::commands::Station;
use crate::directory;
use crate::stream_info;
use crate::weather;

/// Backoff schedule for reconnect attempts after a stream fails or drops:
/// an immediate retry, then 2s/5s/10s delays before giving up (~20s budget
/// once attempt/connect latency is included). Exposed for testing.
pub const RETRY_DELAYS: [Duration; 4] = [
    Duration::from_secs(0),
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(10),
];

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub artwork_url: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReconnectingPayload {
    attempt: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackErrorPayload {
    reason: String,
}

/// Shared `{ok, data, reason}` envelope (AD-5) for all three Info Tile
/// events (`location-updated`/`weather-updated`/`stream-info-updated`).
/// Success sets `data` and leaves `reason` null; failure is the reverse,
/// never both/neither. Generic over the tile-specific payload type so the
/// three events no longer need three near-identical struct definitions
/// (cleanup: the three were previously byte-for-byte identical in shape,
/// differing only in `data`'s type).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TileUpdatePayload<T: Clone + Serialize> {
    ok: bool,
    data: Option<T>,
    reason: Option<String>,
}

impl<T: Clone + Serialize> TileUpdatePayload<T> {
    /// The `ok: false` shape, constructed at every "unavailable" call site —
    /// cleanup: previously repeated inline (with a turbofish for the `None`
    /// case's otherwise-unconstrained `T`) at each of the three tiles'
    /// failure branches.
    fn unavailable(reason: &'static str) -> Self {
        Self {
            ok: false,
            data: None,
            reason: Some(reason.to_string()),
        }
    }
}

/// Frozen copy (Boundaries & Constraints): the exact same placeholder text
/// covers both "no coordinates" and "tile fetch failed" — the latter is
/// resolved on the frontend, not here, but this is the one-and-only string
/// for the "no coordinates" branch so the two can never drift apart.
const LOCATION_UNKNOWN: &str = "Location unknown";

/// One-and-only copy of the "no coordinates"/"fetch failed" placeholder
/// string for the weather event, mirroring `LOCATION_UNKNOWN` above.
const WEATHER_UNAVAILABLE: &str = "Weather unavailable";

/// One-and-only copy of the DNS-failure placeholder string for the stream
/// info event, mirroring `LOCATION_UNKNOWN`/`WEATHER_UNAVAILABLE` above.
const STREAM_INFO_UNAVAILABLE: &str = "unavailable";

pub struct RadioPlayer {
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Option<Arc<Sink>>>>,
    current_station: Arc<Mutex<Option<Station>>>,
    volume: Arc<Mutex<f32>>,
    metadata: Arc<Mutex<Option<Metadata>>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    generation: AtomicU64,
    // spec-2-2 code review finding: tracks which station's `location-updated`
    // was last emitted, so a drop/reconnect cycle to the *same* station
    // (network hiccup, not a genuine station switch) doesn't re-emit and
    // trigger a needless fresh OSM tile fetch + tile-flicker on the
    // frontend. A reconnect to a *different* station (different id) always
    // still emits.
    last_location_station_id: Mutex<Option<String>>,
    // spec-2-3: parallel dedup tracking for `weather-updated`, independent
    // of Location's — the two event streams must never share a field, or a
    // reconnect that legitimately re-emits one would spuriously suppress
    // the other.
    last_weather_station_id: Mutex<Option<String>>,
    // spec-2-4: parallel dedup tracking for `stream-info-updated`,
    // independent of Location's and Weather's, same contract.
    last_stream_info_station_id: Mutex<Option<String>>,
}

impl RadioPlayer {
    /// `initial_volume` seeds playback volume from persisted `Settings` at
    /// startup (I/O matrix: "last volume restored" on relaunch).
    pub fn new(initial_volume: f32) -> Self {
        // `rodio::OutputStream` is `!Send`/`!Sync` (it wraps a platform audio
        // handle), but `RadioPlayer` is shared across threads via
        // `Arc<RadioPlayer>` and Tauri's async command state, which requires
        // `Send + Sync`. So the stream is created and kept alive forever on a
        // dedicated, parked OS thread; only the `Send + Sync` `OutputStreamHandle`
        // (used to build `Sink`s) crosses back out.
        let (tx, rx) = std_mpsc::channel();
        std::thread::spawn(move || match OutputStream::try_default() {
            Ok((_stream, handle)) => {
                let _ = tx.send(Some(handle));
                // Park forever, keeping `_stream` alive for the process lifetime.
                loop {
                    std::thread::park();
                }
            }
            Err(_) => {
                let _ = tx.send(None);
            }
        });

        let stream_handle = rx
            .recv()
            .ok()
            .flatten()
            .expect("Failed to create audio output stream");

        Self {
            stream_handle,
            sink: Arc::new(Mutex::new(None)),
            current_station: Arc::new(Mutex::new(None)),
            volume: Arc::new(Mutex::new(initial_volume.clamp(0.0, 1.0))),
            metadata: Arc::new(Mutex::new(None)),
            app_handle: Arc::new(Mutex::new(None)),
            generation: AtomicU64::new(0),
            last_location_station_id: Mutex::new(None),
            last_weather_station_id: Mutex::new(None),
            last_stream_info_station_id: Mutex::new(None),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.lock() = Some(handle);
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) {
        if let Some(handle) = self.app_handle.lock().as_ref() {
            let _ = handle.emit_all(event, payload);
        }
    }

    fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    fn is_current_generation(&self, gen: u64) -> bool {
        self.current_generation() == gen
    }

    /// Shared shape for Weather's and Stream Info's tile-update emission:
    /// spawn `fetch` off the playback-critical path, re-check
    /// `is_current_generation` right before emitting so a station switch
    /// mid-fetch can't emit a stale tile's data, then emit the `{ok, data,
    /// reason}` envelope for either outcome. Cleanup: these two call sites
    /// were previously near-identical `tokio::spawn` blocks copied one from
    /// the other; Location doesn't use this helper since its coordinates are
    /// read synchronously off the cached `Station`, with nothing to spawn.
    fn spawn_tile_fetch_and_emit<T, F>(self: &Arc<Self>, gen: u64, event: &'static str, unavailable_reason: &'static str, fetch: F)
    where
        T: Clone + Serialize + Send + 'static,
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        let player = self.clone();
        tokio::spawn(async move {
            let result = fetch.await;

            if !player.is_current_generation(gen) {
                return;
            }

            match result {
                Ok(data) => player.emit(
                    event,
                    TileUpdatePayload {
                        ok: true,
                        data: Some(data),
                        reason: None,
                    },
                ),
                Err(_) => player.emit(event, TileUpdatePayload::<T>::unavailable(unavailable_reason)),
            }
        });
    }

    pub fn is_playing(&self) -> bool {
        self.sink.lock().is_some()
    }

    pub fn current_station(&self) -> Option<Station> {
        self.current_station.lock().clone()
    }

    pub fn get_metadata(&self) -> Option<Metadata> {
        self.metadata.lock().clone()
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock()
    }

    /// Toggle play/pause using whatever station is currently loaded. Used by
    /// the system tray and the media-key global shortcut, both of which only
    /// have a `&Arc<RadioPlayer>` and need a synchronous entry point.
    pub fn toggle_play_pause(self: &Arc<Self>) {
        let player = self.clone();
        tauri::async_runtime::spawn(async move {
            if player.is_playing() {
                let _ = player.stop().await;
            } else if let Some(station) = player.current_station() {
                let _ = player.clone().play(station).await;
            }
        });
    }

    /// Starts playback of `station`, tearing down any currently active
    /// stream first (at most one active stream at a time). Returns as soon
    /// as the attempt has been kicked off; playback lifecycle is reported
    /// entirely through `play`/`reconnecting`/`playback-error` events, never
    /// through this method's return value.
    pub async fn play(self: Arc<Self>, station: Station) -> Result<(), String> {
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;

        if let Some(sink) = self.sink.lock().take() {
            sink.stop();
        }

        *self.current_station.lock() = Some(station.clone());
        *self.metadata.lock() = None;

        tokio::spawn(Self::run_playback(self, station, gen));

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(sink) = self.sink.lock().take() {
            sink.stop();
        }
        self.emit("stop", ());
        Ok(())
    }

    pub async fn set_volume(&self, volume: f32) -> Result<(), String> {
        let vol = volume.clamp(0.0, 1.0);
        *self.volume.lock() = vol;
        if let Some(sink) = self.sink.lock().as_ref() {
            sink.set_volume(vol);
        }
        Ok(())
    }

    fn handle_metadata(&self, gen: u64, meta: icy_metadata::IcyMetadata) {
        if !self.is_current_generation(gen) {
            return;
        }

        let raw_title = meta.stream_title().unwrap_or("").trim().to_string();
        if raw_title.is_empty() {
            return;
        }

        let (artist, title) = match raw_title.split_once(" - ") {
            Some((a, t)) => (a.trim().to_string(), t.trim().to_string()),
            None => (String::new(), raw_title.clone()),
        };

        let metadata = Metadata {
            title,
            artist,
            album: String::new(),
            artwork_url: String::new(),
        };

        *self.metadata.lock() = Some(metadata.clone());
        self.emit("metadata-updated", metadata);
    }

    /// Drives the connect-with-retry loop for one playback "session"
    /// (identified by `gen`). Re-enters the retry loop whenever the sink
    /// ends unexpectedly (a stream drop) while this generation is still the
    /// active one; exits quietly once superseded by a newer `play()`/`stop()`.
    async fn run_playback(self: Arc<Self>, station: Station, gen: u64) {
        // A successful `try_connect_and_play` skips `RETRY_DELAYS` entirely
        // (only a *failed* attempt backs off), so a station that connects but
        // whose sink then ends almost immediately, over and over, would
        // otherwise loop forever with no backoff and never reach a terminal
        // `playback-error` (finding #3). Track a streak of short-lived
        // episodes across drop/reconnect cycles and give up once it's long
        // enough that "reconnecting" is clearly not going to help.
        const SHORT_EPISODE_THRESHOLD: Duration = Duration::from_secs(2);
        const MAX_CONSECUTIVE_SHORT_EPISODES: u32 = 3;
        let mut consecutive_short_episodes: u32 = 0;

        loop {
            if !self.is_current_generation(gen) {
                return;
            }

            let player = self.clone();
            let station_for_attempt = station.clone();
            let result = retry_with_backoff(
                &RETRY_DELAYS,
                || {
                    let player = player.clone();
                    let station = station_for_attempt.clone();
                    async move { player.try_connect_and_play(station, gen).await }
                },
                |attempt_index| {
                    if self.is_current_generation(gen) {
                        self.emit(
                            "reconnecting",
                            ReconnectingPayload {
                                attempt: attempt_index as u32 + 1,
                            },
                        );
                    }
                },
            )
            .await;

            match result {
                Ok(join_handle) => {
                    if !self.is_current_generation(gen) {
                        return;
                    }
                    self.emit("play", station.clone());

                    // spec-2-2: fires once per play attempt, right after
                    // `play` — mirrors `metadata`'s reset lifecycle (idle on
                    // play/stop/playback-error). Coordinates are read
                    // synchronously off the already-cached `station` (AD-5
                    // "no redundant fetching"); never a network round-trip
                    // here, and never blocks/delays playback.
                    //
                    // Deduped against `last_location_station_id` (code review
                    // finding): a drop/reconnect cycle back to the *same*
                    // station after a network hiccup must not re-emit and
                    // trigger a needless fresh OSM tile fetch/flicker. A
                    // reconnect to a genuinely *different* station (different
                    // id) always still emits.
                    {
                        let mut last_location_station_id = self.last_location_station_id.lock();
                        if should_emit_tile_update(&mut last_location_station_id, &station.id) {
                            let location_data = directory::location_info_for(&station);
                            let location_ok = location_data.is_some();
                            self.emit(
                                "location-updated",
                                TileUpdatePayload {
                                    ok: location_ok,
                                    data: location_data,
                                    reason: if location_ok {
                                        None
                                    } else {
                                        Some(LOCATION_UNKNOWN.to_string())
                                    },
                                },
                            );
                        }
                    }

                    // spec-2-3: fires once per play attempt, right after
                    // Location. Deduped against `last_weather_station_id`
                    // (parallel to, but independent of, Location's dedup) —
                    // same drop/reconnect-to-same-station suppression, same
                    // always-emit-on-genuine-switch behavior.
                    //
                    // The live Open-Meteo call happens off the
                    // playback-critical path (NFR-3): a no-coordinates
                    // station emits immediately (no network call), but a
                    // coordinates-having station's fetch is `tokio::spawn`'d
                    // rather than `.await`'d inline, so a slow/unreachable
                    // weather API never delays reaching the retry loop's
                    // post-play logic below. The spawned task re-checks
                    // `is_current_generation` right before emitting, so a
                    // station switch mid-fetch can't emit a stale tile's
                    // weather.
                    {
                        let mut last_weather_station_id = self.last_weather_station_id.lock();
                        if should_emit_tile_update(&mut last_weather_station_id, &station.id) {
                            match directory::location_info_for(&station) {
                                Some(location) => {
                                    self.spawn_tile_fetch_and_emit(gen, "weather-updated", WEATHER_UNAVAILABLE, async move {
                                        let client = weather::build_client()?;
                                        weather::fetch_weather(&client, location.geo_lat, location.geo_long).await
                                    });
                                }
                                None => {
                                    self.emit(
                                        "weather-updated",
                                        TileUpdatePayload::<weather::WeatherInfo>::unavailable(WEATHER_UNAVAILABLE),
                                    );
                                }
                            }
                        }
                    }

                    // spec-2-4: fires once per play attempt, right after
                    // Weather. Deduped against `last_stream_info_station_id`
                    // (parallel to, but independent of, Location's and
                    // Weather's dedup) — same drop/reconnect-to-same-station
                    // suppression, same always-emit-on-genuine-switch
                    // behavior.
                    //
                    // Codec/bitrate/country never ride this event — they're
                    // read synchronously off the already-cached `station` on
                    // the frontend (AC1), no round-trip needed. The IP is the
                    // tile's only genuinely fetched field: the host/port
                    // parse happens synchronously (a malformed URL/no host
                    // emits `ok:false` immediately, no spawn needed), but the
                    // actual DNS lookup is `tokio::spawn`'d off the
                    // playback-critical path exactly like Weather's fetch,
                    // re-checking `is_current_generation` right before
                    // emitting so a station switch mid-lookup can't emit a
                    // stale station's IP.
                    {
                        let mut last_stream_info_station_id = self.last_stream_info_station_id.lock();
                        if should_emit_tile_update(&mut last_stream_info_station_id, &station.id) {
                            match stream_info::extract_host_and_port(&station.url) {
                                Some(_) => {
                                    let url = station.url.clone();
                                    self.spawn_tile_fetch_and_emit(gen, "stream-info-updated", STREAM_INFO_UNAVAILABLE, async move {
                                        stream_info::resolve_ip(&url).await
                                    });
                                }
                                None => {
                                    self.emit(
                                        "stream-info-updated",
                                        TileUpdatePayload::<String>::unavailable(STREAM_INFO_UNAVAILABLE),
                                    );
                                }
                            }
                        }
                    }

                    let episode_started = std::time::Instant::now();
                    let _ = join_handle.await;

                    if !self.is_current_generation(gen) {
                        return;
                    }

                    if episode_started.elapsed() < SHORT_EPISODE_THRESHOLD {
                        consecutive_short_episodes += 1;
                        if consecutive_short_episodes >= MAX_CONSECUTIVE_SHORT_EPISODES {
                            // The sink already ended on its own (that's how we
                            // got here), but it's still sitting in `self.sink`
                            // — clear it so `is_playing()` reports `false`
                            // after this terminal give-up. Otherwise
                            // `toggle_play_pause()` sees a stale "playing"
                            // state and calls `stop()` (a no-op) instead of
                            // `play()` to retry.
                            self.sink.lock().take();
                            self.emit(
                                "playback-error",
                                PlaybackErrorPayload {
                                    reason: "Couldn't play this station: connection keeps \
                                             dropping immediately after connecting"
                                        .to_string(),
                                },
                            );
                            return;
                        }
                    } else {
                        // A real stretch of sustained playback happened —
                        // this wasn't a rapid-drop loop, so reset the streak.
                        consecutive_short_episodes = 0;
                    }

                    // The sink ended while this generation is still current:
                    // treat it as an unexpected drop and reconnect.
                    continue;
                }
                Err(reason) => {
                    if self.is_current_generation(gen) {
                        self.emit(
                            "playback-error",
                            PlaybackErrorPayload {
                                reason: format!("Couldn't play this station: {reason}"),
                            },
                        );
                    }
                    return;
                }
            }
        }
    }

    /// One connection attempt: builds the HTTP + ICY + decoder pipeline and
    /// starts playback on a blocking thread. Resolves once the sink has
    /// actually started (or the attempt has definitively failed), not once
    /// playback has finished.
    async fn try_connect_and_play(
        self: Arc<Self>,
        station: Station,
        gen: u64,
    ) -> Result<tokio::task::JoinHandle<()>, String> {
        if !self.is_current_generation(gen) {
            return Err("superseded".to_string());
        }

        let mut headers = http::HeaderMap::new();
        icy_metadata::add_icy_metadata_header(&mut headers);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(Duration::from_secs(8))
            // Per-read idle timeout, NOT a total-request deadline: reqwest's
            // `.timeout()` would cut off every stream after the deadline even
            // while healthy (it "applies from when the request starts
            // connecting until the response body has finished" — fatal for a
            // multi-hour radio stream). `.read_timeout()` resets on every
            // successful read, so it only fires when the server accepts the
            // connection and then genuinely stalls (finding #2).
            .read_timeout(Duration::from_secs(30))
            .user_agent("WinRadio/0.1")
            .build()
            .map_err(|e| e.to_string())?;

        let url = station
            .url
            .parse()
            .map_err(|e| format!("Invalid station URL: {e}"))?;

        let http_stream = HttpStream::new(client, url)
            .await
            .map_err(|e| e.to_string())?;

        let icy_headers = IcyHeaders::parse_from_headers(http_stream.headers());
        let metadata_interval = icy_headers.metadata_interval();
        const STORAGE_CAPACITY_BYTES: u64 = 4 * 1024 * 1024;
        // Clamp well under the bounded storage's capacity: a station reporting
        // an inflated/bogus `icy-br` header must never be able to ask for a
        // prefetch larger than what the buffer can actually hold (finding #8).
        let prefetch_bytes = icy_headers
            .bitrate()
            .map(|kbps| (kbps as u64 / 8) * 1024 * 3)
            .unwrap_or(64 * 1024)
            .min(3 * 1024 * 1024);

        let reader = match StreamDownload::from_stream(
            http_stream,
            BoundedStorageProvider::new(
                MemoryStorageProvider,
                NonZeroUsize::new(STORAGE_CAPACITY_BYTES as usize).unwrap(),
            ),
            DownloadSettings::default().prefetch_bytes(prefetch_bytes),
        )
        .await
        {
            Ok(reader) => reader,
            Err(e) => return Err(e.decode_error().await.to_string()),
        };

        if !self.is_current_generation(gen) {
            return Err("superseded".to_string());
        }

        let player_for_meta = self.clone();
        let icy_reader = IcyMetadataReader::new(reader, metadata_interval, move |meta| {
            if let Ok(meta) = meta {
                player_for_meta.handle_metadata(gen, meta);
            }
        });

        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<Result<(), String>>();
        let stream_handle = self.stream_handle.clone();
        let sink_slot = self.sink.clone();
        let volume = *self.volume.lock();
        // Cloned so the blocking closure can re-check the generation right
        // before publishing the sink — closing the race window between the
        // check above and the moment the (slow-to-build) decoder/sink is
        // finally ready (finding #1).
        let player_for_install = self.clone();

        let join_handle = tokio::task::spawn_blocking(move || {
            let decoder = match rodio::Decoder::new(icy_reader) {
                Ok(d) => d,
                Err(e) => {
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => s,
                Err(e) => {
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
            };
            sink.set_volume(volume);
            sink.append(decoder);

            let sink = Arc::new(sink);
            if let Err(stale_sink) = install_if_current(
                &player_for_install.generation,
                gen,
                &sink_slot,
                sink.clone(),
            ) {
                // A newer play()/stop() superseded us while the decoder/sink
                // was being built. Never publish a stale sink into the shared
                // slot — tear it down instead, so at most one stream is ever
                // active (spec invariant) and no later stop()/set_volume()
                // call can accidentally act on the wrong sink.
                stale_sink.stop();
                let _ = ready_tx.send(Err("superseded".to_string()));
                return;
            }

            let _ = ready_tx.send(Ok(()));

            sink.sleep_until_end();
        });

        match ready_rx.await {
            Ok(Ok(())) => Ok(join_handle),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Playback thread failed to start".to_string()),
        }
    }
}

/// Installs `value` into `slot` only if `generation` still equals `gen` at
/// the moment of installation. Closes the race between an earlier
/// "am I still current?" check and the moment a slow-to-construct resource
/// (e.g. a `Sink`) actually finishes and is ready to be shared: without this,
/// a superseded attempt could still publish itself into `slot` right after
/// the check that was meant to stop it (finding #1). On success, returns
/// `Ok(())` with `value` now in `slot`; if superseded, hands `value` back via
/// `Err` so the caller can tear it down instead of leaking a live resource
/// nothing will ever stop.
fn install_if_current<T>(
    generation: &AtomicU64,
    gen: u64,
    slot: &Mutex<Option<T>>,
    value: T,
) -> Result<(), T> {
    if generation.load(Ordering::SeqCst) != gen {
        return Err(value);
    }
    *slot.lock() = Some(value);
    Ok(())
}

/// Pure decision for an Info Tile event's dedup (code review finding #2,
/// generalized — Location/Weather/Stream Info previously each had a
/// byte-for-byte identical copy of this function under its own name).
/// Given what was last emitted for a tile (`last_emitted_for`, the guarded
/// content of one of `RadioPlayer`'s three `last_*_station_id` fields — each
/// tile keeps its own field/state, deliberately never shared) and the
/// station about to (re)connect, returns whether to emit that tile's event
/// again, updating `last_emitted_for` when it does. A drop/reconnect back to
/// the *same* station (same id) must not re-emit — that's a network hiccup,
/// not a genuine station switch, and re-emitting would trigger a needless
/// fresh fetch and flicker the tile. A genuinely *different* station
/// (different id, including the very first play of a session) must always
/// still emit.
///
/// Split out as a standalone primitive, mirroring `install_if_current`
/// above, so it's unit-testable without constructing a `RadioPlayer` itself
/// — which requires a live audio output device (`OutputStream::try_default`)
/// unavailable in this test harness/CI.
fn should_emit_tile_update(last_emitted_for: &mut Option<String>, station_id: &str) -> bool {
    if last_emitted_for.as_deref() == Some(station_id) {
        return false;
    }
    *last_emitted_for = Some(station_id.to_string());
    true
}

/// Generic retry helper: tries `attempt` once; on failure, calls
/// `on_retry(index)` and retries after each of `delays` in turn (a zero
/// delay is not slept on). Returns the first success, or the last error once
/// every delay has been exhausted.
async fn retry_with_backoff<T, F, Fut>(
    delays: &[Duration],
    mut attempt: F,
    mut on_retry: impl FnMut(usize),
) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    match attempt().await {
        Ok(v) => return Ok(v),
        Err(mut last_err) => {
            for (index, delay) in delays.iter().enumerate() {
                on_retry(index);
                if !delay.is_zero() {
                    tokio::time::sleep(*delay).await;
                }
                match attempt().await {
                    Ok(v) => return Ok(v),
                    Err(e) => last_err = e,
                }
            }
            Err(last_err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn retry_delays_match_spec_backoff_schedule() {
        // immediate, 2s, 5s, 10s, then give up (~20s budget) — see
        // spec-1-1-reliable-single-station-playback-foundation-rescue.md
        assert_eq!(
            RETRY_DELAYS,
            [
                Duration::from_secs(0),
                Duration::from_secs(2),
                Duration::from_secs(5),
                Duration::from_secs(10),
            ]
        );
    }

    // Regression coverage for the stale-sink install race (code review
    // finding #1): a full HTTP-backed integration test isn't feasible in
    // this harness (no test radio server, no audio device in CI), so this
    // exercises the exact generation-guard primitive `try_connect_and_play`
    // uses at the real install site, in isolation from Sink/HttpStream.

    #[test]
    fn install_if_current_installs_when_generation_still_matches() {
        let generation = AtomicU64::new(5);
        let slot: Mutex<Option<&str>> = Mutex::new(None);

        let result = install_if_current(&generation, 5, &slot, "sink-a");

        assert_eq!(result, Ok(()));
        assert_eq!(*slot.lock(), Some("sink-a"));
    }

    #[test]
    fn install_if_current_rejects_and_hands_the_value_back_when_superseded() {
        // Simulates: play() for gen 5 is mid-flight (slow decoder/sink
        // construction) when a newer play()/stop() bumps the generation to 7
        // before the install happens.
        let generation = AtomicU64::new(7);
        let slot: Mutex<Option<&str>> = Mutex::new(None);

        let result = install_if_current(&generation, 5, &slot, "stale-sink");

        assert_eq!(result, Err("stale-sink"));
        // The whole point of the guard: a superseded value must never reach
        // the shared slot, so a caller checking `slot` for "what's currently
        // playing" (e.g. stop()/set_volume()) can never observe it.
        assert_eq!(*slot.lock(), None);
    }

    #[test]
    fn install_if_current_does_not_clobber_a_pre_existing_value_when_superseded() {
        let generation = AtomicU64::new(99);
        let slot: Mutex<Option<&str>> = Mutex::new(Some("already-playing"));

        let result = install_if_current(&generation, 5, &slot, "stale-sink");

        assert_eq!(result, Err("stale-sink"));
        assert_eq!(*slot.lock(), Some("already-playing"));
    }

    // Code review finding #2, generalized: a drop/reconnect to the *same*
    // station must not re-emit a tile's event (and trigger a needless
    // refetch/flicker), but a genuine switch to a *different* station always
    // must. One shared function/test set now covers all three tiles
    // (Location/Weather/Stream Info), which previously each carried a
    // byte-for-byte identical copy of both the function and these tests.

    #[test]
    fn should_emit_tile_update_emits_on_the_first_call_for_a_station() {
        let mut last = None;
        assert!(should_emit_tile_update(&mut last, "station-a"));
        assert_eq!(last, Some("station-a".to_string()));
    }

    #[test]
    fn should_emit_tile_update_suppresses_a_reconnect_to_the_same_station() {
        let mut last = Some("station-a".to_string());
        assert!(!should_emit_tile_update(&mut last, "station-a"));
        // Tracked value is unchanged, still the same station.
        assert_eq!(last, Some("station-a".to_string()));
    }

    #[test]
    fn should_emit_tile_update_emits_when_switching_to_a_different_station() {
        let mut last = Some("station-a".to_string());
        assert!(should_emit_tile_update(&mut last, "station-b"));
        assert_eq!(last, Some("station-b".to_string()));
    }

    #[tokio::test]
    async fn retry_with_backoff_succeeds_after_transient_failures() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let retries_seen = Arc::new(Mutex::new(Vec::new()));

        let delays = [Duration::ZERO, Duration::ZERO, Duration::ZERO];
        let a = attempts.clone();
        let result = retry_with_backoff(
            &delays,
            move || {
                let a = a.clone();
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err("fail".to_string())
                    } else {
                        Ok(42)
                    }
                }
            },
            |i| retries_seen.lock().push(i),
        )
        .await;

        assert_eq!(result, Ok(42));
        // initial attempt + 2 retries = 3 total attempts
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_with_backoff_gives_up_after_exhausting_all_delays() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let retry_count = Arc::new(AtomicUsize::new(0));

        let delays = [Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO];
        let a = attempts.clone();
        let result: Result<(), String> = retry_with_backoff(
            &delays,
            move || {
                let a = a.clone();
                async move {
                    a.fetch_add(1, Ordering::SeqCst);
                    Err("still failing".to_string())
                }
            },
            {
                let retry_count = retry_count.clone();
                move |_| {
                    retry_count.fetch_add(1, Ordering::SeqCst);
                }
            },
        )
        .await;

        assert!(result.is_err());
        // initial attempt + one retry per delay entry
        assert_eq!(attempts.load(Ordering::SeqCst), 1 + delays.len());
        assert_eq!(retry_count.load(Ordering::SeqCst), delays.len());
    }

    #[tokio::test]
    async fn retry_with_backoff_succeeds_immediately_without_retrying() {
        let calls = Arc::new(AtomicUsize::new(0));
        let retry_calls = Arc::new(AtomicUsize::new(0));
        let delays = RETRY_DELAYS;

        let c = calls.clone();
        let result = retry_with_backoff(
            &delays,
            move || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, String>(())
                }
            },
            {
                let retry_calls = retry_calls.clone();
                move |_| {
                    retry_calls.fetch_add(1, Ordering::SeqCst);
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(retry_calls.load(Ordering::SeqCst), 0);
    }
}
