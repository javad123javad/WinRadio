<template>
  <li class="mb-1.5">
    <button
      type="button"
      class="flex h-14 w-full items-center gap-2 rounded-sm px-2.5 text-left transition-colors"
      :class="isCurrent ? 'bg-surface-raised-high' : 'bg-surface-raised hover:bg-surface-raised-high'"
      @click="$emit('play')"
    >
      <span class="flex h-[22px] w-[22px] flex-shrink-0 items-center justify-center rounded-full border border-secondary text-secondary">
        <svg v-if="isCurrent && isPlaying" class="h-2.5 w-2.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 5h4v14H6zm8 0h4v14h-4z" />
        </svg>
        <svg v-else class="h-2.5 w-2.5 translate-x-px" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z" />
        </svg>
      </span>
      <span class="truncate text-label-link text-secondary">{{ name }}</span>
    </button>
  </li>
</template>

<script setup lang="ts">
// Shared row for the Station List rail (`DESIGN.md.components.station-row`,
// EXPERIENCE.md's "same rail, same row" pattern) — extracted from App.vue's
// original inline Favorites `<ul>` markup so Search results (Story 1.2)
// reuse the exact same click-to-play row Story 1.1 built, without any
// search-specific click-to-play wiring (Boundaries & Constraints -> Never).
//
// Deliberately generic: takes only the bits of a station needed to render a
// row (`name`) plus caller-computed `isCurrent`/`isPlaying`, so it works
// identically for a persisted `Station` (Favorites) and a `DirectoryStation`
// (Search results) — two distinct shapes the caller reconciles, not this
// component.
defineProps<{
  name: string
  isCurrent: boolean
  isPlaying: boolean
}>()

defineEmits<{ play: [] }>()
</script>
