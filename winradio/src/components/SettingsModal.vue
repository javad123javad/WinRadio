<template>
  <Transition name="fade">
    <div
      v-if="show"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
      @click.self="close"
    >
      <div class="w-full max-w-md rounded-md bg-surface-raised">
        <div class="flex items-center justify-between border-b border-outline px-6 py-4">
          <h2 class="text-headline text-on-surface">Settings</h2>
          <button type="button" class="p-1 text-on-surface-variant hover:text-on-surface" aria-label="Close settings" @click="close">
            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="divide-y divide-outline px-6">
          <label class="flex items-center justify-between py-4">
            <span class="text-body text-on-surface">Minimize to tray on close</span>
            <input
              type="checkbox"
              class="h-4 w-4 accent-primary"
              v-model="settings.minimizeToTray"
              @change="onChange"
            />
          </label>

          <div class="flex items-center justify-between py-4">
            <span class="text-body text-on-surface">Sleep Timer default</span>
            <select
              class="rounded-sm border border-outline bg-surface-raised-high px-3 py-1 text-body text-on-surface"
              v-model.number="settings.sleepTimerDefaultMinutes"
              @change="onChange"
            >
              <option :value="15">15 min</option>
              <option :value="30">30 min</option>
              <option :value="60">60 min</option>
              <option :value="90">90 min</option>
            </select>
          </div>

          <div class="flex items-center justify-between py-4">
            <span class="text-body text-on-surface">Theme</span>
            <select
              class="rounded-sm border border-outline bg-surface-raised-high px-3 py-1 text-body text-on-surface"
              v-model="settings.theme"
              @change="onChange"
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </div>
        </div>

        <div class="flex justify-end px-6 py-4">
          <button
            type="button"
            class="rounded-sm bg-surface-raised-high px-4 py-2 text-label-link text-on-surface hover:bg-outline"
            @click="close"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { watch } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { usePlaybackStore } from '@/stores/playback'

const props = defineProps<{ show: boolean }>()
const emit = defineEmits<{ close: [] }>()

const settings = useSettingsStore()
const playback = usePlaybackStore()

// No explicit Apply/Save button anywhere in the app — every change applies
// and persists immediately (EXPERIENCE.md, Settings modal).
const onChange = () => {
  settings.saveSettings(playback.volume)
}

const close = () => emit('close')

// `Esc` closes the Settings modal (EXPERIENCE.md Accessibility Floor). A
// window-level listener is used rather than a `@keydown` on the overlay
// because focus may still be on the nav button that opened the modal.
const onKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') close()
}

watch(
  () => props.show,
  (isOpen) => {
    if (isOpen) {
      window.addEventListener('keydown', onKeydown)
    } else {
      window.removeEventListener('keydown', onKeydown)
    }
  },
)
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
