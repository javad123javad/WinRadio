<template>
  <div class="flex items-center gap-4 rounded-sm bg-surface-raised px-5 py-3.5">
    <button
      type="button"
      class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full border border-primary text-primary transition-colors hover:bg-primary/10 disabled:cursor-not-allowed disabled:opacity-50"
      :disabled="!canTogglePlay"
      :aria-label="isPlaying ? 'Pause' : 'Play'"
      @click="togglePlay"
    >
      <svg v-if="isPlaying" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
        <path d="M6 5h4v14H6zm8 0h4v14h-4z" />
      </svg>
      <svg v-else class="h-4 w-4 translate-x-0.5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8 5v14l11-7z" />
      </svg>
    </button>

    <span
      v-if="playback.reconnecting"
      class="w-32 flex-shrink-0 text-caption text-on-surface-variant"
    >Reconnecting&hellip;</span>
    <span
      v-else-if="playback.errorMessage"
      class="w-40 flex-shrink-0 text-caption text-error"
    >{{ playback.errorMessage }}</span>
    <span v-else-if="playback.isPlaying" class="w-16 flex-shrink-0 text-caption text-on-surface-variant">Playing</span>
    <span v-else class="w-16 flex-shrink-0 text-caption text-on-surface-variant">Idle</span>

    <div class="flex flex-1 items-center gap-3">
      <button
        type="button"
        class="flex-shrink-0 text-on-surface transition-colors"
        :class="{ 'text-primary': playback.isMuted }"
        :aria-label="playback.isMuted ? 'Unmute' : 'Mute'"
        @click="playback.toggleMute()"
      >
        <svg v-if="playback.isMuted" class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.42.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06a8.99 8.99 0 003.69-1.81L18.73 21 20 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z" />
        </svg>
        <svg v-else class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z" />
        </svg>
      </button>

      <input
        type="range"
        min="0"
        max="100"
        :value="volumePercent"
        aria-label="Volume"
        class="h-1 w-full max-w-[220px] cursor-pointer appearance-none rounded-full bg-outline accent-primary"
        @input="onVolumeInput"
        @change="onVolumeCommit"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { usePlaybackStore } from '@/stores/playback'

const playback = usePlaybackStore()

const isPlaying = computed(() => playback.isPlaying)
const canTogglePlay = computed(() => !playback.reconnecting && (playback.isPlaying || !!playback.currentStation))

const volumePercent = computed(() => Math.round(playback.volume * 100))

const onVolumeInput = (event: Event) => {
  const val = Number((event.target as HTMLInputElement).value) / 100
  playback.isMuted = false
  playback.setVolume(val)
}

// Fires once when the user releases the slider (native `change`, not
// `input`) — persists to disk on release, matching FR-12, rather than on
// every drag tick.
const onVolumeCommit = () => {
  playback.persistVolume()
}

const togglePlay = () => {
  if (playback.isPlaying) {
    playback.stop()
  } else if (playback.currentStation) {
    playback.play(playback.currentStation)
  }
}
</script>
