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
})
