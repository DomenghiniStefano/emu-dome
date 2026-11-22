<script setup lang="ts">
// The Library section in TV mode (#30, mockup «TV-Libreria», option A): the same library and the
// same actions as the desk's, laid out for the sofa. The focused game large on top (TvHero), the
// shelves under it (TvShelf: «Continua a giocare», «Preferiti», one per console; tvShelves.ts).
//
// Controller: the hero's «Riprendi» / «Gioca» takes the first focus (back from a game's details,
// that game's cover does: useLibraryFocus). On the shelves the d-pad moves by cover and by shelf
// (`useNavGrid`, so a library of thousands measures only the shelf it moves to); up from the
// first shelf, and down from the hero, the spatial rule. A plays the focused game, X opens its
// details (nav.ts `more` is a right click), and the focus fills the hero and tints the haze
// (`ambientGame`, the same as the desk's).
import { computed, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { GameId, LibraryGame } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { ambientGame } from '@/lib/ambient'
import { useNavGrid } from '@/nav'
import { useLibraryFocus } from '@/components/library/useLibraryFocus'
import { useResumePoints } from '@/components/library/useResumePoints'
import TvHero from './TvHero.vue'
import TvShelf from './TvShelf.vue'
import {
  firstShelfLimit,
  nearestCover,
  shelfLimitAfter,
  tvShelves,
  type ShelfId,
} from './tvShelves'

const props = defineProps<{
  games: LibraryGame[]
  /** Back from this game's details: its cover takes the first focus (read on mount). */
  returnTo: GameId | null
}>()
const emit = defineEmits<{
  /** `slot`: the quick save to load once the game runs («Riprendi»). */
  open: [game: LibraryGame, slot?: number]
  details: [LibraryGame]
}>()

const { t } = useI18n()
const failWith = useErrorSink()

const root = ref<HTMLElement | null>(null)
useLibraryFocus(root, props.returnTo)

const shelves = computed(() => tvShelves(props.games))
const played = computed(() => shelves.value.find(s => s.id === 'continue')?.games ?? [])
const resume = useResumePoints(played, failWith)

/** The game on top: the focused cover's, until one the first of the shelves. */
const hero = computed(() => {
  const g = ambientGame.value
  const first = shelves.value[0]?.games[0] ?? null
  return (g && props.games.find(x => x.id === g.id && !x.hidden)) || first
})
const resumable = (g: LibraryGame) => {
  const point = resume.value[g.id]
  return !!point && !point.none
}
/** «Riprendi» from where the newest save is, else the game from its start screen. */
const play = (g: LibraryGame) => emit('open', g, resume.value[g.id]?.slot ?? undefined)

// ---- The covers each shelf renders (tvShelves.ts: a page, the next as the focus nears its end)

const limits = reactive<Partial<Record<ShelfId, number>>>({})
const limitOf = (id: ShelfId) => {
  const shelf = shelves.value.find(s => s.id === id)
  return limits[id] ?? (shelf ? firstShelfLimit(shelf, props.returnTo) : 0)
}
function onFocused(id: ShelfId, g: LibraryGame, index: number) {
  ambientGame.value = g
  const total = shelves.value.find(s => s.id === id)?.games.length ?? 0
  limits[id] = shelfLimitAfter(limitOf(id), index, total)
}

// ---- The steps on the shelves

const shelvesEl = ref<HTMLElement | null>(null)
const centre = (el: Element) => {
  const r = el.getBoundingClientRect()
  return r.left + r.width / 2
}
useNavGrid(shelvesEl, (from, dir) => {
  const row = from.closest<HTMLElement>('[data-shelf-row]')
  const index = Number(from.dataset.shelfIndex)
  if (!row || !shelvesEl.value || Number.isNaN(index)) return false
  if (dir === 'left' || dir === 'right') {
    const next = index + (dir === 'right' ? 1 : -1)
    // At a shelf's ends the focus stays: the next shelf is up or down, never sideways.
    return row.querySelector<HTMLElement>(`[data-shelf-index="${next}"]`) ?? from
  }
  const rows = [...shelvesEl.value.querySelectorAll<HTMLElement>('[data-shelf-row]')]
  const target = rows[rows.indexOf(row) + (dir === 'down' ? 1 : -1)]
  if (!target) return false
  const covers = [...target.querySelectorAll<HTMLElement>('[data-shelf-index]')]
  const at = nearestCover(covers.map(centre), centre(from))
  return (at !== null && covers[at]) || false
})
</script>

<template>
  <div ref="root" class="flex min-h-0 flex-1 flex-col">
    <TvHero
      v-if="hero"
      :game="hero"
      :resumable="resumable(hero)"
      @play="play(hero)"
      @details="emit('details', hero)"
    />
    <div
      ref="shelvesEl"
      class="flex min-h-0 flex-1 flex-col gap-tv-shelf-gap overflow-y-auto pt-24 pb-tv-shelf-gap pl-tv-gutter [scrollbar-width:none]"
      :aria-label="t('tv.shelf.shelves')"
    >
      <TvShelf
        v-for="s in shelves"
        :key="s.id"
        :shelf="s"
        :limit="limitOf(s.id)"
        @play="play"
        @details="g => emit('details', g)"
        @focused="(g, i) => onFocused(s.id, g, i)"
      />
    </div>
  </div>
</template>
