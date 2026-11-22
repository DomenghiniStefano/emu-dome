// The cover grid's controller / keyboard steps (Trello #91): nav.ts asks the grid first
// (`useNavGrid`), the step is worked out by row and column (coverGridNav.ts) and lands on the
// tile's element, rendered first when it is not in the DOM (useCoverWindow's `reveal`).
// - From the parked grid (its focused tile scrolled out of the DOM), a step starts from that
//   tile; one that would leave the grid brings the user back to that tile instead.
// - Steps arriving while a tile is being rendered start from that tile, so none is lost.
// - The phone's list and compact grid (#29) pass their own `step` and what has a button
//   (`playable`); the desk's grid keeps `coverStep` and the games that can start.
import type { Ref } from 'vue'
import type { LibraryGame } from '@/emu'
import { useNavGrid } from '@/nav'
import { coverStep, type CellStep, type CoverCell } from './coverGridNav'
import type { useCoverWindow } from './useCoverWindow'

export interface GridNavOptions {
  step?: CellStep
  /** Whether game `i` of the list has its `play` part to land on. */
  playable?: (list: LibraryGame[], i: number) => boolean
}

const canStart = (list: LibraryGame[], i: number) => list[i]?.available ?? false

export function useCoverGridNav(
  grid: Ref<HTMLElement | null>,
  games: () => LibraryGame[],
  win: ReturnType<typeof useCoverWindow>,
  options: GridNavOptions = {},
) {
  const step = options.step ?? coverStep
  const playable = options.playable ?? canStart
  let pending: CoverCell | null = null

  useNavGrid(grid, (from, dir) => {
    const parked = from === grid.value
    const cell = pending ?? (parked ? win.parkedCell() : win.cellOf(from))
    if (!cell) return false
    const list = games()
    const to = step(cell, dir, {
      cols: win.cols.value,
      count: list.length,
      playable: i => playable(list, i),
    })
    const target = to ?? (parked || pending ? cell : null)
    if (!target) return false
    const el = win.cellElement(target)
    if (el) {
      pending = null
      return el
    }
    pending = target
    return win.reveal(target).finally(() => {
      if (pending === target) pending = null
    })
  })
}
