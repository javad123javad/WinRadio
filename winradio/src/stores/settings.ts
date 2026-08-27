import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'

export type Theme = 'system' | 'light' | 'dark'

export interface Settings {
  minimizeToTray: boolean
  sleepTimerDefaultMinutes: number
  theme: Theme
  volume: number
}

// Settings has no push event of its own (unlike playback) — it's simple
// local config, loaded once and saved immediately on every change (FR-13,
// FR-14), matching the "no explicit Apply/Save button" pattern everywhere
// else in the app.
export const useSettingsStore = defineStore('settings', () => {
  const minimizeToTray = ref(false)
  const sleepTimerDefaultMinutes = ref(30)
  const theme = ref<Theme>('system')

  const loadSettings = async () => {
    try {
      const saved = await invoke<Settings>('load_settings')
      minimizeToTray.value = saved.minimizeToTray
      sleepTimerDefaultMinutes.value = saved.sleepTimerDefaultMinutes
      theme.value = saved.theme as Theme
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
        },
      })
    } catch (e) {
      console.error('Save settings failed:', e)
    }
  }

  return {
    minimizeToTray,
    sleepTimerDefaultMinutes,
    theme,
    loadSettings,
    saveSettings,
  }
})
