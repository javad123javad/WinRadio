import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

// `saveSettings`/`loadSettings` call Tauri's `invoke`, which doesn't exist
// outside a Tauri webview. Mocked so store-mutation tests exercise only the
// logic under test, and let tests assert exactly what payload was sent to
// `save_settings` (spec-1-5: never a partial object).
vi.mock('@tauri-apps/api', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))

import { invoke } from '@tauri-apps/api'
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

describe('useSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('setLastStation reconstructs the full settings snapshot (not a partial object)', async () => {
    const store = useSettingsStore()
    store.minimizeToTray = true
    store.sleepTimerDefaultMinutes = 45
    store.theme = 'dark'

    // The caller passes the live volume explicitly (code review finding
    // #1) — the store has no volume state of its own to go stale.
    await store.setLastStation(makeStation('a'), 0.42)

    expect(store.lastStation).toEqual(makeStation('a'))
    expect(invoke).toHaveBeenCalledWith('save_settings', {
      settings: {
        minimizeToTray: true,
        sleepTimerDefaultMinutes: 45,
        theme: 'dark',
        volume: 0.42,
        lastStation: makeStation('a'),
      },
    })
  })

  it('saveSettings (e.g. from the settings modal) round-trips an already-set lastStation unchanged', async () => {
    const store = useSettingsStore()
    await store.setLastStation(makeStation('a'), 0.7)
    vi.mocked(invoke).mockClear()

    // Simulates the settings modal changing theme and saving — must not
    // blank out the lastStation set by the playback listener earlier.
    store.theme = 'light'
    await store.saveSettings(0.5)

    expect(invoke).toHaveBeenCalledWith('save_settings', {
      settings: expect.objectContaining({
        theme: 'light',
        lastStation: makeStation('a'),
      }),
    })
  })

  it('loadSettings populates lastStation from the persisted snapshot when present', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      minimizeToTray: false,
      sleepTimerDefaultMinutes: 30,
      theme: 'system',
      volume: 0.7,
      lastStation: makeStation('b'),
    })

    const store = useSettingsStore()
    await store.loadSettings()

    expect(store.lastStation).toEqual(makeStation('b'))
  })

  it('loadSettings leaves lastStation undefined on a fresh install (no persisted station)', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      minimizeToTray: false,
      sleepTimerDefaultMinutes: 30,
      theme: 'system',
      volume: 0.7,
    })

    const store = useSettingsStore()
    await store.loadSettings()

    expect(store.lastStation).toBeUndefined()
  })
})
