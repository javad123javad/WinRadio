<template>
  <InfoTile title="Location" :placeholder="placeholderText">
    <div class="flex flex-col items-center gap-2 text-center">
      <p class="text-label-caps uppercase text-on-surface">{{ playbackStore.location.country }}</p>
      <img
        :src="playbackStore.location.tileImage!"
        alt="Map centered on the station's location"
        class="h-24 w-24 rounded-sm object-cover"
      />
      <p class="text-caption text-on-surface-variant">&copy; OpenStreetMap contributors</p>
    </div>
  </InfoTile>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import InfoTile from '@/components/InfoTile.vue'
import { usePlaybackStore } from '@/stores/playback'

const playbackStore = usePlaybackStore()

// Full content (country + map + attribution) only renders once the tile
// image has actually arrived — never country-with-no-map, per the frozen
// "Tile fetch failure and no coordinates both resolve to the exact same
// 'Location unknown' placeholder — never a partial state" constraint. Idle
// (nothing has ever played) gets the same quiet placeholder as the two
// genuine failure cases; no distinct "loading" copy is specified.
const placeholderText = computed(() =>
  playbackStore.location.status === 'ok' && playbackStore.location.tileImage ? null : 'Location unknown'
)
</script>
