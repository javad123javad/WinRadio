<template>
  <Transition name="fade">
    <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="$emit('close')">
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Settings</h2>
          <button @click="$emit('close')" class="p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
            <svg class="w-5 h-5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="p-6 space-y-6">
          <section>
            <h3 class="text-sm font-medium text-gray-900 dark:text-white mb-3">General</h3>
            <div class="space-y-3">
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Minimize to tray on close</span>
                <input
                  type="checkbox"
                  v-model="settings.minimizeToTray"
                  @change="saveSettings"
                  class="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
                />
              </label>
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Start minimized</span>
                <input
                  type="checkbox"
                  v-model="settings.startMinimized"
                  @change="saveSettings"
                  class="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
                />
              </label>
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Show notifications on track change</span>
                <input
                  type="checkbox"
                  v-model="settings.showNotifications"
                  @change="saveSettings"
                  class="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
                />
              </label>
            </div>
          </section>

          <section>
            <h3 class="text-sm font-medium text-gray-900 dark:text-white mb-3">Audio</h3>
            <div class="space-y-4">
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-1">Output Device</label>
                <select
                  v-model="settings.outputDevice"
                  @change="saveSettings"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  <option value="default">Default</option>
                  <option v-for="dev in audioDevices" :key="dev.id" :value="dev.id">{{ dev.name }}</option>
                </select>
              </div>
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-1">Buffer Size: {{ settings.bufferSize }} ms</label>
                <input
                  type="range"
                  min="100"
                  max="2000"
                  step="100"
                  v-model.number="settings.bufferSize"
                  @change="saveSettings"
                  class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none accent-blue-500"
                />
              </div>
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Enable Equalizer</span>
                <input
                  type="checkbox"
                  v-model="settings.eqEnabled"
                  @change="saveSettings"
                  class="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
                />
              </label>
            </div>
          </section>

          <section>
            <h3 class="text-sm font-medium text-gray-900 dark:text-white mb-3">Appearance</h3>
            <div class="space-y-3">
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Dark Mode</span>
                <select
                  v-model="settings.theme"
                  @change="saveSettings"
                  class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  <option value="system">System</option>
                  <option value="light">Light</option>
                  <option value="dark">Dark</option>
                </select>
              </label>
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Compact Player</span>
                <input
                  type="checkbox"
                  v-model="settings.compactPlayer"
                  @change="saveSettings"
                  class="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
                />
              </label>
            </div>
          </section>

          <section>
            <h3 class="text-sm font-medium text-gray-900 dark:text-white mb-3">Recording</h3>
            <div class="space-y-3">
              <label class="flex items-center justify-between">
                <span class="text-sm text-gray-700 dark:text-gray-300">Default Format</span>
                <select
                  v-model="settings.recordingFormat"
                  @change="saveSettings"
                  class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  <option value="mp3">MP3 (compressed)</option>
                  <option value="wav">WAV (uncompressed)</option>
                </select>
              </label>
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-1">MP3 Bitrate: {{ settings.recordingBitrate }} kbps</label>
                <select
                  v-model="settings.recordingBitrate"
                  @change="saveSettings"
                  class="w-full px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  <option value="128">128</option>
                  <option value="192">192</option>
                  <option value="256">256</option>
                  <option value="320">320</option>
                </select>
              </div>
            </div>
          </section>

          <section>
            <h3 class="text-sm font-medium text-gray-900 dark:text-white mb-3">Data</h3>
            <div class="flex gap-3">
              <button
                @click="exportData"
                class="flex-1 px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-sm font-medium"
              >
                Export Stations
              </button>
              <button
                @click="clearAllData"
                class="flex-1 px-4 py-2 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/50 transition-colors text-sm font-medium"
              >
                Clear All Data
              </button>
            </div>
          </section>
        </div>

        <div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-3">
          <button
            @click="$emit('close')"
            class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api'

interface Settings {
  minimizeToTray: boolean
  startMinimized: boolean
  showNotifications: boolean
  outputDevice: string
  bufferSize: number
  eqEnabled: boolean
  theme: 'system' | 'light' | 'dark'
  compactPlayer: boolean
  recordingFormat: 'mp3' | 'wav'
  recordingBitrate: number
}

interface AudioDevice {
  id: string
  name: string
}

const props = defineProps({
  show: Boolean
})

const settings = ref<Settings>({
  minimizeToTray: false,
  startMinimized: false,
  showNotifications: true,
  outputDevice: 'default',
  bufferSize: 500,
  eqEnabled: true,
  theme: 'system',
  compactPlayer: false,
  recordingFormat: 'mp3',
  recordingBitrate: 192,
})

const audioDevices = ref<AudioDevice[]>([])

const loadSettings = async () => {
  try {
    const saved = await invoke<Settings>('load_settings')
    if (saved) settings.value = { ...settings.value, ...saved }
    const devices = await invoke<AudioDevice[]>('list_audio_devices')
    audioDevices.value = devices
  } catch (e) {
    console.error('Load settings failed:', e)
  }
}

const saveSettings = async () => {
  try {
    await invoke('save_settings', { settings: settings.value })
  } catch (e) {
    console.error('Save settings failed:', e)
  }
}

const exportData = async () => {
  try {
    const data = await invoke<string>('export_stations')
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `winradio-stations-${new Date().toISOString().split('T')[0]}.json`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e) {
    console.error('Export failed:', e)
  }
}

const clearAllData = async () => {
  if (!confirm('This will delete all stations, history, and settings. Are you sure?')) return
  try {
    await invoke('clear_all_data')
    settings.value = {
      minimizeToTray: false,
      startMinimized: false,
      showNotifications: true,
      outputDevice: 'default',
      bufferSize: 500,
      eqEnabled: true,
      theme: 'system',
      compactPlayer: false,
      recordingFormat: 'mp3',
      recordingBitrate: 192,
    }
  } catch (e) {
    console.error('Clear data failed:', e)
  }
}

watch(() => props.show, (val) => {
  if (val) loadSettings()
}, { immediate: true })
</script>