<script setup lang="ts">
// The library as a list (#29, `library.view` «Lista», the phone's default): one GameRow per game,
// in the view's order and filters. Windowed like the cover grid (useCoverWindow.ts on a grid of
// one column, two on the phone on its side, #105: the short window shows six games instead of
// three): only the rows near the screen are in the DOM, the list's padding stands for the
// others. The controller and the arrows move by row, and between a row, its button and the next
// column (coverGridNav.ts `listStep`); past the list's edges nav.ts moves on spatially.
import { ref } from 'vue'
import type { GameId, LibraryGame } from '@/emu'
import { versionCount } from '@/lib/families'
import { libraryFamilies } from '@/lib/library'
import GameRow from './GameRow.vue'
import { listStep } from './coverGridNav'
import { useCoverGridNav } from './useCoverGridNav'
import { useCoverWindow } from './useCoverWindow'

/** A row before it is measured: the 66 px cover, 8 px above and below, the 1 px divider. */
const ROW_ESTIMATE = 83

const props = defineProps<{
  games: LibraryGame[]
  /** The game whose row carries «Riprendi» (the phone's, #29), or none. */
  resumeId: GameId | null
  /** A game whose row is rendered first (back from its details), read on mount. */
  anchor: GameId | null
}>()
const emit = defineEmits<{
  details: [LibraryGame]
  play: [LibraryGame]
  resume: [LibraryGame]
  /** The pointer or the focus is on this game: its cover tints the haze. */
  foreground: [LibraryGame]
}>()

const list = ref<HTMLElement | null>(null)
const win = useCoverWindow(
  list,
  () => props.games,
  props.anchor,
  () => ROW_ESTIMATE,
)
const { visible, style } = win
useCoverGridNav(list, () => props.games, win, {
  step: listStep,
  playable: (games, i) => games[i]?.available ?? false,
})
</script>

<template>
  <div
    ref="list"
    class="focus-ring-off grid grid-cols-1 touch:landscape:grid-cols-2 touch:landscape:gap-x-24"
    :style="style"
  >
    <GameRow
      v-for="g in visible"
      :key="g.id"
      :game="g"
      :resume="g.id === resumeId"
      :versions="versionCount(libraryFamilies, g.id)"
      @mouseenter="emit('foreground', g)"
      @focusin="emit('foreground', g)"
      @details="emit('details', g)"
      @play="emit('play', g)"
      @resume="emit('resume', g)"
    />
  </div>
</template>
