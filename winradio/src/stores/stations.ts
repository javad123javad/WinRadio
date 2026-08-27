import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { usePlaybackStore } from './playback'

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

// Seed content so the rail isn't empty on a first run, before Search &
// Browse (Story 1.2) gives Javad a way to find and save his own stations.
const DEFAULT_STATIONS: Station[] = [
  { id: 'soma-groove', name: 'SomaFM Groove Salad', url: 'https://ice1.somafm.com/groovesalad-128-mp3', category: 'Ambient', isFavorite: true, addedAt: Date.now() },
  { id: 'soma-drift', name: 'SomaFM Drone Zone', url: 'https://ice1.somafm.com/dronezone-128-mp3', category: 'Ambient', isFavorite: true, addedAt: Date.now() },
  { id: 'paradise-main', name: 'Radio Paradise Main', url: 'https://stream.radioparadise.com/mp3-128', category: 'Eclectic', isFavorite: true, addedAt: Date.now() },
  { id: 'kexp', name: 'KEXP Seattle', url: 'https://live-aacplus-64.kexp.org/kexp64.aac', category: 'Indie', isFavorite: true, addedAt: Date.now() },
]

export const useStationsStore = defineStore('stations', () => {
  const stations = ref<Station[]>([])
  const loaded = ref(false)

  const loadStations = async () => {
    try {
      const saved = await invoke<Station[]>('list_stations')
      stations.value = saved.length > 0 ? saved : DEFAULT_STATIONS
      if (saved.length === 0) {
        await saveStations()
      }
    } catch (e) {
      console.error('Load stations failed:', e)
      stations.value = DEFAULT_STATIONS
    } finally {
      loaded.value = true
    }
  }

  const saveStations = async () => {
    try {
      await invoke('save_stations', { stations: stations.value })
    } catch (e) {
      console.error('Save stations failed:', e)
    }
  }

  // Row click plays immediately (FR-4) — no select-then-play step. Favorite
  // toggling, reordering, and search results are Stories 1.2-1.4.
  const playStation = (station: Station) => {
    const playbackStore = usePlaybackStore()
    playbackStore.play(station)
  }

  return {
    stations,
    loaded,
    loadStations,
    saveStations,
    playStation,
  }
})
