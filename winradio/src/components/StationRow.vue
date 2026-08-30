<template>
  <li class="mb-1.5">
    <!-- Not a native <button>: it hosts real nested <button>s (reorder,
         favorite) which HTML forbids inside a <button>. role="button" +
         tabindex + @keydown keep it equivalently keyboard-operable — both
         Enter and Space activate it, matching native <button> semantics
         (EXPERIENCE.md: Enter activates the focused station row). Each
         nested button below stops keydown from bubbling here (`@keydown.stop`,
         alongside its existing `@click.stop`), so pressing Enter/Space to
         activate the star or a reorder arrow doesn't *also* fire `play` on
         the row underneath it. -->
    <div
      role="button"
      tabindex="0"
      class="flex h-14 w-full cursor-pointer items-center gap-2 rounded-sm px-2.5 text-left transition-colors"
      :class="isCurrent ? 'bg-surface-raised-high' : 'bg-surface-raised hover:bg-surface-raised-high'"
      @click="$emit('play')"
      @keydown.enter="$emit('play')"
      @keydown.space.prevent="$emit('play')"
    >
      <!-- Reorder controls: Favorites rows only, always visible (not
           hover-reveal), separate click target from row-click-to-play. -->
      <span v-if="showReorder" class="flex flex-shrink-0 flex-col">
        <button
          type="button"
          class="flex h-4 w-4 items-center justify-center leading-none text-on-surface-variant hover:text-on-surface disabled:cursor-default disabled:opacity-30"
          :disabled="!canMoveUp"
          aria-label="Move up"
          @click.stop="$emit('move-up')"
          @keydown.stop
        >
          <svg class="h-2.5 w-2.5" viewBox="0 0 24 24" fill="currentColor"><path d="M12 6l7 10H5z" /></svg>
        </button>
        <button
          type="button"
          class="flex h-4 w-4 items-center justify-center leading-none text-on-surface-variant hover:text-on-surface disabled:cursor-default disabled:opacity-30"
          :disabled="!canMoveDown"
          aria-label="Move down"
          @click.stop="$emit('move-down')"
          @keydown.stop
        >
          <svg class="h-2.5 w-2.5" viewBox="0 0 24 24" fill="currentColor"><path d="M12 18L5 8h14z" /></svg>
        </button>
      </span>

      <span class="flex h-[22px] w-[22px] flex-shrink-0 items-center justify-center rounded-full border border-secondary text-secondary">
        <svg v-if="isCurrent && isPlaying" class="h-2.5 w-2.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 5h4v14H6zm8 0h4v14h-4z" />
        </svg>
        <svg v-else class="h-2.5 w-2.5 translate-x-px" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z" />
        </svg>
      </span>
      <span class="min-w-0 flex-1 truncate text-label-link text-secondary">{{ name }}</span>

      <!-- Favorite star: separate click target from the row's play click,
           always visible on every row (Favorites and Search). -->
      <button
        type="button"
        class="flex h-6 w-6 flex-shrink-0 items-center justify-center text-lg leading-none"
        :class="isFavorite ? 'text-primary' : 'text-on-surface-variant'"
        :aria-label="isFavorite ? 'Remove from favorites' : 'Add to favorites'"
        :aria-pressed="isFavorite"
        @click.stop="$emit('toggle-favorite')"
        @keydown.stop
      >
        <span aria-hidden="true">{{ isFavorite ? '★' : '☆' }}</span>
      </button>
    </div>
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
// row (`name`) plus caller-computed `isCurrent`/`isPlaying`/`isFavorite`/
// reorder-boundary flags, so it works identically for a persisted `Station`
// (Favorites) and a `DirectoryStation` (Search results) — two distinct
// shapes the caller reconciles, not this component.
withDefaults(
  defineProps<{
    name: string
    isCurrent: boolean
    isPlaying: boolean
    isFavorite: boolean
    // Reorder controls are Favorites-only (Boundaries & Constraints: never
    // on Search rows) — canMoveUp/canMoveDown only matter when true.
    showReorder?: boolean
    canMoveUp?: boolean
    canMoveDown?: boolean
  }>(),
  {
    showReorder: false,
    canMoveUp: false,
    canMoveDown: false,
  },
)

defineEmits<{
  play: []
  'toggle-favorite': []
  'move-up': []
  'move-down': []
}>()
</script>
