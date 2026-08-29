<template>
  <div class="mb-3">
    <input
      type="text"
      class="w-full rounded-sm border border-outline bg-surface-raised px-3 py-2 text-body text-on-surface placeholder:text-on-surface-variant focus:border-primary focus:outline-none"
      :class="{ 'border-primary': isFocused }"
      placeholder="Search stations…"
      aria-label="Search stations"
      :value="searchStore.query"
      @input="onQueryInput"
      @keydown.enter="searchStore.submitQuery()"
      @focus="isFocused = true"
      @blur="isFocused = false"
    />

    <!-- Filter control (EXPERIENCE.md: "a small panel, not a full page") —
         toggled by the Filter nav icon in App.vue, alongside the field
         above rather than replacing it. -->
    <div v-if="showFilterControls" class="mt-2 flex flex-col gap-2">
      <select
        aria-label="Genre filter"
        class="w-full rounded-sm border border-outline bg-surface-raised-high px-2.5 py-1.5 text-caption text-on-surface"
        :value="searchStore.filters.genre"
        @change="onFilterChange('genre', $event)"
      >
        <option value="">Genre (any)</option>
        <option v-for="genre in genres" :key="genre" :value="genre">{{ genre }}</option>
      </select>

      <select
        aria-label="Country filter"
        class="w-full rounded-sm border border-outline bg-surface-raised-high px-2.5 py-1.5 text-caption text-on-surface"
        :value="searchStore.filters.country"
        @change="onFilterChange('country', $event)"
      >
        <option value="">Country (any)</option>
        <option v-for="country in countries" :key="country" :value="country">{{ country }}</option>
      </select>

      <select
        aria-label="Language filter"
        class="w-full rounded-sm border border-outline bg-surface-raised-high px-2.5 py-1.5 text-caption text-on-surface"
        :value="searchStore.filters.language"
        @change="onFilterChange('language', $event)"
      >
        <option value="">Language (any)</option>
        <option v-for="language in languages" :key="language" :value="language">{{ language }}</option>
      </select>
    </div>

    <!-- Removable chips (`DESIGN.md.components.filter-chip`) — visible
         whenever a filter is active, independent of whether the select
         panel above is currently expanded, so it's always clear *what's*
         filtering the rail (EXPERIENCE.md Filter control). -->
    <div v-if="activeChips.length > 0" class="mt-2 flex flex-wrap gap-1.5">
      <span
        v-for="chip in activeChips"
        :key="chip.key"
        class="flex items-center gap-1 rounded-full bg-surface-raised-high px-2.5 py-1 text-caption text-on-surface-variant"
      >
        <span>{{ chip.label }}: <span class="text-on-surface">{{ chip.value }}</span></span>
        <button
          type="button"
          class="text-on-surface-variant hover:text-on-surface"
          :aria-label="`Remove ${chip.label} filter`"
          @click="searchStore.clearFilter(chip.key)"
        >
          ✕
        </button>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useSearchStore, type FilterKey } from '@/stores/search'

defineProps<{ showFilterControls: boolean }>()

const searchStore = useSearchStore()
const isFocused = ref(false)

// Lazily loaded once, the first time the Search rail is shown (Code Map:
// "filterOptions (lazy-loaded once)") — never a hardcoded static list.
onMounted(() => {
  searchStore.loadFilterOptions()
})

const genres = computed(() => searchStore.filterOptions?.genres ?? [])
const countries = computed(() => searchStore.filterOptions?.countries ?? [])
const languages = computed(() => searchStore.filterOptions?.languages ?? [])

const onQueryInput = (event: Event) => {
  searchStore.setQuery((event.target as HTMLInputElement).value)
}

const onFilterChange = (key: FilterKey, event: Event) => {
  searchStore.setFilter(key, (event.target as HTMLSelectElement).value)
}

const activeChips = computed<{ key: FilterKey; label: string; value: string }[]>(() => {
  const chips: { key: FilterKey; label: string; value: string }[] = []
  if (searchStore.filters.genre) chips.push({ key: 'genre', label: 'Genre', value: searchStore.filters.genre })
  if (searchStore.filters.country) chips.push({ key: 'country', label: 'Country', value: searchStore.filters.country })
  if (searchStore.filters.language) chips.push({ key: 'language', label: 'Language', value: searchStore.filters.language })
  return chips
})
</script>
