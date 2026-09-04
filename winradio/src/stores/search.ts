import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { usePlaybackStore } from './playback'
import type { Station } from './stations'

// Search-result stations from Radio-Browser — deliberately distinct from
// the persisted `Station` (Favorites) shape returned by `list_stations`
// (spec Intent: "search-result stations distinct from persisted Stations").
export interface DirectoryStation {
  id: string
  name: string
  url: string
  favicon?: string
  tags?: string
  country?: string
  language?: string
  codec?: string
  bitrate?: number
  geoLat?: number
  geoLong?: number
}

export interface FilterOptions {
  genres: string[]
  countries: string[]
  languages: string[]
}

export type FilterKey = 'genre' | 'country' | 'language'

export type SearchStatus = 'idle' | 'loading' | 'ok' | 'zero-match' | 'offline'

// No explicit "Go" button anywhere (Boundaries & Constraints -> Never) — a
// typed query debounces at this interval; Enter (or a filter select change)
// bypasses the debounce and searches immediately.
const DEBOUNCE_MS = 350

// Adapts a search-result `DirectoryStation` into the persisted `Station`
// shape — used both to play a result ephemerally (never persisted) and to
// let `stationsStore.toggleFavorite` accept a `DirectoryStation` directly,
// without App.vue doing ad-hoc field mapping (Code Map). Module-level (not
// store state) since it's a pure shape conversion.
export const toPlayableStation = (station: DirectoryStation): Station => ({
  id: station.id,
  name: station.name,
  url: station.url,
  faviconUrl: station.favicon,
  homepage: undefined,
  category: station.tags?.split(',')[0]?.trim() || station.country,
  isFavorite: false,
  addedAt: Date.now(),
  favoriteOrder: 0,
  // spec-2-2: carried through so the Location Tile still has coordinates
  // when a search result (never persisted with the rest of the station
  // shape) is played or favorited.
  country: station.country,
  geoLat: station.geoLat,
  geoLong: station.geoLong,
})

export const useSearchStore = defineStore('search', () => {
  const query = ref('')
  const filters = ref<{ genre: string; country: string; language: string }>({
    genre: '',
    country: '',
    language: '',
  })
  const results = ref<DirectoryStation[]>([])
  const status = ref<SearchStatus>('idle')
  // The query the current `status`/`results` actually reflect — kept
  // separate from the live `query` ref so a "No stations found for 'x'"
  // message never flashes the wrong (still-being-typed) text.
  const lastQuery = ref('')
  // The exact message the backend's `Err(...)` carried (verbatim — Rust's
  // `CONNECT_ERROR` is the single source of truth for this copy, never
  // duplicated as a separate hardcoded string on the frontend).
  const errorMessage = ref('')
  const filterOptions = ref<FilterOptions | null>(null)
  const filterOptionsLoading = ref(false)

  let debounceHandle: ReturnType<typeof setTimeout> | undefined
  // AD-13: never re-issue a network call for an unchanged
  // {query, genre, country, language} tuple — tracked across both debounced
  // and immediate (Enter) triggers.
  let lastSignature: string | null = null
  let requestSeq = 0

  const signatureOf = () =>
    JSON.stringify({
      query: query.value.trim(),
      genre: filters.value.genre,
      country: filters.value.country,
      language: filters.value.language,
    })

  const hasAnyCriteria = () =>
    query.value.trim().length > 0 ||
    !!filters.value.genre ||
    !!filters.value.country ||
    !!filters.value.language

  const executeSearch = async () => {
    const signature = signatureOf()
    if (signature === lastSignature) return
    lastSignature = signature

    if (!hasAnyCriteria()) {
      // Nothing to search for (empty query, no filters) — ephemeral idle
      // state, no network call.
      results.value = []
      status.value = 'idle'
      lastQuery.value = ''
      return
    }

    const trimmedQuery = query.value.trim()
    lastQuery.value = trimmedQuery
    status.value = 'loading'
    const seq = ++requestSeq

    try {
      const stations = await invoke<DirectoryStation[]>('search_stations', {
        name: trimmedQuery || undefined,
        genre: filters.value.genre || undefined,
        country: filters.value.country || undefined,
        language: filters.value.language || undefined,
      })
      if (seq !== requestSeq) return // superseded by a newer search
      results.value = stations
      status.value = stations.length === 0 ? 'zero-match' : 'ok'
    } catch (e) {
      if (seq !== requestSeq) return
      console.error('Search failed:', e)
      results.value = []
      status.value = 'offline'
      // A Tauri command's `Err(String)` rejects `invoke()` with that exact
      // string — display it verbatim rather than re-deriving a copy here.
      errorMessage.value = typeof e === 'string' ? e : "Can't reach the station directory — check your connection"
      // Un-mark this tuple as "already tried": a transient offline failure
      // must not permanently block retrying the identical query once
      // reconnected — without this, the signature guard above would
      // silently no-op every later retry of the same {query, filters}
      // forever (found in code review).
      lastSignature = null
    }
  }

  const runSearch = (options: { immediate?: boolean } = {}) => {
    if (debounceHandle) {
      clearTimeout(debounceHandle)
      debounceHandle = undefined
    }
    if (options.immediate) {
      void executeSearch()
      return
    }
    debounceHandle = setTimeout(() => {
      debounceHandle = undefined
      void executeSearch()
    }, DEBOUNCE_MS)
  }

  const setQuery = (value: string) => {
    query.value = value
    runSearch()
  }

  // Enter submits immediately — no debounce (Boundaries & Constraints).
  const submitQuery = () => {
    runSearch({ immediate: true })
  }

  // Discrete select changes apply immediately too; only free-typed text
  // needs debouncing.
  const setFilter = (key: FilterKey, value: string) => {
    filters.value[key] = value
    runSearch({ immediate: true })
  }

  const clearFilter = (key: FilterKey) => setFilter(key, '')

  const loadFilterOptions = async () => {
    if (filterOptions.value || filterOptionsLoading.value) return
    filterOptionsLoading.value = true
    try {
      filterOptions.value = await invoke<FilterOptions>('get_filter_options')
    } catch (e) {
      console.error('Load filter options failed:', e)
    } finally {
      filterOptionsLoading.value = false
    }
  }

  const playResult = (station: DirectoryStation) => {
    const playbackStore = usePlaybackStore()
    playbackStore.play(toPlayableStation(station))
  }

  return {
    query,
    filters,
    results,
    status,
    lastQuery,
    errorMessage,
    filterOptions,
    setQuery,
    submitQuery,
    setFilter,
    clearFilter,
    loadFilterOptions,
    playResult,
  }
})
