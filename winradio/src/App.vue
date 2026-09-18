<template>
  <div class="flex min-h-screen flex-col bg-surface-base">
    <nav class="flex gap-4 px-5 pb-2 pt-4" aria-label="Primary">
      <button
        type="button"
        class="flex w-14 flex-col items-center gap-1"
        aria-label="Favorites"
        :aria-pressed="activeView === 'favorites'"
        @click="activeView = 'favorites'"
      >
        <span
          class="flex h-12 w-12 items-center justify-center rounded-full border text-on-surface"
          :class="activeView === 'favorites' ? 'border-primary bg-primary/20' : 'border-outline'"
        >
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z" />
          </svg>
        </span>
        <span class="text-caption text-on-surface-variant">Favorites</span>
      </button>

      <button
        type="button"
        class="flex w-14 flex-col items-center gap-1"
        aria-label="Filter"
        :aria-pressed="activeView === 'search' && showFilterControls"
        @click="onFilterClick"
      >
        <span
          class="flex h-12 w-12 items-center justify-center rounded-full border text-on-surface"
          :class="activeView === 'search' && showFilterControls ? 'border-primary bg-primary/20' : 'border-outline'"
        >
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 5h16M7 12h10M10 19h4" />
          </svg>
        </span>
        <span class="text-caption text-on-surface-variant">Filter</span>
      </button>

      <button
        type="button"
        class="flex w-14 flex-col items-center gap-1"
        aria-label="Search"
        :aria-pressed="activeView === 'search'"
        @click="activeView = 'search'"
      >
        <span
          class="flex h-12 w-12 items-center justify-center rounded-full border text-on-surface"
          :class="activeView === 'search' ? 'border-primary bg-primary/20' : 'border-outline'"
        >
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
            <circle cx="11" cy="11" r="6" />
            <path stroke-linecap="round" d="M20 20l-4.35-4.35" />
          </svg>
        </span>
        <span class="text-caption text-on-surface-variant">Search</span>
      </button>

      <button
        type="button"
        class="flex w-14 flex-col items-center gap-1"
        aria-label="Settings"
        @click="showSettings = true"
      >
        <span class="flex h-12 w-12 items-center justify-center rounded-full border border-outline text-on-surface">
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6">
            <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
        </span>
        <span class="text-caption text-on-surface-variant">Settings</span>
      </button>
    </nav>

    <div class="flex flex-1 items-start">
      <aside class="w-rail flex-shrink-0 px-4 pb-4">
        <!-- Only one of Favorites/Search is the active rail content at a
             time (Boundaries & Constraints) — same rail, same StationRow,
             content source swaps underneath. -->
        <template v-if="activeView === 'favorites'">
          <h2 class="mb-3 text-body font-semibold text-on-surface">Favorites ({{ stationsStore.stations.length }})</h2>

          <p
            v-if="stationsStore.loaded && stationsStore.stations.length === 0"
            class="text-caption text-on-surface-variant"
          >
            No favorites yet — search to find a station.
          </p>

          <ul>
            <StationRow
              v-for="(station, index) in stationsStore.sortedStations"
              :key="station.id"
              :name="station.name"
              :is-current="isCurrent(station.id)"
              :is-playing="playbackStore.isPlaying"
              :is-favorite="true"
              :show-reorder="true"
              :can-move-up="index > 0"
              :can-move-down="index < stationsStore.sortedStations.length - 1"
              @play="stationsStore.playStation(station)"
              @toggle-favorite="stationsStore.toggleFavorite(station)"
              @move-up="stationsStore.moveUp(station.id)"
              @move-down="stationsStore.moveDown(station.id)"
            />
          </ul>
        </template>

        <template v-else>
          <h2 class="mb-3 text-body font-semibold text-on-surface">
            Search{{ searchStore.status === 'ok' ? ` (${searchStore.results.length})` : '' }}
          </h2>

          <SearchPanel :show-filter-controls="showFilterControls" />

          <ul v-if="searchStore.results.length > 0">
            <StationRow
              v-for="station in searchStore.results"
              :key="station.id"
              :name="station.name"
              :is-current="isCurrent(station.id)"
              :is-playing="playbackStore.isPlaying"
              :is-favorite="stationsStore.isFavorite(station.id)"
              :show-reorder="false"
              @play="searchStore.playResult(station)"
              @toggle-favorite="stationsStore.toggleFavorite(toPlayableStation(station))"
            />
          </ul>
          <p v-else-if="searchStore.status === 'loading'" class="text-caption text-on-surface-variant">
            Searching…
          </p>
          <p v-else-if="searchStore.status === 'zero-match'" class="text-caption text-on-surface-variant">
            No stations found for '{{ searchStore.lastQuery }}'.
          </p>
          <p v-else-if="searchStore.status === 'offline'" class="text-caption text-on-surface-variant">
            {{ searchStore.errorMessage }}
          </p>
          <p v-else-if="searchStore.status === 'idle'" class="text-caption text-on-surface-variant">
            Type a query or choose a filter to search stations.
          </p>
        </template>
      </aside>

      <main class="flex flex-1 flex-col gap-4 p-5">
        <section class="rounded-sm bg-surface-raised p-6 text-center">
          <div class="mx-auto mb-4 h-24 w-24 rounded-full bg-surface-raised-high"></div>

          <p v-if="playbackStore.currentStation" class="text-display text-on-surface">
            {{ playbackStore.currentStation.name }}
          </p>
          <p v-else class="text-display text-on-surface-variant">No station selected</p>

          <!-- No ICY metadata is normal, not an error state — the subtext
               is simply blank rather than a category/generic-label
               fallback (spec-2-1, AC2). -->
          <p v-if="playbackStore.metadata.title" class="mt-1 text-body text-on-surface-variant">
            {{ playbackStore.metadata.title }}<span v-if="playbackStore.metadata.artist"> — {{ playbackStore.metadata.artist }}</span>
          </p>
        </section>

        <TransportBar />

        <!-- Info Tile grid: Location (spec-2-2) and Weather (spec-2-3) are
             wired up; Stream Info stays a reserved-but-empty div until
             Story 2.4. -->
        <section class="grid grid-cols-2 gap-4 tiles:grid-cols-3">
          <LocationTile />
          <WeatherTile />
          <InfoTile v-for="tile in remainingInfoTiles" :key="tile" :title="tile" />
        </section>
      </main>
    </div>

    <SettingsModal :show="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import TransportBar from '@/components/TransportBar.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import StationRow from '@/components/StationRow.vue'
import SearchPanel from '@/components/SearchPanel.vue'
import LocationTile from '@/components/LocationTile.vue'
import WeatherTile from '@/components/WeatherTile.vue'
import InfoTile from '@/components/InfoTile.vue'
import { useStationsStore } from '@/stores/stations'
import { usePlaybackStore } from '@/stores/playback'
import { useSettingsStore } from '@/stores/settings'
import { useSearchStore, toPlayableStation } from '@/stores/search'

const stationsStore = useStationsStore()
const playbackStore = usePlaybackStore()
const settingsStore = useSettingsStore()
const searchStore = useSearchStore()

const showSettings = ref(false)
// Location (spec-2-2) and Weather (spec-2-3) are now `<LocationTile />` /
// `<WeatherTile />`; Stream Info stays a reserved, empty placeholder div
// until Story 2.4 fills it in.
const remainingInfoTiles = ['Stream Info']

// Rail content swap (Code Map) — Favorites is the default/landing view
// (DESIGN.md nav-icon-button note: WinRadio has no separate "Home").
const activeView = ref<'favorites' | 'search'>('favorites')
// Filter is a refinement of whichever rail content is showing, not its own
// destination (Boundaries & Constraints) — it only makes sense against
// Search results, so clicking it switches to Search and reveals/hides the
// filter selects within `SearchPanel`.
const showFilterControls = ref(false)
const onFilterClick = () => {
  activeView.value = 'search'
  showFilterControls.value = !showFilterControls.value
}

const isCurrent = (id: string) => playbackStore.currentStation?.id === id

let systemDarkQuery: MediaQueryList | undefined

// `Space` toggles play/pause when the window has focus and no text field is
// focused (EXPERIENCE.md Accessibility Floor / Interaction Primitives).
const isTextInputFocused = () => {
  const el = document.activeElement
  if (!el) return false
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.hasAttribute('contenteditable')
}

const onGlobalKeydown = (event: KeyboardEvent) => {
  if (event.code !== 'Space' || isTextInputFocused()) return
  event.preventDefault()
  if (playbackStore.isPlaying) {
    playbackStore.stop()
  } else if (playbackStore.currentStation) {
    playbackStore.play(playbackStore.currentStation)
  }
}

const applyTheme = () => {
  // Dark is the default surface (DESIGN.md); "system" only switches to
  // light when the OS explicitly prefers it.
  const isLight =
    settingsStore.theme === 'light' ||
    (settingsStore.theme === 'system' && systemDarkQuery?.matches === false)
  document.documentElement.classList.toggle('light', isLight)
}

watch(() => settingsStore.theme, applyTheme)

onMounted(async () => {
  systemDarkQuery = window.matchMedia('(prefers-color-scheme: dark)')
  systemDarkQuery.addEventListener('change', applyTheme)
  window.addEventListener('keydown', onGlobalKeydown)

  // Listeners first so no event is missed once loading kicks off playback.
  // `initListeners` now rejects (rather than silently no-op'ing forever) if
  // registration partially fails, so it doesn't get to permanently block
  // the rest of startup here — loading stations/settings/volume should
  // still proceed even if this hiccupped.
  try {
    await playbackStore.initListeners()
  } catch {
    // Already logged inside the store; startup continues regardless.
  }

  await Promise.all([
    stationsStore.loadStations(),
    settingsStore.loadSettings(),
    playbackStore.loadVolume(),
  ])

  // AC4: show the last-played station's info idle (not auto-playing) once
  // settings have resolved — `restoreLastStation` only sets `currentStation`,
  // never `isPlaying`/`play()` (spec-1-5).
  playbackStore.restoreLastStation(settingsStore.lastStation)

  applyTheme()
})
</script>
