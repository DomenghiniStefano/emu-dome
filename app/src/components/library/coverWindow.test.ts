import { describe, expect, it } from 'vitest'
import {
  averageHeight,
  firstTrack,
  padding,
  rowCount,
  rowOf,
  rowTops,
  rowsInView,
  scrollToShow,
  trackCount,
  type RowLayout,
} from './coverWindow'

const layout = (rows: number, heights: (number | undefined)[] = [], estimate = 100, gap = 10) =>
  ({ rows, heights, estimate, gap }) satisfies RowLayout

describe('the grid template', () => {
  it('counts the resolved tracks, at least one', () => {
    expect(trackCount('156px 156px 156.5px')).toBe(3)
    expect(trackCount('200px')).toBe(1)
    expect(trackCount('none')).toBe(1)
    expect(trackCount('')).toBe(1)
  })
  it("reads the first track's width", () => {
    expect(firstTrack('156.25px 156.25px')).toBe(156.25)
    expect(firstTrack('none')).toBeNull()
  })
})

describe('rows', () => {
  it('counts the rows and finds the row of a game', () => {
    expect(rowCount(0, 6)).toBe(0)
    expect(rowCount(13, 6)).toBe(3)
    expect(rowCount(12, 6)).toBe(2)
    expect(rowOf(5, 6)).toBe(0)
    expect(rowOf(6, 6)).toBe(1)
    expect(rowCount(5, 0)).toBe(5)
  })
  it('averages the measured heights, the fallback when none is', () => {
    expect(averageHeight([], 42)).toBe(42)
    expect(averageHeight([100, undefined, 200, 0], 42)).toBe(150)
  })
  it('places each row after the ones above, estimated where not measured', () => {
    // 100 estimated, 50 measured, 100 estimated; 10 of gap after each.
    expect(rowTops(layout(3, [undefined, 50]))).toEqual([0, 110, 170, 280])
    expect(rowTops(layout(0))).toEqual([0])
  })
})

describe('the rows in view', () => {
  const l = layout(100)
  const tops = rowTops(l) // every row 110 apart

  it('renders the rows the view touches plus the margin', () => {
    expect(rowsInView(l, tops, 0, 500, 2)).toEqual({ start: 0, end: 7 })
    // View from 1100 (row 10) to 1600 (row 14).
    expect(rowsInView(l, tops, 1100, 1600, 2)).toEqual({ start: 8, end: 17 })
  })
  it('a view above the grid starts at the first row', () => {
    // The toolbar and «Continue playing» above: the grid's top is below the view's top.
    expect(rowsInView(l, tops, -400, 300, 1)).toEqual({ start: 0, end: 4 })
  })
  it('a view past the end still renders the last rows', () => {
    expect(rowsInView(l, tops, 50_000, 50_500, 2)).toEqual({ start: 97, end: 100 })
  })
  it('no rows, nothing to render', () => {
    expect(rowsInView(layout(0), [0], 0, 500, 2)).toEqual({ start: 0, end: 0 })
  })
  it('a thousand-row list renders a dozen rows', () => {
    const big = layout(1000)
    const r = rowsInView(big, rowTops(big), 55_000, 56_080, 2)
    expect(r.end - r.start).toBeLessThanOrEqual(15)
  })
})

describe('padding', () => {
  const l = layout(10)
  const tops = rowTops(l)
  it('stands for the rows above and below, so the height is the whole list', () => {
    const p = padding(tops, 10, { start: 3, end: 6 })
    expect(p).toEqual({ top: 330, bottom: 440 })
    // The rendered rows (3 × 100 + 2 gaps) plus the padding: the full grid (10 × 100 + 9 gaps).
    expect(p.top + 3 * 100 + 2 * 10 + p.bottom).toBe(10 * 100 + 9 * 10)
  })
  it('none when every row is rendered', () => {
    expect(padding(tops, 10, { start: 0, end: 10 })).toEqual({ top: 0, bottom: 0 })
  })
})

describe('scrollToShow', () => {
  it('moves the least, as block: nearest', () => {
    expect(scrollToShow(100, 200, 0, 500)).toBe(0)
    expect(scrollToShow(600, 700, 0, 500)).toBe(200)
    expect(scrollToShow(50, 150, 100, 500)).toBe(-50)
    // Taller than the view: its top at the top.
    expect(scrollToShow(600, 1300, 0, 500)).toBe(600)
  })
})
