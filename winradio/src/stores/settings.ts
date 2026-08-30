import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import type { Station } from './stations'

export type Theme = 'system' | 'light' | 'dark'

export interface Settings {
  minimizeToTray: boolean
  sleepTimerDefaultMinutes: number
  theme: Theme
  volume: number
  lastStation?: Station
}

// Settings has no push event of its own (unlike playback) — it's simple
// local config, loaded once and saved immediately on every change (FR-13,
// FR-14), matching the "no explicit Apply/Save button" pattern everywhere
// else in the app. `lastStation` is the one field with a second writer: the
// playback store's `play` event listener calls `setLastStation` (spec-1-5),
// so every `saveSettings` call below reconstructs the *complete* in-memory
// snapshot — never a partial object — or that second writer's field would
// get silently blanked by the next unrelated settings save.
export const useSettingsStore = defineStore('settings', () => {
  const minimizeToTray = ref(false)
  const sleepTimerDefaultMinutes = ref(30)
  const theme = ref<Theme>('system')
  const lastStation = ref<Station | undefined>(undefined)

  const loadSettings = async () => {
    try {
      const saved = await invoke<Settings>('load_settings')
      minimizeToTray.value = saved.minimizeToTray
      sleepTimerDefaultMinutes.value = saved.sleepTimerDefaultMinutes
      theme.value = saved.theme as Theme
      lastStation.value = saved.lastStation
    } catch (e) {
      console.error('Load settings failed:', e)
    }
  }

  const saveSettings = async (currentVolume: number) => {
    try {
      await invoke('save_settings', {
        settings: {
          minimizeToTray: minimizeToTray.value,
          sleepTimerDefaultMinutes: sleepTimerDefaultMinutes.value,
          theme: theme.value,
          volume: currentVolume,
          lastStation: lastStation.value,
        },
      })
    } catch (e) {
      console.error('Save settings failed:', e)
    }
  }

  // Called from the playback store's `play` event listener (mirroring how
  // `save_stations`/`save_settings` are already frontend-triggered
  // persistence calls) — updates the one field, then re-saves the *full*
  // current snapshot via `saveSettings` so nothing else gets dropped.
  // `currentVolume` must be the caller's live volume value (e.g. playback
  // store's `volume.value`), the same way every other `saveSettings` caller
  // (`persistVolume`, the settings modal) already passes it explicitly —
  // never a value cached on this store, which could go stale mid-drag.
  const setLastStation = async (station: Station, currentVolume: number) => {
    lastStation.value = station
    await saveSettings(currentVolume)
  }

  return {
    minimizeToTray,
    sleepTimerDefaultMinutes,
    theme,
    lastStation,
    loadSettings,
    saveSettings,
    setLastStation,
  }
})
