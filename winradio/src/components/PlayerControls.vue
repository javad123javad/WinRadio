<template>
  <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 shadow-lg">
    <div class="flex flex-col sm:flex-row items-center justify-between gap-4">
      <div class="flex items-center gap-4 min-w-0 flex-1">
        <div class="w-14 h-14 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center flex-shrink-0">
          <svg class="w-8 h-8 text-white" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
          </svg>
        </div>
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900 dark:text-white truncate" v-if="currentStation">
            {{ currentStation.name }}
          </p>
          <p class="text-sm font-medium text-gray-500 dark:text-gray-400 truncate" v-else>
            No station selected
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5" v-if="metadata.title">
            {{ metadata.title }}{{ metadata.artist ? ' - ' + metadata.artist : '' }}
          </p>
          <p class="text-xs text-gray-400 dark:text-gray-500 truncate" v-else-if="currentStation">
            {{ currentStation.category || 'Radio' }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-4 flex-shrink-0">
        <div class="flex items-center gap-2 hidden sm:flex">
          <label class="text-xs text-gray-500 dark:text-gray-400">Vol</label>
          <input
            type="range"
            min="0"
            max="100"
            v-model="volume"
            @input="setVolume"
            class="w-32 h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none accent-blue-500 cursor-pointer"
          />
        </div>

        <div class="flex items-center gap-2">
          <button
            @click="previousStation"
            :disabled="!stations.length"
            class="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            aria-label="Previous station"
          >
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/>
            </svg>
          </button>

          <button
            @click="togglePlay"
            :class="isPlaying
              ? 'bg-red-500 hover:bg-red-600 text-white'
              : 'bg-green-500 hover:bg-green-600 text-white'"
            class="p-3 rounded-xl transition-colors shadow-lg"
            aria-label="Play/Stop"
          >
            <svg v-if="isPlaying" class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
            </svg>
            <svg v-else class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z"/>
            </svg>
          </button>

          <button
            @click="nextStation"
            :disabled="!stations.length"
            class="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            aria-label="Next station"
          >
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
            </svg>
          </button>
        </div>

        <div class="flex items-center gap-2 ml-2 sm:ml-4">
          <button
            @click="showSettings = true"
            class="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
            aria-label="Settings"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <div class="mt-4 hidden sm:block" v-if="isPlaying">
      <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
        <span class="w-16 text-right">{{ formatTime(position) }}</span>
        <div class="flex-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div
            class="h-full bg-blue-500 rounded-full transition-all duration-100"
            :style="{ width: progress + '%' }"
          ></div>
        </div>
        <span class="w-16">{{ formatTime(duration) }}</span>
      </div>
    </div>

    <SettingsModal v-model:show="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePlaybackStore } from '@/stores/playback'
import { useStationsStore } from '@/stores/stations'
import SettingsModal from '@/components/SettingsModal.vue'

const playbackStore = usePlaybackStore()
const stationsStore = useStationsStore()

const isPlaying = computed(() => playbackStore.isPlaying)
const currentStation = computed(() => playbackStore.currentStation)
const volume = computed({
  get: () => playbackStore.volume * 100,
  set: (val) => playbackStore.setVolume(val / 100)
})
const metadata = computed(() => playbackStore.metadata)
const position = computed(() => playbackStore.position)
const duration = computed(() => playbackStore.duration)
const progress = computed(() => duration.value > 0 ? (position.value / duration.value) * 100 : 0)
const stations = computed(() => stationsStore.stations)

const showSettings = ref(false)

const togglePlay = () => {
  if (playbackStore.isPlaying) {
    playbackStore.stop()
  } else if (currentStation.value) {
    playbackStore.play(currentStation.value.url)
  }
}

const setVolume = () => {
  playbackStore.setVolume(volume.value / 100)
}

const nextStation = () => stationsStore.playNext()
const previousStation = () => stationsStore.playPrevious()

const formatTime = (seconds: number) => {
  if (!seconds || isNaN(seconds)) return '0:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}
</script>