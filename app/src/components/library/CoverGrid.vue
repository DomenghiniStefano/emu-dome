<script setup lang="ts">
// The cover grid of «I tuoi giochi»: as many columns as fit, each at least `capsule-medium`.
// Always rendered, empty too: the view's gaps around it stay the same.
import type { LibraryGame } from '@/emu'
import CoverTile from './CoverTile.vue'

defineProps<{
  games: LibraryGame[]
  /** The id of the game whose «…» menu is open. */
  menuFor: string | null
}>()
const emit = defineEmits<{
  details: [LibraryGame]
  play: [LibraryGame]
  toggleMenu: [LibraryGame, Event]
  closeMenu: []
  flags: [LibraryGame, { favourite?: boolean; hidden?: boolean }]
  /** The pointer or the focus is on this game: its cover tints the haze. */
  foreground: [LibraryGame]
}>()
</script>

<template>
  <div class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-capsule-medium),1fr))] gap-16">
    <CoverTile
      v-for="g in games"
      :key="g.id"
      :game="g"
      :menu-open="menuFor === g.id"
      @mouseenter="emit('foreground', g)"
      @focusin="emit('foreground', g)"
      @details="emit('details', g)"
      @play="emit('play', g)"
      @toggle-menu="e => emit('toggleMenu', g, e)"
      @close-menu="emit('closeMenu')"
      @flags="f => emit('flags', g, f)"
    />
  </div>
</template>
