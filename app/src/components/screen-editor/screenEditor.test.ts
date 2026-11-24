import { describe, expect, it } from 'vitest'
import type { CustomLayout, CustomScreen } from '@/emu'
import {
  GRID,
  MIN_SIZE,
  aspect,
  contain,
  fromPercent,
  front,
  grow,
  heightFor,
  keepPanelSide,
  moveTo,
  panelSide,
  percent,
  pick,
  replace,
  resizeTo,
  setField,
  step,
  toFront,
  type Proportion,
} from './screenEditor'

const area = { width: 1600, height: 900 }
const screen = (over: Partial<CustomScreen> = {}): CustomScreen => ({
  id: 'bottom',
  x: 0.5,
  y: 0.25,
  w: 0.25,
  h: 0.25,
  ...over,
})
const keep: Proportion = { keep: true, ratio: 4 / 3, area }
const free: Proportion = { keep: false, ratio: 4 / 3, area }
const layout: CustomLayout = { screens: [screen({ id: 'top', x: 0 }), screen()] }
const close = (a: number, b: number) => expect(a).toBeCloseTo(b, 3)

describe('the screens editor', () => {
  it('keeps each picture’s proportions: the 3DS top screen is wider', () => {
    expect(aspect('nds', 'top', false)).toBeCloseTo(4 / 3)
    expect(aspect('3ds', 'top', false)).toBeCloseTo(5 / 3)
    expect(aspect('3ds', 'bottom', false)).toBeCloseTo(4 / 3)
    // Swapped, the top rectangle shows the bottom screen.
    expect(aspect('3ds', 'top', true)).toBeCloseTo(4 / 3)
    // 4:3 in a 16:9 area: 0.25 of the width is 0.333 of the height.
    close(heightFor(0.25, 4 / 3, area), (0.25 * 1600) / ((4 / 3) * 900))
  })

  it('keeps a screen inside the area and above its smallest size', () => {
    const s = contain(screen({ x: 0.9, y: -0.2, w: 0.3, h: 2 }))
    expect(s.x + s.w).toBeLessThanOrEqual(1)
    expect(s.y).toBe(0)
    expect(s.h).toBe(1)
    expect(contain(screen({ w: 0.01 })).w).toBe(MIN_SIZE)
  })

  it('moves by grid steps, snapping a drag when asked', () => {
    const s = step(screen(), 1, -1)
    close(s.x, 0.5 + 1 / GRID)
    close(s.y, 0.25 - 1 / GRID)
    const snapped = moveTo(screen(), 0.51, 0.2, true)
    close(snapped.x, 16 / GRID)
    close(snapped.y, Math.round(0.2 * GRID) / GRID)
    close(moveTo(screen(), 0.51, 0.2, false).x, 0.51)
    // Never out of the area.
    expect(step(screen({ x: 0.75 }), 1, 0).x).toBe(0.75)
  })

  it('resizes from the corner, the height following the width with the proportions kept', () => {
    const s = resizeTo(screen(), 0.3, 0.9, keep, false)
    close(s.w, 0.3)
    close(s.h, heightFor(0.3, 4 / 3, area))
    const loose = resizeTo(screen(), 0.3, 0.4, free, false)
    close(loose.h, 0.4)
    // Too tall to fit below: both shrink together.
    const tall = resizeTo(screen({ y: 0.8 }), 0.4, 0, keep, false)
    close(tall.y + tall.h, 1)
    close(tall.w / tall.h, (4 / 3) * (900 / 1600))
    // The right stick: one step wider; up and down too when the proportions are kept.
    close(grow(screen(), 1, 0, keep).w, 0.25 + 1 / GRID)
    close(grow(screen(), 0, -1, keep).w, 0.25 - 1 / GRID)
    close(grow(screen(), 0, 1, free).h, 0.25 + 1 / GRID)
  })

  it('takes the numeric fields in per cent, with a comma or a point', () => {
    expect(percent(0.125)).toBe(12.5)
    expect(fromPercent('12,5')).toBe(0.125)
    expect(fromPercent(' 40 ')).toBe(0.4)
    expect(fromPercent('')).toBeNull()
    expect(fromPercent('abc')).toBeNull()
    close(setField(screen(), 'x', 0.1, keep).x, 0.1)
    const h = setField(screen(), 'h', 0.5, keep)
    close(h.h, 0.5)
    close(h.w / h.h, (4 / 3) * (900 / 1600))
    close(setField(screen(), 'h', 0.5, free).w, 0.25)
  })

  it('orders the screens back to front and picks them in turn', () => {
    expect(front(layout)).toBe('bottom')
    const topFirst = toFront(layout, 'top')
    expect(topFirst.screens.map(s => s.id)).toEqual(['bottom', 'top'])
    expect(front(topFirst)).toBe('top')
    expect(pick(layout, 'top', 1)).toBe('bottom')
    expect(pick(layout, 'top', -1)).toBe('bottom')
    expect(pick({ screens: [screen({ id: 'top' })] }, 'top', 1)).toBe('top')
    const moved = replace(layout, screen({ x: 0.1 }))
    expect(moved.screens[1]?.x).toBe(0.1)
    expect(moved.screens[0]?.x).toBe(0)
  })

  it('puts the panel on the side away from the selected screen', () => {
    expect(panelSide(screen({ x: 0.6 }))).toBe('left')
    expect(panelSide(screen({ x: 0.05 }))).toBe('right')
    expect(panelSide(undefined)).toBe('right')
  })

  it('keeps the panel on its side unless the screen would be wholly under it (#156)', () => {
    // The phone's panel: most of the top half, 80% across.
    const panel = { w: 0.8, h: 0.5 }
    // A screen half under the panel, its centre left: the panel stays right.
    expect(keepPanelSide('right', screen({ x: 0.05, y: 0.1, w: 0.9, h: 0.3 }), panel)).toBe('right')
    // Wholly under it on the right, free on the left: it goes left.
    expect(keepPanelSide('right', screen({ x: 0.7, y: 0.1, w: 0.25, h: 0.2 }), panel)).toBe('left')
    // Wholly under it on either side: it stays.
    expect(keepPanelSide('left', screen({ x: 0.3, y: 0.1, w: 0.4, h: 0.2 }), panel)).toBe('left')
    // Below the panel: it stays.
    expect(keepPanelSide('left', screen({ x: 0, y: 0.6, w: 0.3, h: 0.3 }), panel)).toBe('left')
  })
})
