<template>
  <InfoTile title="Stream Info" :placeholder="placeholderText">
    <div class="flex flex-col items-center gap-1 text-center">
      <p class="text-label-caps uppercase text-on-surface">{{ codecBitrateLine }}</p>
      <p class="text-caption text-on-surface-variant">{{ countryLine }}</p>
      <p class="text-caption text-on-surface-variant">IP {{ ipLine }}</p>
    </div>
  </InfoTile>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import InfoTile from '@/components/InfoTile.vue'
import { usePlaybackStore } from '@/stores/playback'

const playbackStore = usePlaybackStore()

// Codec/bitrate/country render the instant a station is playing, read
// straight off `playbackStore.currentStation` — never wait for any Rust
// event for these three fields (AC1). The shared `InfoTile` placeholder is
// reserved for the true idle case (nothing currently playing); a station
// missing one of these fields renders a per-field blank/dash instead of
// collapsing the whole tile (Boundaries & Constraints).
const placeholderText = computed(() =>
  playbackStore.currentStation ? null : 'Stream info unavailable'
)

const BLANK = '—' // em dash, per-field "missing" fallback

const codecBitrateLine = computed(() => {
  const station = playbackStore.currentStation
  const parts = [station?.codec, station?.bitrate ? `${station.bitrate}kbps` : undefined].filter(
    (part): part is string => Boolean(part)
  )
  return parts.length > 0 ? parts.join(' · ') : BLANK
})

const countryLine = computed(() => playbackStore.currentStation?.country ?? BLANK)

// The IP is the tile's only fetched field: idle until `stream-info-updated`
// arrives (never a loading flash on the other three fields, but this one
// genuinely has nothing to show yet), then either the resolved IP or the
// literal "unavailable" on a DNS failure — independent of the other fields,
// and never a `playback-error` (Boundaries & Constraints).
const ipLine = computed(() => {
  if (playbackStore.streamInfo.status === 'ok') return playbackStore.streamInfo.ip
  if (playbackStore.streamInfo.status === 'unavailable') return 'unavailable'
  return BLANK
})
</script>
