<template>
  <InfoTile title="Weather" :placeholder="placeholderText">
    <div class="flex flex-col items-center gap-2 text-center">
      <p class="text-display text-on-surface">{{ Math.round(playbackStore.weather.temperatureC!) }}&deg;C</p>
      <p class="text-label-caps uppercase text-on-surface-variant">{{ playbackStore.weather.condition }}</p>
      <p class="text-caption text-on-surface-variant">
        H: {{ Math.round(playbackStore.weather.forecastHighC!) }}&deg; L: {{ Math.round(playbackStore.weather.forecastLowC!) }}&deg;
      </p>
    </div>
  </InfoTile>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import InfoTile from '@/components/InfoTile.vue'
import { usePlaybackStore } from '@/stores/playback'

const playbackStore = usePlaybackStore()

// Full content only renders once the event has actually arrived with a
// populated payload — idle (nothing has ever played) and both genuine
// failure cases (no coordinates / fetch failed) all resolve to the same
// quiet "Weather unavailable" placeholder, mirroring `LocationTile.vue`.
const placeholderText = computed(() =>
  playbackStore.weather.status === 'ok' ? null : 'Weather unavailable'
)
</script>
