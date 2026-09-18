import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'
import type { Station } from './stations'
import { useSettingsStore } from './settings'

export interface Metadata {
  title: string
  artist: string
  album: string
  artworkUrl: string
}

const emptyMetadata = (): Metadata => ({ title: '', artist: '', album: '', artworkUrl: '' })

// Shared `{ok, data, reason}` envelope (AD-5) — this is `location-updated`'s
// `data` shape.
export interface LocationEventData {
  country: string | null
  geoLat: number
  geoLong: number
}

export interface LocationState {
  status: 'idle' | 'ok' | 'unavailable'
  country: string | null
  tileImage: string | null
}

const emptyLocation = (): LocationState => ({ status: 'idle', country: null, tileImage: null })

// Shared `{ok, data, reason}` envelope (AD-5) — this is `weather-updated`'s
// `data` shape. Unlike Location, the whole payload rides inside the event
// itself — no follow-up `invoke()` fetch.
export interface WeatherEventData {
  temperatureC: number
  condition: string
  forecastHighC: number
  forecastLowC: number
}

export interface WeatherState {
  status: 'idle' | 'ok' | 'unavailable'
  temperatureC: number | null
  condition: string | null
  forecastHighC: number | null
  forecastLowC: number | null
}

const emptyWeather = (): WeatherState => ({
  status: 'idle',
  temperatureC: null,
  condition: null,
  forecastHighC: null,
  forecastLowC: null,
})

// `stream-info-updated`'s `data` shape (spec-2-4) — unlike Location/Weather,
// this is just the resolved IP string, not a multi-field object; codec/
// bitrate/country never ride this event at all (they're read synchronously
// off `currentStation` instead).
export interface StreamInfoState {
  status: 'idle' | 'ok' | 'unavailable'
  ip: string | null
}

const emptyStreamInfo = (): StreamInfoState => ({ status: 'idle', ip: null })

// Read-model only: every field here is set from a `listen()` handler tied to
// a Rust-pushed event, never optimistically after an `invoke()` call, per
// spec-1-1's event model. The one exception is `volume`, which is live
// slider UI state (FR-12) with no backing event of its own.
export const usePlaybackStore = defineStore('playback', () => {
  const isPlaying = ref(false)
  const currentStation = ref<Station | null>(null)
  const volume = ref(0.7)
  const isMuted = ref(false)
  const volumeBeforeMute = ref(0.7)
  const metadata = ref<Metadata>(emptyMetadata())
  const reconnecting = ref(false)
  const reconnectAttempt = ref(0)
  const errorMessage = ref<string | null>(null)
  // Local, cosmetic-only clock for the informational elapsed-time display
  // (EXPERIENCE.md: "position display is elapsed-time-only"). Never used to
  // poll backend state — internet radio streams have no seek/duration.
  const playStartedAt = ref<number | null>(null)
  // Read-model for the Sleep Timer's armed state (spec-1-7). Deliberately no
  // countdown/remaining-time field anywhere — the badge is the only UI
  // indicator (Boundaries & Constraints: "No persistent countdown"). Cleared
  // only by an explicit cancel or by firing, never by unrelated
  // pause/play elsewhere (the Rust `SleepTimer` is independent of playback
  // state changes made outside `set_sleep_timer`).
  const sleepTimerArmed = ref(false)
  const sleepTimerMinutes = ref<number | null>(null)
  // Read-model for the Location Tile (spec-2-2), same conventions as
  // `metadata` above: only ever set from `location-updated` (never
  // optimistically), reset to idle on `play`/`stop`/`playback-error`.
  const location = ref<LocationState>(emptyLocation())
  // Read-model for the Weather Tile (spec-2-3), same conventions as
  // `location` above: only ever set from `weather-updated` (never
  // optimistically), reset to idle on `play`/`stop`/`playback-error`.
  const weather = ref<WeatherState>(emptyWeather())
  // Read-model for the Stream Info Tile's IP field (spec-2-4), same
  // conventions as `location`/`weather` above: only ever set from
  // `stream-info-updated` (never optimistically), reset to idle on
  // `play`/`stop`/`playback-error`. Codec/bitrate/country render straight
  // off `currentStation` in the component instead — no store state needed
  // for those.
  const streamInfo = ref<StreamInfoState>(emptyStreamInfo())

  let listenersReady = false
  // Bumped on every reset (play/stop/playback-error) and on every
  // `location-updated` event — the in-flight `get_location_tile` fetch it
  // started checks this before applying its result, so a station switch (or
  // stop) mid-fetch can never let a stale tile/failure clobber a newer
  // station's already-current location state.
  let locationRequestSeq = 0

  // Only flips to `true` once every `listen()` call below has actually
  // resolved. If any of them throws (e.g. a transient IPC hiccup during
  // startup), the flag is reset so a later `initListeners()` call retries
  // registration instead of permanently no-op'ing (previously the flag was
  // set up front, so a partial failure here silently dropped whichever
  // events hadn't registered yet, for the rest of the session).
  const initListeners = async () => {
    if (listenersReady) return
    try {
      await listen<Station>('play', (event) => {
        currentStation.value = event.payload
        isPlaying.value = true
        reconnecting.value = false
        errorMessage.value = null
        metadata.value = emptyMetadata()
        // spec-2-2: reset at the same point as `metadata` — a stale
        // previous station's location/tile must never linger. The
        // `location-updated` event (fired right after this one on the Rust
        // side) supplies the real state moments later.
        location.value = emptyLocation()
        locationRequestSeq++
        weather.value = emptyWeather()
        streamInfo.value = emptyStreamInfo()
        playStartedAt.value = Date.now()

        // Persist which station was last played (spec-1-5, AC4) so it can
        // be restored idle on the next launch. Full station snapshot, not
        // just an id — see settings.ts. Fire-and-forget: a failed persist
        // here shouldn't block playback UI state above. Pass the *live*
        // slider volume (`volume.value`), same as `persistVolume` below —
        // not a value cached on the settings store, which could be stale if
        // this fires mid-drag before the slider's `change` event commits.
        const settingsStore = useSettingsStore()
        void settingsStore.setLastStation(event.payload, volume.value)
      })

      await listen('stop', () => {
        isPlaying.value = false
        reconnecting.value = false
        // Nothing is playing any more — a stale track title left over from
        // the last station would otherwise linger in the UI.
        metadata.value = emptyMetadata()
        location.value = emptyLocation()
        locationRequestSeq++
        weather.value = emptyWeather()
        streamInfo.value = emptyStreamInfo()
        playStartedAt.value = null
      })

      await listen<{ attempt: number }>('reconnecting', (event) => {
        reconnecting.value = true
        reconnectAttempt.value = event.payload.attempt
        errorMessage.value = null
      })

      await listen<{ reason: string }>('playback-error', (event) => {
        isPlaying.value = false
        reconnecting.value = false
        errorMessage.value = event.payload.reason
        playStartedAt.value = null
        // Playback has genuinely stopped (not just interrupted mid-retry
        // like `reconnecting`) — a stale track title from before the
        // failure would misrepresent what's actually playing (spec-2-1).
        metadata.value = emptyMetadata()
        location.value = emptyLocation()
        locationRequestSeq++
        weather.value = emptyWeather()
        streamInfo.value = emptyStreamInfo()
      })

      await listen<Metadata>('metadata-updated', (event) => {
        metadata.value = event.payload
      })

      // spec-2-2: fires once per play attempt, right after `play`.
      // `ok: false` (no cached coordinates) resolves immediately to the
      // "unavailable" placeholder, no tile fetch attempted. `ok: true` shows
      // nothing yet (never country-without-a-map, per the frozen "never a
      // partial state" constraint) until `get_location_tile` actually
      // resolves; a fetch failure resolves to that exact same placeholder.
      await listen<{ ok: boolean; data: LocationEventData | null; reason: string | null }>(
        'location-updated',
        (event) => {
          const seq = ++locationRequestSeq
          const { ok, data } = event.payload

          if (!ok || !data) {
            location.value = { status: 'unavailable', country: null, tileImage: null }
            return
          }

          location.value = { status: 'ok', country: data.country, tileImage: null }
          void invoke<string>('get_location_tile', { lat: data.geoLat, long: data.geoLong })
            .then((tileImage) => {
              if (seq !== locationRequestSeq) return // superseded by a newer play/stop
              location.value = { status: 'ok', country: data.country, tileImage }
            })
            .catch((e) => {
              if (seq !== locationRequestSeq) return
              console.error('Get location tile failed:', e)
              location.value = { status: 'unavailable', country: null, tileImage: null }
            })
        }
      )

      // spec-2-3: fires once per play attempt, right after `location-updated`.
      // Unlike Location, the whole payload rides inside the event itself —
      // no follow-up `invoke()` fetch/superseded-fetch bookkeeping needed.
      await listen<{ ok: boolean; data: WeatherEventData | null; reason: string | null }>(
        'weather-updated',
        (event) => {
          const { ok, data } = event.payload

          if (!ok || !data) {
            weather.value = { status: 'unavailable', temperatureC: null, condition: null, forecastHighC: null, forecastLowC: null }
            return
          }

          weather.value = {
            status: 'ok',
            temperatureC: data.temperatureC,
            condition: data.condition,
            forecastHighC: data.forecastHighC,
            forecastLowC: data.forecastLowC,
          }
        }
      )

      // spec-2-4: fires once per play attempt, right after
      // `weather-updated`. Unlike Location/Weather, `data` is just the
      // resolved IP string — set directly, no follow-up `invoke()` fetch and
      // no multi-field mapping needed.
      await listen<{ ok: boolean; data: string | null; reason: string | null }>(
        'stream-info-updated',
        (event) => {
          const { ok, data } = event.payload
          streamInfo.value = ok ? { status: 'ok', ip: data } : { status: 'unavailable', ip: null }
        }
      )

      await listen<{ minutes: number }>('sleep-timer-armed', (event) => {
        sleepTimerArmed.value = true
        sleepTimerMinutes.value = event.payload.minutes
      })

      await listen('sleep-timer-cleared', () => {
        sleepTimerArmed.value = false
        sleepTimerMinutes.value = null
      })

      listenersReady = true
    } catch (e) {
      listenersReady = false
      console.error('Registering playback listeners failed:', e)
      throw e
    }
  }

  // Restores the last-played station's info into the Now-Playing area on
  // app launch (spec-1-5, AC4) without playing it: sets `currentStation`
  // only — never `isPlaying`, never calls `play()`. The existing dashboard
  // template (`v-if="playbackStore.currentStation"`) already renders idle
  // correctly once this is set. Defensive guard: never overwrite a station
  // that's already actually playing — today's startup ordering can't
  // trigger this (no auto-play path exists, and nothing `await`s between
  // settings loading and this call), but it costs nothing to protect
  // against a future refactor that changes that ordering.
  const restoreLastStation = (station: Station | null | undefined) => {
    if (isPlaying.value || !station) return
    currentStation.value = station
  }

  const play = async (station: Station) => {
    errorMessage.value = null
    try {
      await invoke('play', { station })
    } catch (e) {
      console.error('Play failed:', e)
    }
  }

  const stop = async () => {
    try {
      await invoke('stop')
    } catch (e) {
      console.error('Stop failed:', e)
    }
  }

  const setVolume = async (vol: number) => {
    const clamped = Math.max(0, Math.min(1, vol))
    volume.value = clamped
    try {
      await invoke('set_volume', { volume: clamped })
    } catch (e) {
      console.error('Set volume failed:', e)
    }
  }

  const loadVolume = async () => {
    try {
      volume.value = await invoke<number>('get_volume')
    } catch (e) {
      console.error('Load volume failed:', e)
    }
  }

  // Volume lives in the `Settings` record on the Rust side (so it can be
  // restored on relaunch — I/O matrix: "last volume restored"), but
  // `save_settings` replaces the whole record, so persisting it means going
  // through the settings store. `setVolume` above only pushes the live value
  // to the backend for immediate playback — without this, dragging the
  // transport-bar slider alone never survived a restart. Call this on
  // "release" (e.g. `change`, not `input`), matching "persists on release"
  // (FR-12) rather than writing to disk on every drag tick.
  const persistVolume = async () => {
    const settingsStore = useSettingsStore()
    await settingsStore.saveSettings(volume.value)
  }

  // Mute is its own state, not "drag to zero": muting and later unmuting
  // restores the prior volume level exactly (EXPERIENCE.md, transport bar).
  const toggleMute = async () => {
    if (isMuted.value) {
      isMuted.value = false
      await setVolume(volumeBeforeMute.value)
    } else {
      volumeBeforeMute.value = volume.value
      isMuted.value = true
      await setVolume(0)
    }
  }

  // Arms (or re-arms) the Sleep Timer for `minutes` minutes. Not
  // optimistic — `sleepTimerArmed`/`sleepTimerMinutes` only update once the
  // Rust side's `sleep-timer-armed` event round-trips back (matching this
  // store's read-model-only convention for every other field above).
  // Re-arming while already armed simply supersedes the old timer on the
  // Rust side; the new `sleep-timer-armed` event reflects the new duration.
  const armSleepTimer = async (minutes: number) => {
    try {
      await invoke('set_sleep_timer', { minutes })
    } catch (e) {
      console.error('Arm sleep timer failed:', e)
    }
  }

  // Cancels the currently-armed timer. `set_sleep_timer` treats `minutes: 0`
  // as "cancel" (existing Rust command semantics) rather than "arm for zero
  // minutes".
  const cancelSleepTimer = async () => {
    try {
      await invoke('set_sleep_timer', { minutes: 0 })
    } catch (e) {
      console.error('Cancel sleep timer failed:', e)
    }
  }

  return {
    isPlaying,
    currentStation,
    volume,
    isMuted,
    metadata,
    reconnecting,
    reconnectAttempt,
    errorMessage,
    playStartedAt,
    sleepTimerArmed,
    sleepTimerMinutes,
    location,
    weather,
    streamInfo,
    initListeners,
    restoreLastStation,
    play,
    stop,
    setVolume,
    loadVolume,
    persistVolume,
    toggleMute,
    armSleepTimer,
    cancelSleepTimer,
  }
})
