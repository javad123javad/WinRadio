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

  let listenersReady = false

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
      })

      await listen<Metadata>('metadata-updated', (event) => {
        metadata.value = event.payload
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
    initListeners,
    restoreLastStation,
    play,
    stop,
    setVolume,
    loadVolume,
    persistVolume,
    toggleMute,
  }
})
