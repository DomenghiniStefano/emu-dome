// The cover grid's windowing (Trello #91): only the rows near the screen are rendered, the rows
// above and below are the grid's own padding, so the scroll height stays that of the whole grid.
//
// Rows are measured once rendered (the titles wrap, so a row's height depends on its games); a
// row not measured yet counts as the average of the measured ones. Everything here is in the
// grid's own coordinates: 0 is the top of the grid's box, a row's top includes the gaps above it.
// Pure: CoverGrid measures the DOM, these functions do the sums.

/** The grid's rows as known now. */
export interface RowLayout {
  /** How many rows the whole list makes. */
  rows: number
  /** Measured heights by row; `undefined` or 0 for a row never rendered. */
  heights: readonly (number | undefined)[]
  /** The height of a row not measured yet. */
  estimate: number
  /** The gap between two rows. */
  gap: number
}

/** Rows `[start, end)`. */
export interface RowRange {
  start: number
  end: number
}

/** The number of tracks in a resolved `grid-template-columns` ("156px 156px 156px": 3), at least 1. */
export function trackCount(template: string): number {
  const tracks = template
    .trim()
    .split(/\s+/)
    .filter(t => /^\d/.test(t))
  return Math.max(1, tracks.length)
}

/** The width of a resolved template's first track, or null when it has none. */
export function firstTrack(template: string): number | null {
  const first = template.trim().split(/\s+/)[0]
  const width = first === undefined ? NaN : parseFloat(first)
  return Number.isFinite(width) ? width : null
}

/** How many rows `count` games make in `cols` columns. */
export const rowCount = (count: number, cols: number) => Math.ceil(count / Math.max(1, cols))

/** The row of the game at `index`. */
export const rowOf = (index: number, cols: number) => Math.floor(index / Math.max(1, cols))

/** The average of the measured heights, or `fallback` when none is measured. */
export function averageHeight(heights: readonly (number | undefined)[], fallback: number): number {
  let sum = 0
  let n = 0
  for (const h of heights) {
    if (!h) continue
    sum += h
    n++
  }
  return n ? sum / n : fallback
}

/**
 * The top of every row and, last, the end of the list plus one gap: `tops[r]` is where row `r`
 * starts, `tops[rows]` - gap is the grid's full height.
 */
export function rowTops(layout: RowLayout): number[] {
  const tops = new Array<number>(layout.rows + 1)
  let y = 0
  for (let r = 0; r < layout.rows; r++) {
    tops[r] = y
    y += (layout.heights[r] || layout.estimate) + layout.gap
  }
  tops[layout.rows] = y
  return tops
}

/** The last row whose top is at or above `y` (0 when `y` is above the first). */
function rowAt(tops: number[], rows: number, y: number): number {
  let lo = 0
  let hi = rows - 1
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1
    if ((tops[mid] ?? 0) <= y) lo = mid
    else hi = mid - 1
  }
  return lo
}

/**
 * The rows to render for the view `[top, bottom)` (grid coordinates): those it touches, plus
 * `margin` rows on each side. Never empty while there are rows, so a view far off the estimates
 * still renders the rows nearest to it.
 */
export function rowsInView(
  layout: RowLayout,
  tops: number[],
  top: number,
  bottom: number,
  margin: number,
): RowRange {
  const { rows } = layout
  if (rows === 0) return { start: 0, end: 0 }
  const first = rowAt(tops, rows, top)
  const last = rowAt(tops, rows, Math.max(top, bottom - 1))
  return {
    start: Math.max(0, first - margin),
    end: Math.min(rows, last + 1 + margin),
  }
}

/** The grid's padding above and below the rendered rows, standing for the rows not rendered. */
export function padding(tops: number[], rows: number, range: RowRange) {
  return {
    top: tops[range.start] ?? 0,
    bottom: Math.max(0, (tops[rows] ?? 0) - (tops[range.end] ?? 0)),
  }
}

/**
 * How far to scroll so that `[top, bottom)` is in the view `[viewTop, viewTop + height)`, the
 * least move as `scrollIntoView({ block: 'nearest' })`: 0 when it is already in.
 */
export function scrollToShow(top: number, bottom: number, viewTop: number, height: number) {
  if (top < viewTop) return top - viewTop
  if (bottom > viewTop + height) return Math.min(top - viewTop, bottom - (viewTop + height))
  return 0
}

/** Whether two ranges are the same rows. */
export const sameRange = (a: RowRange, b: RowRange) => a.start === b.start && a.end === b.end
