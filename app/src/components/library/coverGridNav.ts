// Controller / keyboard steps inside the cover grid (Trello #91), by row and column instead of
// measuring every tile. Each tile has three focusables: its cover (the click area, on top) and,
// below it, «Gioca» and «…» (CoverTile.vue). The moves are those nav.ts's spatial rule made on
// the full grid:
// - cover: down to its own strip, up to the strip of the tile above, left / right to the next
//   cover in the row;
// - strip («Gioca», «…»): left / right along the row's strips, up to its own cover, down to the
//   cover below (the last game when the row below is shorter);
// - a strip lands on «Gioca», or on «…» when the game cannot be played («Gioca» aria-disabled).
// `null`: the step leaves the grid (top row up, first column left, last column right, last row
// down), and nav.ts moves spatially from there (toolbar, «Continua a giocare», the rail).
import type { Direction } from '@/nav'

/** The focusable parts of a tile, top to bottom, left to right. */
export type CoverPart = 'cover' | 'play' | 'more'
/** One focusable part of one game of the grid. */
export interface CoverCell {
  index: number
  part: CoverPart
}

export const COVER_PARTS: readonly CoverPart[] = ['cover', 'play', 'more']

export interface GridShape {
  cols: number
  count: number
  /** Whether game `i`'s «Gioca» can take the focus. */
  playable: (i: number) => boolean
}

/** The strip part a step onto game `i`'s strip lands on. */
const stripOf = (i: number, g: GridShape): CoverCell => ({
  index: i,
  part: g.playable(i) ? 'play' : 'more',
})

/** The strip parts of the row from `first` to `last`, left to right. */
function rowStrip(first: number, last: number, g: GridShape): CoverCell[] {
  const cells: CoverCell[] = []
  for (let i = first; i <= last; i++) {
    if (g.playable(i)) cells.push({ index: i, part: 'play' })
    cells.push({ index: i, part: 'more' })
  }
  return cells
}

/** Where a step from `from` lands in the grid, or null when it leaves the grid. */
export function coverStep(from: CoverCell, dir: Direction, g: GridShape): CoverCell | null {
  const cols = Math.max(1, g.cols)
  const { index } = from
  if (index < 0 || index >= g.count) return null
  const col = index % cols
  const rowFirst = index - col
  const rowLast = Math.min(rowFirst + cols, g.count) - 1
  // The game below, or the last one when the row below is shorter; none on the last row.
  const below = rowFirst + cols < g.count ? Math.min(index + cols, g.count - 1) : null

  if (from.part === 'cover') {
    switch (dir) {
      case 'up':
        return index >= cols ? stripOf(index - cols, g) : null
      case 'down':
        return stripOf(index, g)
      case 'left':
        return index > rowFirst ? { index: index - 1, part: 'cover' } : null
      case 'right':
        return index < rowLast ? { index: index + 1, part: 'cover' } : null
    }
  }
  switch (dir) {
    case 'up':
      return { index, part: 'cover' }
    case 'down':
      return below === null ? null : { index: below, part: 'cover' }
    case 'left':
    case 'right': {
      const strip = rowStrip(rowFirst, rowLast, g)
      const at = strip.findIndex(c => c.index === index && c.part === from.part)
      return strip[at + (dir === 'left' ? -1 : 1)] ?? null
    }
  }
}
