import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api'
import { usePlaybackStore } from './playback'
import type { Station } from './playback'

const DEFAULT_STATIONS: Station[] = [
  { id: 'soma-groove', name: 'SomaFM Groove Salad', url: 'https://ice1.somafm.com/groovesalad-128-mp3', category: 'Ambient', isFavorite: false, addedAt: Date.now() },
  { id: 'soma-drift', name: 'SomaFM Drone Zone', url: 'https://ice1.somafm.com/dronezone-128-mp3', category: 'Ambient', isFavorite: false, addedAt: Date.now() },
  { id: 'soma-lush', name: 'SomaFM Lush', url: 'https://ice1.somafm.com/lush-128-mp3', category: 'Chillout', isFavorite: false, addedAt: Date.now() },
  { id: 'soma-defcon', name: 'SomaFM DEF CON Radio', url: 'https://ice1.somafm.com/defcon-128-mp3', category: 'Electronic', isFavorite: false, addedAt: Date.now() },
  { id: 'soma-space', name: 'SomaFM Space Station', url: 'https://ice1.somafm.com/spacestation-128-mp3', category: 'Ambient', isFavorite: false, addedAt: Date.now() },
  { id: 'paradise-main', name: 'Radio Paradise Main', url: 'https://stream.radioparadise.com/mp3-128', category: 'Eclectic', isFavorite: false, addedAt: Date.now() },
  { id: 'paradise-mellow', name: 'Radio Paradise Mellow', url: 'https://stream.radioparadise.com/mellow-128', category: 'Mellow', isFavorite: false, addedAt: Date.now() },
  { id: 'paradise-rock', name: 'Radio Paradise Rock', url: 'https://stream.radioparadise.com/rock-128', category: 'Rock', isFavorite: false, addedAt: Date.now() },
  { id: 'kexp', name: 'KEXP Seattle', url: 'https://live-aacplus-64.kexp.org/kexp64.aac', category: 'Indie', isFavorite: false, addedAt: Date.now() },
  { id: 'wfmu', name: 'WFMU Freeform', url: 'https://wfmu-ice.streamguys1.com/wfmu-128.mp3', category: 'Freeform', isFavorite: false, addedAt: Date.now() },
  { id: 'npr', name: 'NPR News', url: 'https://npr-ice.streamguys1.com/live.mp3', category: 'News', isFavorite: false, addedAt: Date.now() },
  { id: 'bbc-radio1', name: 'BBC Radio 1', url: 'https://stream.live.vc.bbcmedia.co.uk/bbc_radio1', category: 'Pop', isFavorite: false, addedAt: Date.now() },
  { id: 'bbc-radio2', name: 'BBC Radio 2', url: 'https://stream.live.vc.bbcmedia.co.uk/bbc_radio2', category: 'Mixed', isFavorite: false, addedAt: Date.now() },
  { id: 'bbc-6music', name: 'BBC 6 Music', url: 'https://stream.live.vc.bbcmedia.co.uk/bbc_6music', category: 'Alternative', isFavorite: false, addedAt: Date.now() },
  { id: 'fip', name: 'FIP (France)', url: 'https://icecast.radiofrance.fr/fip-midfi.mp3', category: 'Eclectic', isFavorite: false, addedAt: Date.now() },
  { id: 'jazz24', name: 'Jazz24', url: 'https://live.wostreaming.net/direct/ppb-jazz24-ibc1-64', category: 'Jazz', isFavorite: false, addedAt: Date.now() },
  { id: 'classical-king', name: 'Classical King FM', url: 'https://player.streamtheworld.com/KINGFMAAC.aac', category: 'Classical', isFavorite: false, addedAt: Date.now() },
  { id: 'swr1', name: 'SWR1 Baden-Württemberg', url: 'https://swr1bw.akacast.akamaistream.net/7/957/137185/v1/gnl.akacast.akamaistream.net/swr1bw', category: 'Oldies', isFavorite: false, addedAt: Date.now() },
  { id: 'swr3', name: 'SWR3', url: 'https://swr3.akacast.akamaistream.net/7/720/137187/v1/gnl.akacast.akamaistream.net/swr3', category: 'Pop', isFavorite: false, addedAt: Date.now() },
  { id: 'ndr2', name: 'NDR 2', url: 'https://ndr-ndr2-niedersachsen.akacast.akamaistream.net/7/516/137184/v1/gnl.akacast.akamaistream.net/ndr-ndr2-niedersachsen', category: 'Pop', isFavorite: false, addedAt: Date.now() },
]

export const useStationsStore = defineStore('stations', () => {
  const stations = ref<Station[]>([])
  const searchQuery = ref('')
  const selectedCategory = ref<string>('all')
  const sortBy = ref<'name' | 'category' | 'added'>('name')
  const sortAsc = ref(true)

  const playbackStore = usePlaybackStore()

  const categories = computed(() => {
    const cats = new Set(stations.value.map(s => s.category).filter(Boolean))
    return ['all', ...Array.from(cats).sort()]
  })

  const filteredStations = computed(() => {
    let result = stations.value

    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      result = result.filter(s =>
        s.name.toLowerCase().includes(q) ||
        s.category?.toLowerCase().includes(q)
      )
    }

    if (selectedCategory.value !== 'all') {
      result = result.filter(s => s.category === selectedCategory.value)
    }

    result = [...result].sort((a, b) => {
      let cmp = 0
      if (sortBy.value === 'name') cmp = a.name.localeCompare(b.name)
      else if (sortBy.value === 'category') cmp = (a.category || '').localeCompare(b.category || '')
      else if (sortBy.value === 'added') cmp = a.addedAt - b.addedAt
      return sortAsc.value ? cmp : -cmp
    })

    return result
  })

  const favorites = computed(() => stations.value.filter(s => s.isFavorite))

  const loadStations = async () => {
    try {
      const saved = await invoke<Station[]>('list_stations')
      if (saved.length > 0) {
        stations.value = saved
      } else {
        stations.value = DEFAULT_STATIONS
        await saveStations()
      }
    } catch {
      stations.value = DEFAULT_STATIONS
      await saveStations()
    }
  }

  const saveStations = async () => {
    try {
      await invoke('save_stations', { stations: stations.value })
    } catch (e) {
      console.error('Save stations failed:', e)
    }
  }

  const addStation = async (station: Omit<Station, 'id' | 'addedAt'>) => {
    const newStation: Station = {
      ...station,
      id: `custom-${Date.now()}`,
      addedAt: Date.now(),
    }
    stations.value.unshift(newStation)
    await saveStations()
    return newStation
  }

  const updateStation = async (updated: Station) => {
    const idx = stations.value.findIndex(s => s.id === updated.id)
    if (idx >= 0) {
      stations.value[idx] = updated
      await saveStations()
    }
  }

  const deleteStation = async (id: string) => {
    stations.value = stations.value.filter(s => s.id !== id)
    await saveStations()
  }

  const toggleFavorite = async (id: string) => {
    const station = stations.value.find(s => s.id === id)
    if (station) {
      station.isFavorite = !station.isFavorite
      await saveStations()
    }
  }

  const playStation = (station: Station) => {
    playbackStore.play(station.url, station)
  }

  const playNext = () => {
    const current = playbackStore.currentStation
    if (!current) return
    const idx = stations.value.findIndex(s => s.id === current.id)
    const next = stations.value[(idx + 1) % stations.value.length]
    if (next) playStation(next)
  }

  const playPrevious = () => {
    const current = playbackStore.currentStation
    if (!current) return
    const idx = stations.value.findIndex(s => s.id === current.id)
    const prev = stations.value[(idx - 1 + stations.value.length) % stations.value.length]
    if (prev) playStation(prev)
  }

  const importStations = async (data: Station[]) => {
    const existingIds = new Set(stations.value.map(s => s.id))
    const imported = data.filter(s => !existingIds.has(s.id))
    stations.value = [...imported, ...stations.value]
    await saveStations()
    return imported.length
  }

  const exportStations = () => {
    return stations.value
  }

  return {
    stations,
    searchQuery,
    selectedCategory,
    sortBy,
    sortAsc,
    categories,
    filteredStations,
    favorites,
    loadStations,
    addStation,
    updateStation,
    deleteStation,
    toggleFavorite,
    playStation,
    playNext,
    playPrevious,
    importStations,
    exportStations,
  }
})