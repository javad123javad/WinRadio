import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'

export interface Metadata {
  title: string
  artist: string
  album: string
  artworkUrl: string
}

export interface Station {
  id: string
  name: string
  url: string
  faviconUrl?: string
  homepage?: string
  category?: string
  isFavorite: boolean
  addedAt: number
}

export const usePlaybackStore = defineStore('playback', () => {
  const isPlaying = ref(false)
  const currentStation = ref<Station | null>(null)
  const volume = ref(0.7)
  const metadata = ref<Metadata>({ title: '', artist: '', album: '', artworkUrl: '' })
  const position = ref(0)
  const duration = ref(0)
  const eqBands = ref<number[]>(new Array(10).fill(0))
  const sleepTimerMinutes = ref(0)
  const sleepTimerEndsAt = ref(0)
  const isRecording = ref(false)
  const recordingPath = ref('')

  const play = async (url: string, station?: Station) => {
    try {
      await invoke('play', { url })
      isPlaying.value = true
      if (station) currentStation.value = station
      metadata.value = { title: '', artist: '', album: '', artworkUrl: '' }
      position.value = 0
      duration.value = 0
    } catch (e) {
      console.error('Play failed:', e)
    }
  }

  const stop = async () => {
    try {
      await invoke('stop')
      isPlaying.value = false
      currentStation.value = null
      position.value = 0
      duration.value = 0
    } catch (e) {
      console.error('Stop failed:', e)
    }
  }

  const setVolume = async (vol: number) => {
    volume.value = Math.max(0, Math.min(1, vol))
    try {
      await invoke('set_volume', { volume: volume.value })
    } catch (e) {
      console.error('Set volume failed:', e)
    }
  }

  const setEqBand = async (band: number, gainDb: number) => {
    eqBands.value[band] = Math.max(-12, Math.min(12, gainDb))
    try {
      await invoke('set_eq_band', { band, gainDb: eqBands.value[band] })
    } catch (e) {
      console.error('Set EQ band failed:', e)
    }
  }

  const resetEq = async () => {
    eqBands.value.fill(0)
    try {
      await invoke('reset_eq')
    } catch (e) {
      console.error('Reset EQ failed:', e)
    }
  }

  const updateMetadata = (data: Partial<Metadata>) => {
    metadata.value = { ...metadata.value, ...data }
  }

  const updatePosition = (pos: number, dur: number) => {
    position.value = pos
    duration.value = dur
  }

  const startRecording = async (filename: string) => {
    try {
      const path = await invoke<string>('start_recording', { filename })
      isRecording.value = true
      recordingPath.value = path
      return path
    } catch (e) {
      console.error('Start recording failed:', e)
    }
  }

  const stopRecording = async () => {
    try {
      await invoke('stop_recording')
      isRecording.value = false
    } catch (e) {
      console.error('Stop recording failed:', e)
    }
  }

  const setSleepTimer = async (minutes: number) => {
    sleepTimerMinutes.value = minutes
    sleepTimerEndsAt.value = minutes > 0 ? Date.now() + minutes * 60 * 1000 : 0
    try {
      await invoke('set_sleep_timer', { minutes })
    } catch (e) {
      console.error('Set sleep timer failed:', e)
    }
  }

  return {
    isPlaying,
    currentStation,
    volume,
    metadata,
    position,
    duration,
    eqBands,
    sleepTimerMinutes,
    sleepTimerEndsAt,
    isRecording,
    recordingPath,
    play,
    stop,
    setVolume,
    setEqBand,
    resetEq,
    updateMetadata,
    updatePosition,
    startRecording,
    stopRecording,
    setSleepTimer,
  }
})