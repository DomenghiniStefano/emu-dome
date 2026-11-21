import { describe, expect, it } from 'vitest'
import type { Direction } from '@/nav'
import { coverStep, type CoverCell, type CoverPart, type GridShape } from './coverGridNav'

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
