<template>
  <div class="rounded-sm bg-surface-raised p-3">
    <h3 class="text-label-caps uppercase text-on-surface-variant">{{ title }}</h3>
    <div class="mt-2">
      <!-- Same shell either way (DESIGN.md {components.info-tile}, PRD
           NFR-3): a degraded tile never gets a different background/radius/
           padding, just muted placeholder text in place of the slot. -->
      <p v-if="placeholder" class="text-caption text-on-surface-variant">{{ placeholder }}</p>
      <slot v-else />
    </div>
  </div>
</template>

<script setup lang="ts">
// Shared shell for all three Info Tiles (Location/Weather/Stream Info,
// Stories 2.2/2.3/2.4) — title + either the populated default slot or a
// quiet placeholder for the degraded state. Never a red/error style; never
// a blank void (DESIGN.md Do's and Don'ts).
defineProps<{
  title: string
  placeholder?: string | null
}>()
</script>
