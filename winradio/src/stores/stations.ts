import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
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
  favoriteOrder: number
  // spec-2-2: read synchronously off this cached record by the Location
  // Tile (AD-5 "no redundant fetching") — never re-fetched.
  country?: string
  geoLat?: number
  geoLong?: number
}

// A station-shaped object from elsewhere (e.g. a search result) that hasn't
// been converted into a persisted `Station` yet — `toggleFavorite` accepts
// either this or a full `Station` so callers never have to hand-map fields.
export type FavoritableStation = Omit<Station, 'isFavorite' | 'addedAt' | 'favoriteOrder'> &
  Partial<Pick<Station, 'isFavorite' | 'addedAt' | 'favoriteOrder'>>

// Seed content so the rail isn't empty on a first run, before Search &
// Browse (Story 1.2) gives Javad a way to find and save his own stations.
const DEFAULT_STATIONS: Station[] = [
  { id: 'soma-groove', name: 'SomaFM Groove Salad', url: 'https://ice1.somafm.com/groovesalad-128-mp3', category: 'Ambient', isFavorite: true, addedAt: Date.now(), favoriteOrder: 0 },
  { id: 'soma-drift', name: 'SomaFM Drone Zone', url: 'https://ice1.somafm.com/dronezone-128-mp3', category: 'Ambient', isFavorite: true, addedAt: Date.now(), favoriteOrder: 1 },
  { id: 'paradise-main', name: 'Radio Paradise Main', url: 'https://stream.radioparadise.com/mp3-128', category: 'Eclectic', isFavorite: true, addedAt: Date.now(), favoriteOrder: 2 },
  { id: 'kexp', name: 'KEXP Seattle', url: 'https://live-aacplus-64.kexp.org/kexp64.aac', category: 'Indie', isFavorite: true, addedAt: Date.now(), favoriteOrder: 3 },
]

// Pure helpers (exported for unit testing) — kept free of Pinia/store state
// so `moveUp`/`moveDown`/`toggleFavorite` logic can be verified without
// mounting the store.
export const sortByFavoriteOrder = (list: Station[]): Station[] =>
  [...list].sort((a, b) => a.favoriteOrder - b.favoriteOrder)

// Detects a `favoriteOrder` migration collision — 2+ stations sharing the
// same value, which is exactly what happens to *every* station in a
// `store.json` written before this field existed (Rust's
// `#[serde(default)]` backfills a bare `0` for all of them) — and rewrites
// distinct sequential values from the stations' existing array order.
// Without this, two tied stations have the same `favoriteOrder` *value*, so
// `swapFavoriteOrder` "swapping" them is a no-op: it exchanges `0` for `0`
// (code review finding #1). A list with no collision is returned as-is.
export const normalizeFavoriteOrder = (list: Station[]): Station[] => {
  const seen = new Set<number>()
  const hasCollision = list.some((s) => {
    if (seen.has(s.favoriteOrder)) return true
    seen.add(s.favoriteOrder)
    return false
  })
  if (!hasCollision) return list
  return list.map((s, index) => ({ ...s, favoriteOrder: index }))
}

export const swapFavoriteOrder = (list: Station[], id: string, direction: 'up' | 'down'): Station[] => {
  // Backfill first so a migrated (all-tied) list gets genuinely distinct
  // values to swap between, not just its own value swapped with itself.
  const normalized = normalizeFavoriteOrder(list)
  const sorted = sortByFavoriteOrder(normalized)
  const index = sorted.findIndex((s) => s.id === id)
  if (index === -1) return list

  const targetIndex = direction === 'up' ? index - 1 : index + 1
  if (targetIndex < 0 || targetIndex >= sorted.length) return normalized // boundary: no swap, but keep the backfilled order

  const a = sorted[index]
  const b = sorted[targetIndex]
  const aOrder = a.favoriteOrder
  const bOrder = b.favoriteOrder

  return normalized.map((s) => {
    if (s.id === a.id) return { ...s, favoriteOrder: bOrder }
    if (s.id === b.id) return { ...s, favoriteOrder: aOrder }
    return s
  })
}

const nextFavoriteOrder = (list: Station[]): number =>
  list.reduce((max, s) => Math.max(max, s.favoriteOrder), -1) + 1

export const useStationsStore = defineStore('stations', () => {
  const stations = ref<Station[]>([])
  const loaded = ref(false)

  // Render order is always `favoriteOrder` ascending, never raw array/insert
  // position (Boundaries & Constraints: order is explicit, not positional).
  const sortedStations = computed(() => sortByFavoriteOrder(stations.value))

  const loadStations = async () => {
    try {
      const saved = await invoke<Station[]>('list_stations')
      stations.value = normalizeFavoriteOrder(saved.length > 0 ? saved : DEFAULT_STATIONS)
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

  // Row click plays immediately (FR-4) — no select-then-play step.
  const playStation = (station: Station) => {
    const playbackStore = usePlaybackStore()
    playbackStore.play(station)
  }

  // Membership check driven by the store's current state, not a caller's
  // possibly-stale local copy (I/O matrix: "stale/racing isFavorite").
  const isFavorite = (id: string) => stations.value.some((s) => s.id === id)

  // Accepts either a persisted `Station` (Favorites row) or a
  // search-result-shaped station (Search row) — converts+appends on add,
  // removes on already-present. Reuses `save_stations` for persistence
  // (Boundaries & Constraints: no new Tauri command).
  const toggleFavorite = async (candidate: FavoritableStation) => {
    if (isFavorite(candidate.id)) {
      stations.value = stations.value.filter((s) => s.id !== candidate.id)
    } else {
      const station: Station = {
        id: candidate.id,
        name: candidate.name,
        url: candidate.url,
        faviconUrl: candidate.faviconUrl,
        homepage: candidate.homepage,
        category: candidate.category,
        isFavorite: true,
        addedAt: candidate.addedAt ?? Date.now(),
        favoriteOrder: nextFavoriteOrder(stations.value),
        country: candidate.country,
        geoLat: candidate.geoLat,
        geoLong: candidate.geoLong,
      }
      stations.value = [...stations.value, station]
    }
    await saveStations()
  }

  const moveUp = async (id: string) => {
    stations.value = swapFavoriteOrder(stations.value, id, 'up')
    await saveStations()
  }

  const moveDown = async (id: string) => {
    stations.value = swapFavoriteOrder(stations.value, id, 'down')
    await saveStations()
  }

  return {
    stations,
    sortedStations,
    loaded,
    loadStations,
    saveStations,
    playStation,
    isFavorite,
    toggleFavorite,
    moveUp,
    moveDown,
  }
})
