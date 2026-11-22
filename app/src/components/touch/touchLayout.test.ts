import { describe, expect, it } from 'vitest'
import type { ConsoleId, TouchControlId, TouchLayout, TouchOrientation, TouchSetup } from '@/emu'
import {
  activeLayout,
  add,
  defaultLayout,
  minSize,
  missing,
  moveTo,
  nudge,
  orientationOf,
  overlaps,
  pinchSize,
  rectOf,
  remove,
  reservedBottom,
  resizeTo,
  withOpacity,
  type Area,
  type Box,
  type Rect,
} from './touchLayout'

// The controls' own boxes at 100%, as the design system draws them (assets/theme/spacing.css:
// `size-dpad`, `size-pad-stick`, `size-pad-face`, the shoulders `w-pad-shoulder` × `h-touch`,
// Select / Start `h-touch` with their words, the menu `w-48` × `h-touch`).
const BOX: Record<TouchControlId, Box> = {
  dpad: { w: 148, h: 148 },
  circlePad: { w: 128, h: 128 },
  a: { w: 64, h: 64 },
  b: { w: 64, h: 64 },
  x: { w: 64, h: 64 },
  y: { w: 64, h: 64 },
  l: { w: 88, h: 44 },
  r: { w: 88, h: 44 },
  select: { w: 80, h: 44 },
  start: { w: 80, h: 44 },
  menu: { w: 48, h: 44 },
}
const boxOf = (id: TouchControlId) => BOX[id]

// What `ControlId::of` answers (emudome_core::touch).
const AVAILABLE: Record<ConsoleId, TouchControlId[]> = {
  gb: ['dpad', 'b', 'a', 'select', 'start', 'menu'],
  gbc: ['dpad', 'b', 'a', 'select', 'start', 'menu'],
  gba: ['l', 'r', 'dpad', 'b', 'a', 'select', 'start', 'menu'],
  nds: ['l', 'r', 'dpad', 'y', 'x', 'b', 'a', 'select', 'start', 'menu'],
  '3ds': ['l', 'r', 'circlePad', 'dpad', 'y', 'x', 'b', 'a', 'select', 'start', 'menu'],
}
const LIMITS = { sizeMin: 50, sizeMax: 200, opacityMin: 20, opacityMax: 100 }

/** The phones of the card's checks: portrait 390 × 844 and 360 × 740, landscape both ways. */
const AREAS: Area[] = [
  { width: 390, height: 844 },
  { width: 360, height: 740 },
  { width: 844, height: 390 },
  { width: 740, height: 360 },
]
const CONSOLES: ConsoleId[] = ['gb', 'gbc', 'gba', 'nds', '3ds']

function setup(console: ConsoleId, saved: Partial<Record<TouchOrientation, TouchLayout>> = {}) {
  return {
    available: AVAILABLE[console],
    buttons: AVAILABLE[console].filter(id => id !== 'dpad' && id !== 'circlePad' && id !== 'menu'),
    portrait: saved.portrait ?? { controls: [] },
    landscape: saved.landscape ?? { controls: [] },
    ...LIMITS,
  } satisfies TouchSetup
}

describe('default layouts', () => {
  for (const console of CONSOLES) {
    for (const area of AREAS) {
      const o = orientationOf(area)
      it(`${console} ${o} ${area.width}×${area.height}: every control, inside, none on another`, () => {
        const layout = defaultLayout(console, o, area, AVAILABLE[console])
        expect(layout.controls.map(c => c.id)).toEqual(AVAILABLE[console])
        const rects = layout.controls.map(c => ({ id: c.id, r: rectOf(c, area, boxOf(c.id)) }))
        for (const { id, r } of rects) {
          expect(r.left, id).toBeGreaterThanOrEqual(0)
          expect(r.top, id).toBeGreaterThanOrEqual(0)
          expect(r.right, id).toBeLessThanOrEqual(area.width)
          expect(r.bottom, id).toBeLessThanOrEqual(area.height)
        }
        // Two round face buttons touch only when their circles do (a press is hit-tested inside
        // the circle); anything else by its rectangle.
        const round = (id: TouchControlId) => ['a', 'b', 'x', 'y'].includes(id)
        const centre = (r: Rect) => [(r.left + r.right) / 2, (r.top + r.bottom) / 2] as const
        for (let i = 0; i < rects.length; i++)
          for (let j = i + 1; j < rects.length; j++) {
            const [a, b] = [rects[i]!, rects[j]!]
            const [ax, ay] = centre(a.r)
            const [bx, by] = centre(b.r)
            const touching =
              round(a.id) && round(b.id)
                ? Math.hypot(ax - bx, ay - by) < (a.r.right - a.r.left + b.r.right - b.r.left) / 2
                : overlaps(a.r, b.r)
            expect(touching, `${a.id} on ${b.id}`).toBe(false)
          }
        for (const c of layout.controls) {
          expect(c.size).toBeGreaterThanOrEqual(minSize(boxOf(c.id), LIMITS))
          expect(c.behaviour).toBe('tap')
        }
      })
    }
  }

  it('keeps the middle of a DS held sideways free for the touch screen', () => {
    const area = { width: 844, height: 390 }
    // The screens stacked at the height of the area: 260 px wide in the middle.
    const middle = { left: area.width / 2 - 130, right: area.width / 2 + 130, top: 0, bottom: 390 }
    for (const console of ['nds', '3ds'] as const) {
      for (const c of defaultLayout(console, 'landscape', area, AVAILABLE[console]).controls)
        expect(overlaps(rectOf(c, area, boxOf(c.id)), middle), `${console} ${c.id}`).toBe(false)
    }
  })

  it('draws the saved layout only when it is chosen and has controls', () => {
    const area = { width: 844, height: 390 }
    const saved = {
      controls: [
        { id: 'menu' as const, x: 0.1, y: 0.1, size: 100, opacity: 50, behaviour: 'tap' as const },
      ],
    }
    const s = setup('gba', { landscape: saved })
    expect(activeLayout('gba', s, 'landscape', 'custom', area)).toEqual(saved)
    const fallback = defaultLayout('gba', 'landscape', area, AVAILABLE.gba)
    expect(activeLayout('gba', s, 'landscape', 'default', area)).toEqual(fallback)
    expect(activeLayout('gba', setup('gba'), 'landscape', 'custom', area)).toEqual(fallback)
  })
})

describe('geometry', () => {
  const area = { width: 800, height: 400 }
  const a = { id: 'a' as const, x: 0.5, y: 0.5, size: 100, opacity: 100, behaviour: 'tap' as const }

  it('snaps the centre to the 8 px grid and keeps the box inside', () => {
    const moved = moveTo(a, 203, 101, area, BOX.a, true)
    expect([moved.x * 800, moved.y * 400]).toEqual([200, 104])
    const free = moveTo(a, 203, 101, area, BOX.a, false)
    expect(free.x * 800).toBeCloseTo(203, 0)
    expect(free.y * 400).toBeCloseTo(101, 0)
    const edge = moveTo(a, -50, 999, area, BOX.a, true)
    expect([edge.x * 800, edge.y * 400]).toEqual([32, 368])
  })

  it('moves one grid step with the d-pad', () => {
    const n = nudge(a, 1, -1, area, BOX.a)
    expect([n.x * 800, n.y * 400]).toEqual([408, 192])
  })

  it('never makes a control smaller than the touch minimum', () => {
    expect(minSize(BOX.a, LIMITS)).toBe(70) // 64 px × 70% = 44.8
    expect(minSize(BOX.l, LIMITS)).toBe(100) // 44 px tall already
    expect(minSize(BOX.dpad, LIMITS)).toBe(50)
    expect(resizeTo(a, 10, area, BOX.a, LIMITS).size).toBe(70)
    expect(resizeTo(a, 999, area, BOX.a, LIMITS).size).toBe(200)
    expect(resizeTo(a, 127, area, BOX.a, LIMITS).size).toBe(125)
    // Grown at an edge, it moves back inside.
    const corner = { ...a, x: 0.04, y: 0.08 }
    const big = resizeTo(corner, 200, area, BOX.a, LIMITS)
    expect(rectOf(big, area, BOX.a).left).toBeGreaterThanOrEqual(0)
    expect(rectOf(big, area, BOX.a).top).toBeGreaterThanOrEqual(0)
  })

  it('follows a pinch', () => {
    expect(pinchSize(100, 50, 75)).toBe(150)
    expect(pinchSize(100, 0, 75)).toBe(100)
  })

  it('keeps the opacity within the limits', () => {
    expect(withOpacity(a, 5, LIMITS).opacity).toBe(20)
    expect(withOpacity(a, 64.4, LIMITS).opacity).toBe(64)
  })
})

describe('editing', () => {
  const area = { width: 844, height: 390 }
  const layout = defaultLayout('gba', 'landscape', area, AVAILABLE.gba)

  it('removes any control but the menu, and adds it back at its place', () => {
    const without = remove(layout, 'l')
    expect(missing(without, AVAILABLE.gba)).toEqual(['l'])
    expect(remove(without, 'menu')).toBe(without)
    const back = add(without, 'l', 'gba', 'landscape', area)
    expect(back.controls.find(c => c.id === 'l')).toEqual(layout.controls.find(c => c.id === 'l'))
    expect(add(back, 'l', 'gba', 'landscape', area)).toBe(back)
  })
})

describe('the band a portrait layout keeps', () => {
  it('reaches from the highest control of the lower half to the bottom', () => {
    const area = { width: 390, height: 844 }
    const layout = defaultLayout('gba', 'portrait', area, AVAILABLE.gba)
    // L / R / menu at 300 px from the bottom, 22 px half height, a grid step of air.
    expect(reservedBottom(layout, area, boxOf)).toBe(300 + 22 + 8)
    expect(reservedBottom(layout, { width: 844, height: 390 }, boxOf)).toBe(0)
    expect(reservedBottom({ controls: [] }, area, boxOf)).toBe(0)
  })
})
