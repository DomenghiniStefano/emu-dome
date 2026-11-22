import { describe, expect, it } from 'vitest'
import type { Direction } from '@/nav'
import {
  compactStep,
  coverStep,
  listStep,
  type CoverCell,
  type CoverPart,
  type GridShape,
} from './coverGridNav'

// 4 columns, 10 games: rows 0–3, 4–7, 8–9. Game 5 cannot be played («Gioca» disabled).
const shape: GridShape = { cols: 4, count: 10, playable: i => i !== 5 }
const at = (index: number, part: CoverPart): CoverCell => ({ index, part })
const step = (from: CoverCell, dir: Direction) => coverStep(from, dir, shape)

describe('from a cover', () => {
  it('down to its own «Gioca», or «…» when it cannot be played', () => {
    expect(step(at(1, 'cover'), 'down')).toEqual(at(1, 'play'))
    expect(step(at(5, 'cover'), 'down')).toEqual(at(5, 'more'))
  })
  it('up to the strip of the tile above; the top row leaves the grid', () => {
    expect(step(at(9, 'cover'), 'up')).toEqual(at(5, 'more'))
    expect(step(at(6, 'cover'), 'up')).toEqual(at(2, 'play'))
    expect(step(at(2, 'cover'), 'up')).toBeNull()
  })
  it('left / right along the row; its ends leave the grid', () => {
    expect(step(at(5, 'cover'), 'left')).toEqual(at(4, 'cover'))
    expect(step(at(5, 'cover'), 'right')).toEqual(at(6, 'cover'))
    expect(step(at(4, 'cover'), 'left')).toBeNull()
    expect(step(at(7, 'cover'), 'right')).toBeNull()
    // The last game of a short last row.
    expect(step(at(9, 'cover'), 'right')).toBeNull()
  })
})

describe('from a strip', () => {
  it('up to its own cover', () => {
    expect(step(at(6, 'play'), 'up')).toEqual(at(6, 'cover'))
    expect(step(at(6, 'more'), 'up')).toEqual(at(6, 'cover'))
  })
  it('down to the cover below, the last game when the row below is shorter', () => {
    expect(step(at(1, 'more'), 'down')).toEqual(at(5, 'cover'))
    expect(step(at(7, 'play'), 'down')).toEqual(at(9, 'cover'))
    expect(step(at(9, 'play'), 'down')).toBeNull()
  })
  it('left / right along the strips of the row, skipping a disabled «Gioca»', () => {
    expect(step(at(4, 'play'), 'right')).toEqual(at(4, 'more'))
    expect(step(at(4, 'more'), 'right')).toEqual(at(5, 'more'))
    expect(step(at(5, 'more'), 'right')).toEqual(at(6, 'play'))
    expect(step(at(6, 'play'), 'left')).toEqual(at(5, 'more'))
    expect(step(at(5, 'more'), 'left')).toEqual(at(4, 'more'))
    expect(step(at(4, 'play'), 'left')).toBeNull()
    expect(step(at(7, 'more'), 'right')).toBeNull()
  })
})

describe('the shape', () => {
  it('one column: up and down walk the list, left and right leave', () => {
    const one: GridShape = { cols: 1, count: 3, playable: () => true }
    expect(coverStep(at(0, 'play'), 'down', one)).toEqual(at(1, 'cover'))
    expect(coverStep(at(1, 'cover'), 'up', one)).toEqual(at(0, 'play'))
    expect(coverStep(at(1, 'cover'), 'left', one)).toBeNull()
    expect(coverStep(at(1, 'more'), 'right', one)).toBeNull()
  })
  it('a game no longer in the list leaves the grid', () => {
    expect(step(at(10, 'cover'), 'down')).toBeNull()
    expect(step(at(-1, 'cover'), 'up')).toBeNull()
  })
  it('a thousand-game grid steps without walking the list', () => {
    const big: GridShape = { cols: 8, count: 100_000, playable: () => true }
    expect(coverStep(at(50_003, 'play'), 'down', big)).toEqual(at(50_011, 'cover'))
  })
})

describe('the phone’s list (#29)', () => {
  // Five rows; game 2 cannot be played (no button to land on).
  const list: GridShape = { cols: 1, count: 5, playable: i => i !== 2 }
  const go = (from: CoverCell, dir: Direction) => listStep(from, dir, list)
  it('up and down by row, keeping the side; a row without its button takes the row', () => {
    expect(go(at(1, 'cover'), 'down')).toEqual(at(2, 'cover'))
    expect(go(at(1, 'play'), 'down')).toEqual(at(2, 'cover'))
    expect(go(at(3, 'play'), 'up')).toEqual(at(2, 'cover'))
    expect(go(at(3, 'play'), 'down')).toEqual(at(4, 'play'))
    expect(go(at(0, 'cover'), 'up')).toBeNull()
    expect(go(at(4, 'play'), 'down')).toBeNull()
  })
  it('left and right between a row and its button; past them the list is left', () => {
    expect(go(at(1, 'cover'), 'right')).toEqual(at(1, 'play'))
    expect(go(at(1, 'play'), 'left')).toEqual(at(1, 'cover'))
    expect(go(at(2, 'cover'), 'right')).toBeNull()
    expect(go(at(1, 'cover'), 'left')).toBeNull()
    expect(go(at(1, 'play'), 'right')).toBeNull()
  })
  it('a hundred thousand rows step without walking the list', () => {
    const big: GridShape = { cols: 1, count: 100_000, playable: () => true }
    expect(listStep(at(50_000, 'play'), 'down', big)).toEqual(at(50_001, 'play'))
  })
})

describe('the phone’s list on its side, two columns (#105)', () => {
  // 2 columns, 5 games: rows 0–1, 2–3, 4. Game 2 cannot be played.
  const list: GridShape = { cols: 2, count: 5, playable: i => i !== 2 }
  const go = (from: CoverCell, dir: Direction) => listStep(from, dir, list)
  it('up and down by row in the same column, keeping the side', () => {
    expect(go(at(1, 'play'), 'down')).toEqual(at(3, 'play'))
    expect(go(at(1, 'cover'), 'down')).toEqual(at(3, 'cover'))
    expect(go(at(0, 'play'), 'down')).toEqual(at(2, 'cover'))
    expect(go(at(3, 'cover'), 'up')).toEqual(at(1, 'cover'))
    expect(go(at(1, 'cover'), 'up')).toBeNull()
  })
  it('down to the last game when the row below is shorter; none from the last row', () => {
    expect(go(at(3, 'play'), 'down')).toEqual(at(4, 'play'))
    expect(go(at(2, 'cover'), 'down')).toEqual(at(4, 'cover'))
    expect(go(at(4, 'cover'), 'down')).toBeNull()
  })
  it('left and right through the row, its button, the next column; the ends leave', () => {
    expect(go(at(0, 'cover'), 'right')).toEqual(at(0, 'play'))
    expect(go(at(0, 'play'), 'right')).toEqual(at(1, 'cover'))
    expect(go(at(1, 'cover'), 'left')).toEqual(at(0, 'play'))
    expect(go(at(1, 'play'), 'left')).toEqual(at(1, 'cover'))
    expect(go(at(1, 'play'), 'right')).toBeNull()
    expect(go(at(0, 'cover'), 'left')).toBeNull()
    // A row without its button: straight to the next column, and back onto the row itself.
    expect(go(at(2, 'cover'), 'right')).toEqual(at(3, 'cover'))
    expect(go(at(3, 'cover'), 'left')).toEqual(at(2, 'cover'))
    // The last game of a short last row.
    expect(go(at(4, 'play'), 'right')).toBeNull()
  })
})

describe('the phone’s compact grid (#29)', () => {
  // 3 columns, 7 games: rows 0–2, 3–5, 6. Game 1 has «Riprendi» in its corner.
  const grid: GridShape = { cols: 3, count: 7, playable: i => i === 1 }
  const go = (from: CoverCell, dir: Direction) => compactStep(from, dir, grid)
  it('covers by row and column, the last game under a shorter row', () => {
    expect(go(at(0, 'cover'), 'right')).toEqual(at(1, 'cover'))
    expect(go(at(2, 'cover'), 'right')).toBeNull()
    expect(go(at(3, 'cover'), 'left')).toBeNull()
    expect(go(at(4, 'cover'), 'up')).toEqual(at(1, 'cover'))
    expect(go(at(0, 'cover'), 'down')).toEqual(at(3, 'cover'))
    expect(go(at(5, 'cover'), 'down')).toEqual(at(6, 'cover'))
    expect(go(at(6, 'cover'), 'down')).toBeNull()
    expect(go(at(2, 'cover'), 'up')).toBeNull()
  })
  it('down from the cover with «Riprendi» lands on it first, then on the cover below', () => {
    expect(go(at(1, 'cover'), 'down')).toEqual(at(1, 'play'))
    expect(go(at(1, 'play'), 'down')).toEqual(at(4, 'cover'))
    expect(go(at(1, 'play'), 'up')).toEqual(at(1, 'cover'))
    expect(go(at(1, 'play'), 'left')).toEqual(at(1, 'cover'))
    expect(go(at(1, 'play'), 'right')).toEqual(at(2, 'cover'))
  })
})
