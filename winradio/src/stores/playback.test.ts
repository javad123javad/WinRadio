import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

// `invoke`/`listen` don't exist outside a Tauri webview. `listen` is mocked
// to capture each event's handler so tests can trigger it directly instead
// of needing a real Tauri backend to emit events.
vi.mock('@tauri-apps/api', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))

const handlers: Record<string, (event: { payload: unknown }) => void> = {}
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((eventName: string, handler: (event: { payload: unknown }) => void) => {
    handlers[eventName] = handler
    return Promise.resolve(() => {})
  }),
}))

import { invoke } from '@tauri-apps/api'
import { usePlaybackStore } from './playback'
import { useSettingsStore } from './settings'
import type { Station } from './stations'

const makeStation = (id: string): Station => ({
  id,
  name: id,
  url: `https://example.com/${id}`,
  isFavorite: false,
  addedAt: 0,
  favoriteOrder: 0,
})

describe('usePlaybackStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    for (const key of Object.keys(handlers)) delete handlers[key]
  })

  describe('restoreLastStation', () => {
    it('sets currentStation without setting isPlaying or calling play (AC4)', () => {
      const store = usePlaybackStore()
      const station = makeStation('a')

      store.restoreLastStation(station)

      expect(store.currentStation).toEqual(station)
      expect(store.isPlaying).toBe(false)
      expect(invoke).not.toHaveBeenCalled()
    })

    it('is a no-op when there is no last station (fresh install)', () => {
      const store = usePlaybackStore()

      store.restoreLastStation(undefined)

      expect(store.currentStation).toBeNull()
    })

    it('does not overwrite currentStation when playback is already actually playing (defensive guard)', () => {
      const store = usePlaybackStore()
      const alreadyPlaying = makeStation('already-playing')
      store.currentStation = alreadyPlaying
      store.isPlaying = true

      store.restoreLastStation(makeStation('some-other-station'))

      expect(store.currentStation).toEqual(alreadyPlaying)
    })
  })

  describe('play event listener', () => {
    it('persists the played station as lastStation via the settings store', async () => {
      const playback = usePlaybackStore()
      const settings = useSettingsStore()
      await playback.initListeners()

      const station = makeStation('a')
      handlers['play']({ payload: station })
      // setLastStation's save_settings call is fire-and-forget from within
      // the listener; flush microtasks so it has a chance to run.
      await Promise.resolve()
      await Promise.resolve()

      expect(playback.currentStation).toEqual(station)
      expect(playback.isPlaying).toBe(true)
      expect(settings.lastStation).toEqual(station)
      expect(invoke).toHaveBeenCalledWith('save_settings', expect.objectContaining({
        settings: expect.objectContaining({ lastStation: station }),
      }))
    })

    // Code review finding #1: `setLastStation` must be given the *live*
    // slider volume, not a value cached on the settings store — otherwise a
    // station switch mid-drag (before the slider's `change` event commits)
    // would persist a stale, previously-saved volume instead.
    it('persists the live playback volume, not a stale cached one', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()
      playback.volume = 0.33

      handlers['play']({ payload: makeStation('a') })
      await Promise.resolve()
      await Promise.resolve()

      expect(invoke).toHaveBeenCalledWith('save_settings', expect.objectContaining({
        settings: expect.objectContaining({ volume: 0.33 }),
      }))
    })
  })

  describe('metadata (spec-2-1)', () => {
    it('metadata-updated sets metadata from the event payload', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['metadata-updated']({
        payload: { title: 'Some Great Song', artist: 'A Cool Artist', album: '', artworkUrl: '' },
      })

      expect(playback.metadata.title).toBe('Some Great Song')
      expect(playback.metadata.artist).toBe('A Cool Artist')
    })

    it('play resets metadata to empty, so a new station never shows the previous one\'s stale title', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['metadata-updated']({
        payload: { title: 'Old Song', artist: 'Old Artist', album: '', artworkUrl: '' },
      })
      handlers['play']({ payload: makeStation('a') })

      expect(playback.metadata.title).toBe('')
      expect(playback.metadata.artist).toBe('')
    })

    it('stop resets metadata to empty (AC2: no lingering title once nothing is playing)', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['metadata-updated']({
        payload: { title: 'Some Song', artist: '', album: '', artworkUrl: '' },
      })
      handlers['stop']({ payload: undefined })

      expect(playback.metadata.title).toBe('')
    })

    // Code review finding: a stale title from before a genuine playback
    // failure would misrepresent what's actually playing — unlike
    // `reconnecting`, which keeps the last known title since it's the same
    // stream momentarily interrupted, not abandoned.
    it('playback-error resets metadata to empty', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['metadata-updated']({
        payload: { title: 'Some Song', artist: '', album: '', artworkUrl: '' },
      })
      handlers['playback-error']({ payload: { reason: "Couldn't play this station" } })

      expect(playback.metadata.title).toBe('')
    })
  })

  describe('location (spec-2-2)', () => {
    it('ok:true with coordinates shows nothing until get_location_tile resolves, then populates country + tile', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      let resolveTile!: (value: string) => void
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'get_location_tile') return new Promise((resolve) => (resolveTile = resolve))
        return Promise.resolve(undefined)
      })

      handlers['location-updated']({
        payload: { ok: true, data: { country: 'Belgium', geoLat: 50.85, geoLong: 4.35 }, reason: null },
      })

      // Country known already, but never shown without the map (frozen
      // "never a partial state" constraint) — status is 'ok' yet no
      // tileImage yet.
      expect(playback.location.status).toBe('ok')
      expect(playback.location.tileImage).toBeNull()
      expect(invoke).toHaveBeenCalledWith('get_location_tile', { lat: 50.85, long: 4.35 })

      resolveTile('data:image/png;base64,abc')
      await Promise.resolve()
      await Promise.resolve()

      expect(playback.location.status).toBe('ok')
      expect(playback.location.country).toBe('Belgium')
      expect(playback.location.tileImage).toBe('data:image/png;base64,abc')
    })

    it('ok:false (no coordinates) resolves to unavailable immediately, with no tile fetch attempted', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['location-updated']({
        payload: { ok: false, data: null, reason: 'Location unknown' },
      })

      expect(playback.location.status).toBe('unavailable')
      expect(playback.location.tileImage).toBeNull()
      expect(invoke).not.toHaveBeenCalledWith('get_location_tile', expect.anything())
    })

    it('a tile fetch failure resolves to the same unavailable placeholder as no-coordinates — never a partial (country-only) state', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'get_location_tile') return Promise.reject('network error')
        return Promise.resolve(undefined)
      })

      handlers['location-updated']({
        payload: { ok: true, data: { country: 'Belgium', geoLat: 50.85, geoLong: 4.35 }, reason: null },
      })
      await Promise.resolve()
      await Promise.resolve()

      expect(playback.location.status).toBe('unavailable')
      expect(playback.location.country).toBeNull()
      expect(playback.location.tileImage).toBeNull()
    })

    it('play resets location to idle, so a new station never shows the previous one\'s stale tile', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['location-updated']({
        payload: { ok: true, data: { country: 'Belgium', geoLat: 50.85, geoLong: 4.35 }, reason: null },
      })
      await Promise.resolve()
      handlers['play']({ payload: makeStation('a') })

      expect(playback.location.status).toBe('idle')
      expect(playback.location.country).toBeNull()
    })

    it('stop resets location to idle', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['location-updated']({
        payload: { ok: true, data: { country: 'Belgium', geoLat: 50.85, geoLong: 4.35 }, reason: null },
      })
      handlers['stop']({ payload: undefined })

      expect(playback.location.status).toBe('idle')
    })

    it('playback-error resets location to idle', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['location-updated']({
        payload: { ok: true, data: { country: 'Belgium', geoLat: 50.85, geoLong: 4.35 }, reason: null },
      })
      handlers['playback-error']({ payload: { reason: "Couldn't play this station" } })

      expect(playback.location.status).toBe('idle')
    })

    it('a superseded tile fetch (station switched mid-fetch) never clobbers the newer station\'s location', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      let resolveFirstTile!: (value: string) => void
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'get_location_tile') return new Promise((resolve) => (resolveFirstTile = resolve))
        return Promise.resolve(undefined)
      })

      // First station starts playing and its tile fetch begins...
      handlers['location-updated']({
        payload: { ok: true, data: { country: 'Belgium', geoLat: 50.85, geoLong: 4.35 }, reason: null },
      })

      // ...but a second station takes over before that fetch resolves.
      handlers['play']({ payload: makeStation('b') })
      handlers['location-updated']({
        payload: { ok: false, data: null, reason: 'Location unknown' },
      })

      // The first (now-stale) fetch finally resolves.
      resolveFirstTile('data:image/png;base64,stale')
      await Promise.resolve()
      await Promise.resolve()

      // Must still reflect the second station's (unavailable) state, not
      // the first station's late-arriving tile.
      expect(playback.location.status).toBe('unavailable')
      expect(playback.location.tileImage).toBeNull()
    })
  })

  describe('weather (spec-2-3)', () => {
    it('ok:true populates temperature/condition/high/low directly from the event, no follow-up invoke', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['weather-updated']({
        payload: { ok: true, data: { temperatureC: 18, condition: 'Overcast', forecastHighC: 21, forecastLowC: 12 }, reason: null },
      })

      expect(playback.weather.status).toBe('ok')
      expect(playback.weather.temperatureC).toBe(18)
      expect(playback.weather.condition).toBe('Overcast')
      expect(playback.weather.forecastHighC).toBe(21)
      expect(playback.weather.forecastLowC).toBe(12)
      expect(invoke).not.toHaveBeenCalledWith('get_weather_tile', expect.anything())
    })

    it('ok:false (no coordinates or fetch failure) resolves to unavailable', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['weather-updated']({
        payload: { ok: false, data: null, reason: 'Weather unavailable' },
      })

      expect(playback.weather.status).toBe('unavailable')
      expect(playback.weather.temperatureC).toBeNull()
      expect(playback.weather.condition).toBeNull()
    })

    it('play resets weather to idle, so a new station never shows the previous one\'s stale weather', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['weather-updated']({
        payload: { ok: true, data: { temperatureC: 18, condition: 'Overcast', forecastHighC: 21, forecastLowC: 12 }, reason: null },
      })
      handlers['play']({ payload: makeStation('a') })

      expect(playback.weather.status).toBe('idle')
      expect(playback.weather.temperatureC).toBeNull()
    })

    it('stop resets weather to idle', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['weather-updated']({
        payload: { ok: true, data: { temperatureC: 18, condition: 'Overcast', forecastHighC: 21, forecastLowC: 12 }, reason: null },
      })
      handlers['stop']({ payload: undefined })

      expect(playback.weather.status).toBe('idle')
    })

    it('playback-error resets weather to idle', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['weather-updated']({
        payload: { ok: true, data: { temperatureC: 18, condition: 'Overcast', forecastHighC: 21, forecastLowC: 12 }, reason: null },
      })
      handlers['playback-error']({ payload: { reason: "Couldn't play this station" } })

      expect(playback.weather.status).toBe('idle')
    })
  })

  describe('stream info (spec-2-4)', () => {
    it('ok:true populates the resolved ip directly from the event, no follow-up invoke', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['stream-info-updated']({
        payload: { ok: true, data: '31.12.64.60', reason: null },
      })

      expect(playback.streamInfo.status).toBe('ok')
      expect(playback.streamInfo.ip).toBe('31.12.64.60')
    })

    it('ok:false (DNS failure) resolves to unavailable, with a null ip', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['stream-info-updated']({
        payload: { ok: false, data: null, reason: 'unavailable' },
      })

      expect(playback.streamInfo.status).toBe('unavailable')
      expect(playback.streamInfo.ip).toBeNull()
    })

    it('play resets streamInfo to idle, so a new station never shows the previous one\'s stale ip', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['stream-info-updated']({
        payload: { ok: true, data: '31.12.64.60', reason: null },
      })
      handlers['play']({ payload: makeStation('a') })

      expect(playback.streamInfo.status).toBe('idle')
      expect(playback.streamInfo.ip).toBeNull()
    })

    it('stop resets streamInfo to idle', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['stream-info-updated']({
        payload: { ok: true, data: '31.12.64.60', reason: null },
      })
      handlers['stop']({ payload: undefined })

      expect(playback.streamInfo.status).toBe('idle')
    })

    it('playback-error resets streamInfo to idle', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['stream-info-updated']({
        payload: { ok: true, data: '31.12.64.60', reason: null },
      })
      handlers['playback-error']({ payload: { reason: "Couldn't play this station" } })

      expect(playback.streamInfo.status).toBe('idle')
    })
  })

  describe('sleep timer', () => {
    it('arms the timer via set_sleep_timer, but only reflects it once the armed event round-trips', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      const armPromise = playback.armSleepTimer(30)
      // Not optimistic: still unarmed until the Rust-pushed event arrives.
      expect(playback.sleepTimerArmed).toBe(false)

      handlers['sleep-timer-armed']({ payload: { minutes: 30 } })
      await armPromise

      expect(invoke).toHaveBeenCalledWith('set_sleep_timer', { minutes: 30 })
      expect(playback.sleepTimerArmed).toBe(true)
      expect(playback.sleepTimerMinutes).toBe(30)
    })

    it('clears armed state when a sleep-timer-cleared event arrives (cancel or fire)', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['sleep-timer-armed']({ payload: { minutes: 15 } })
      expect(playback.sleepTimerArmed).toBe(true)

      handlers['sleep-timer-cleared']({ payload: undefined })

      expect(playback.sleepTimerArmed).toBe(false)
      expect(playback.sleepTimerMinutes).toBeNull()
    })

    it('cancelSleepTimer invokes set_sleep_timer with minutes: 0', async () => {
      const playback = usePlaybackStore()
      await playback.initListeners()

      handlers['sleep-timer-armed']({ payload: { minutes: 60 } })
      expect(playback.sleepTimerArmed).toBe(true)

      const cancelPromise = playback.cancelSleepTimer()
      handlers['sleep-timer-cleared']({ payload: undefined })
      await cancelPromise

      expect(invoke).toHaveBeenCalledWith('set_sleep_timer', { minutes: 0 })
      expect(playback.sleepTimerArmed).toBe(false)
      expect(playback.sleepTimerMinutes).toBeNull()
    })
  })
})
