<script setup lang="ts">
// The cover grid of «I tuoi giochi»: as many columns as fit, each at least `capsule-medium`.
// `compact` (the phone, #29, mockup «Libreria sul telefono» B): CompactTile, each at least
// `capsule-touch` upright (three columns) and `capsule-touch-side` on its side (five or six), the
// game played last with «Riprendi» in its cover's corner (`resumeId`).
// Always rendered, empty too: the view's gaps around it stay the same.
//
// Windowed (Trello #91, useCoverWindow.ts): only the rows near the screen are in the DOM, the
// grid's padding stands for the others, so the scroll height is the whole list's. The covers
// still load lazily (GameCover). The grid holds the focus itself (no ring) only while its
// focused tile is scrolled out of the DOM. The controller and the arrows move inside it by row and
// column (useCoverGridNav.ts; the compact grid by coverGridNav.ts `compactStep`), and leave it at
// its edges by nav.ts's spatial rule.
import { ref, watch } from 'vue'
import type { GameId, LibraryGame } from '@/emu'
import { versionCount } from '@/lib/families'
import { libraryFamilies } from '@/lib/library'
import CompactTile from './CompactTile.vue'
import CoverTile from './CoverTile.vue'
import { compactStep, coverStep, type CellStep } from './coverGridNav'
import { useCoverGridNav } from './useCoverGridNav'
import { useCoverWindow } from './useCoverWindow'

const props = withDefaults(
  defineProps<{
    games: LibraryGame[]
    /** The id of the game whose «…» menu is open. */
    menuFor: string | null
    /** A game whose row is rendered first (back from its details), read on mount. */
    anchor: GameId | null
    /** The phone's compact grid. */
    compact?: boolean
    /** In the compact grid, the game whose cover carries «Riprendi», or none. */
    resumeId?: GameId | null
  }>(),
  { compact: false, resumeId: null },
)
const emit = defineEmits<{
  details: [LibraryGame]
  play: [LibraryGame]
  resume: [LibraryGame]
  toggleMenu: [LibraryGame, Event]
  closeMenu: []
  flags: [LibraryGame, { favourite?: boolean; hidden?: boolean }]
  /** The pointer or the focus is on this game: its cover tints the haze. */
  foreground: [LibraryGame]
}>()

const grid = ref<HTMLElement | null>(null)
const win = useCoverWindow(grid, () => props.games, props.anchor)
const { visible, style } = win
const step: CellStep = (from, dir, shape) =>
  props.compact ? compactStep(from, dir, shape) : coverStep(from, dir, shape)
useCoverGridNav(grid, () => props.games, win, {
  step,
  // The compact grid's one button is «Riprendi»; the desk's tiles have «Gioca» when they can start.
  playable: (games, i) =>
    props.compact ? games[i]?.id === props.resumeId : (games[i]?.available ?? false),
})

// A «…» menu whose tile leaves the DOM closes with it.
watch(visible, shown => {
  if (props.menuFor !== null && !shown.some(g => g.id === props.menuFor)) emit('closeMenu')
})
</script>

<template>
  <div
    ref="grid"
    class="focus-ring-off grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-capsule-medium),1fr))] gap-12 touch:grid-cols-[repeat(auto-fill,minmax(var(--spacing-capsule-touch),1fr))] touch:gap-x-8 touch:landscape:grid-cols-[repeat(auto-fill,minmax(var(--spacing-capsule-touch-side),1fr))]"
    :style="style"
  >
    <template v-if="compact">
      <CompactTile
        v-for="g in visible"
        :key="g.id"
        :game="g"
        :resume="g.id === resumeId"
        :versions="versionCount(libraryFamilies, g.id)"
        @mouseenter="emit('foreground', g)"
        @focusin="emit('foreground', g)"
        @details="emit('details', g)"
        @resume="emit('resume', g)"
      />
    </template>
    <template v-else>
      <CoverTile
        v-for="g in visible"
        :key="g.id"
        :game="g"
        :versions="versionCount(libraryFamilies, g.id)"
        :menu-open="menuFor === g.id"
        @mouseenter="emit('foreground', g)"
        @focusin="emit('foreground', g)"
        @details="emit('details', g)"
        @play="emit('play', g)"
        @toggle-menu="e => emit('toggleMenu', g, e)"
        @close-menu="emit('closeMenu')"
        @flags="f => emit('flags', g, f)"
      />
    </template>
  </div>
</template>
