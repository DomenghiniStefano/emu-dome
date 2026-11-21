<script setup lang="ts">
// The cover grid of «I tuoi giochi»: as many columns as fit, each at least `capsule-medium`.
// Always rendered, empty too: the view's gaps around it stay the same.
//
// Windowed (Trello #91, useCoverWindow.ts): only the rows near the screen are in the DOM, the
// grid's padding stands for the others, so the scroll height is the whole list's. The covers
// still load lazily (GameCover). The grid holds the focus itself (no ring) only while its
// focused tile is scrolled out of the DOM.
import { ref, watch } from 'vue'
import type { GameId, LibraryGame } from '@/emu'
import CoverTile from './CoverTile.vue'
import { useCoverWindow } from './useCoverWindow'

const props = defineProps<{
  games: LibraryGame[]
  /** The id of the game whose «…» menu is open. */
  menuFor: string | null
  /** A game whose row is rendered first (back from its details), read on mount. */
  anchor: GameId | null
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

const grid = ref<HTMLElement | null>(null)
const { visible, style } = useCoverWindow(grid, () => props.games, props.anchor)

// A «…» menu whose tile leaves the DOM closes with it.
watch(visible, shown => {
  if (props.menuFor !== null && !shown.some(g => g.id === props.menuFor)) emit('closeMenu')
})
</script>

<template>
  <div
    ref="grid"
    class="focus-ring-off grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-capsule-medium),1fr))] gap-16"
    :style="style"
  >
    <CoverTile
      v-for="g in visible"
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
