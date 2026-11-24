import { describe, expect, it } from 'vitest'
import type { ConsoleId, TouchControlId, TouchLayout, TouchOrientation, TouchSetup } from '@/emu'
import {
  activeLayout,
  add,
  defaultBand,
  defaultLayout,
  isDualScreen,
  minSize,
  missing,
  moveTo,
  nudge,
  orientationOf,
  overlaps,
  playInset,
  playPlacement,
  pinchSize,
  rectOf,
  remove,
  reservedBottom,
  reservedInset,
  reservedSides,
  resizeTo,
  SIDE_EDGE_PX,
  SIDE_GAP_PX,
  STRIP_PX,
  withOpacity,
  type Area,
  type Box,
  type Rect,
} from './touchLayout'
import { NO_INSET, type InsetPx } from '@/components/game/inset'
import { touchShown } from './touchVisibility'

// The controls' own boxes at 100%, as the design system draws them (design-system/tokens/spacing.css:
// `size-dpad`, `size-pad-stick`, `size-pad-face`, the shoulders `w-pad-shoulder` × `h-pad-strip`,
// Select / Start `w-pad-system` × `h-pad-system-h`, the menu `size-pad-strip`).
const BOX: Record<TouchControlId, Box> = {
  dpad: { w: 116, h: 116 },
  circlePad: { w: 104, h: 104 },
  a: { w: 48, h: 48 },
  b: { w: 48, h: 48 },
  x: { w: 48, h: 48 },
  y: { w: 48, h: 48 },
  l: { w: 84, h: 36 },
  r: { w: 84, h: 36 },
  select: { w: 60, h: 32 },
  start: { w: 60, h: 32 },
  menu: { w: 36, h: 36 },
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

/** The phones of the card's checks: portrait 390 × 844, 360 × 740, 412 × 892 and the Galaxy S23
 * as it reports itself (360 × 780, #29), landscape all ways. */
const AREAS: Area[] = [
  { width: 390, height: 844 },
  { width: 360, height: 740 },
  { width: 412, height: 892 },
  { width: 360, height: 780 },
  { width: 844, height: 390 },
  { width: 740, height: 360 },
  { width: 892, height: 412 },
  { width: 780, height: 360 },
]
const CONSOLES: ConsoleId[] = ['gb', 'gbc', 'gba', 'nds', '3ds']
/** The narrowest phones (320 px): the two-button consoles fit there too; the DS / 3DS diamond
 * needs 360. */
const NARROW: Area = { width: 320, height: 640 }
const areasOf = (console: ConsoleId) =>
  console === 'nds' || console === '3ds' ? AREAS : [...AREAS, NARROW]

function setup(console: ConsoleId, saved: Partial<Record<TouchOrientation, TouchLayout>> = {}) {
  return {
    available: AVAILABLE[console],
    buttons: AVAILABLE[console].filter(id => id !== 'dpad' && id !== 'circlePad' && id !== 'menu'),
    portrait: saved.portrait ?? { controls: [], screens: [] },
    landscape: saved.landscape ?? { controls: [], screens: [] },
    screenShapes: [],
    ...LIMITS,
  } satisfies TouchSetup
}

describe('default layouts', () => {
  for (const console of CONSOLES) {
    for (const area of areasOf(console)) {
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

  it('draws the saved layout only when it is chosen and has controls', () => {
    const area = { width: 844, height: 390 }
    const saved = {
      controls: [
        { id: 'menu' as const, x: 0.1, y: 0.1, size: 100, opacity: 50, behaviour: 'tap' as const },
      ],
      screens: [],
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
    expect([edge.x * 800, edge.y * 400]).toEqual([24, 376])
  })

  it('moves one grid step with the d-pad', () => {
    const n = nudge(a, 1, -1, area, BOX.a)
    expect([n.x * 800, n.y * 400]).toEqual([408, 192])
  })

  it('never makes a control smaller than the touch minimum, nor a compact one than its own', () => {
    expect(minSize(BOX.a, LIMITS)).toBe(95) // 48 px × 95% = 45.6
    expect(minSize(BOX.l, LIMITS)).toBe(100) // 36 px tall by design: never under that
    expect(minSize(BOX.select, LIMITS)).toBe(100)
    expect(minSize(BOX.dpad, LIMITS)).toBe(50)
    expect(minSize({ w: 64, h: 64 }, LIMITS)).toBe(70) // 64 px × 70% = 44.8
    expect(resizeTo(a, 10, area, BOX.a, LIMITS).size).toBe(95)
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
    // L / R at 194 px from the bottom, 18 px half height, a grid step of air.
    expect(reservedBottom(layout, area, boxOf)).toBe(194 + 18 + 8)
    expect(reservedBottom(layout, { width: 844, height: 390 }, boxOf)).toBe(0)
    expect(reservedBottom({ controls: [], screens: [] }, area, boxOf)).toBe(0)
  })
})

describe('the edges a layout keeps clear of the game (#29)', () => {
  /** The part of `area` the game may use once `reservedInset` is kept clear. */
  const free = (area: Area, layout: TouchLayout): Rect => {
    const i = reservedInset(layout, area, boxOf)
    return { left: i.left, top: i.top, right: area.width - i.right, bottom: area.height - i.bottom }
  }

  for (const console of CONSOLES.filter(c => !isDualScreen(c))) {
    for (const area of areasOf(console)) {
      const o = orientationOf(area)
      it(`${console} ${o} ${area.width}×${area.height}: no default control over the game's part`, () => {
        const layout = defaultLayout(console, o, area, AVAILABLE[console])
        const game = free(area, layout)
        for (const c of layout.controls)
          expect(overlaps(rectOf(c, area, boxOf(c.id)), game), c.id).toBe(false)
        // The game keeps a useful part of the window: a third of it at least, either way.
        if (o === 'landscape') expect(game.right - game.left).toBeGreaterThan(area.width / 3)
        else expect(game.bottom - game.top).toBeGreaterThan(area.height / 3)
      })
    }
  }

  it('upright: the band at the bottom, the game against it', () => {
    const area = { width: 412, height: 892 }
    const layout = defaultLayout('gba', 'portrait', area, AVAILABLE.gba)
    expect(reservedInset(layout, area, boxOf)).toEqual({
      left: 0,
      top: 0,
      right: 0,
      bottom: 194 + 18 + 8,
      align: 'bottom',
      between: 0,
    })
  })

  it('on the side: the two columns, the game centred between them', () => {
    // GBA at 892 × 412: the d-pad (116 px centred 82 px in) and B (48 px, 112 px from the right).
    const area = { width: 892, height: 412 }
    const layout = defaultLayout('gba', 'landscape', area, AVAILABLE.gba)
    expect(reservedSides(layout, area, boxOf)).toEqual({ left: 82 + 58 + 8, right: 112 + 24 + 8 })
    expect(reservedInset(layout, area, boxOf)).toEqual({
      left: 148,
      top: 0,
      right: 144,
      bottom: 0,
      align: 'centre',
      between: 0,
    })
    expect(reservedSides(layout, { width: 412, height: 892 }, boxOf)).toEqual({ left: 0, right: 0 })
  })

  it('leaves out a control the user put across the middle, and keeps nothing with none', () => {
    const area = { width: 844, height: 390 }
    const menu = {
      id: 'menu' as const,
      x: 0.5,
      y: 0.1,
      size: 100,
      opacity: 100,
      behaviour: 'tap' as const,
    }
    const a = { ...menu, id: 'a' as const, x: 0.9, y: 0.5 }
    expect(reservedSides({ controls: [menu, a], screens: [] }, area, boxOf)).toEqual({
      left: 0,
      right: Math.round(844 - (0.9 * 844 - 24) + 8),
    })
    expect(reservedInset({ controls: [], screens: [] }, area, boxOf)).toEqual({
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
      align: 'centre',
      between: 0,
    })
    expect(
      reservedInset({ controls: [], screens: [] }, { width: 390, height: 844 }, boxOf).bottom,
    ).toBe(0)
  })
})

// ---- DS and 3DS on a phone (owner, 2026-11-22): the screens apart and large ----

/** The two screens' size at 1×. */
const SCREENS: Record<'nds' | '3ds', { top: Box; bottom: Box }> = {
  nds: { top: { w: 256, h: 192 }, bottom: { w: 256, h: 192 } },
  '3ds': { top: { w: 400, h: 240 }, bottom: { w: 320, h: 240 } },
}

/**
 * Where Rust puts the two screens with `inset` and the `auto` layout filling its room
 * (emudome_core::screens, its tests `a_ds_upright_on_the_s23_…`, `a_3ds_on_the_phone_…`): the
 * checks below hold the controls against them. Upright: the top screen against the safe top, the
 * touch screen against the band; on its side: side by side at the top.
 */
function phoneScreens(console: 'nds' | '3ds', area: Area, inset: InsetPx) {
  const { top: t, bottom: b } = SCREENS[console]
  if (orientationOf(area) === 'portrait') {
    const end = area.height - inset.bottom
    const s = Math.min(
      (area.width - inset.left - inset.right) / Math.max(t.w, b.w),
      (end - inset.top - inset.between) / (t.h + b.h),
    )
    const rect = (box: Box, y: number): Rect => {
      const left = inset.left + (area.width - inset.left - inset.right - box.w * s) / 2
      return { left, top: y, right: left + box.w * s, bottom: y + box.h * s }
    }
    return { top: rect(t, inset.top), bottom: rect(b, end - b.h * s) }
  }
  const free = area.width - inset.left - inset.right
  const s = Math.min((free - inset.between) / (t.w + b.w), (area.height - inset.top) / t.h)
  const rect = (box: Box, x: number): Rect => ({
    left: x,
    top: inset.top,
    right: x + box.w * s,
    bottom: inset.top + box.h * s,
  })
  return { top: rect(t, inset.left), bottom: rect(b, inset.left + t.w * s + inset.between) }
}

/** The S23 as it reports itself (360 × 780) with its camera: 38 px at the top upright, on the
 * left on its side; and a 412 × 892 phone with a 32 px camera edge. */
const S23_UP: Area = { width: 360, height: 780, safe: { top: 38, right: 0, left: 0 } }
const S23_SIDE: Area = { width: 780, height: 360, safe: { top: 0, right: 0, left: 38 } }
const WIDE_UP: Area = { width: 412, height: 892, safe: { top: 32, right: 0, left: 0 } }
const WIDE_SIDE: Area = { width: 892, height: 412, safe: { top: 0, right: 0, left: 32 } }

describe('two screens on a phone', () => {
  const DUAL = ['nds', '3ds'] as const
  const PHONES = [...AREAS, S23_UP, S23_SIDE, WIDE_UP, WIDE_SIDE]

  it('keeps the band, the strip and the safe top upright, the camera edge on its side', () => {
    const up = defaultLayout('nds', 'portrait', S23_UP, AVAILABLE.nds)
    expect(reservedInset(up, S23_UP, boxOf, true)).toEqual({
      left: 0,
      top: 38,
      right: 0,
      bottom: 166,
      align: 'apart',
      between: STRIP_PX,
    })
    const side = defaultLayout('nds', 'landscape', S23_SIDE, AVAILABLE.nds)
    expect(reservedInset(side, S23_SIDE, boxOf, true)).toEqual({
      left: 38,
      top: 0,
      right: SIDE_EDGE_PX,
      bottom: 0,
      align: 'top',
      between: SIDE_GAP_PX,
    })
    // The one-screen rule is unchanged without the flag.
    expect(reservedInset(up, S23_UP, boxOf).align).toBe('bottom')
  })

  it('gives the S23 the mockup: both DS screens 360 x 270 edge to edge upright', () => {
    const up = defaultLayout('nds', 'portrait', S23_UP, AVAILABLE.nds)
    const screens = phoneScreens('nds', S23_UP, reservedInset(up, S23_UP, boxOf, true))
    // Top 38-308, the strip 308-344, the touch screen 344-614, the band below.
    expect(screens.top).toEqual({ left: 0, top: 38, right: 360, bottom: 308 })
    expect(screens.bottom).toEqual({ left: 0, top: 344, right: 360, bottom: 614 })
    // On its side: 359 x 269.25 each, side by side after the camera's edge.
    const side = defaultLayout('nds', 'landscape', S23_SIDE, AVAILABLE.nds)
    const apart = phoneScreens('nds', S23_SIDE, reservedInset(side, S23_SIDE, boxOf, true))
    expect(apart.top).toEqual({ left: 38, top: 0, right: 397, bottom: 269.25 })
    expect(apart.bottom).toEqual({ left: 405, top: 0, right: 764, bottom: 269.25 })
    // The 3DS upright: its top screen full width too.
    const n3ds = defaultLayout('3ds', 'portrait', S23_UP, AVAILABLE['3ds'])
    const big = phoneScreens('3ds', S23_UP, reservedInset(n3ds, S23_UP, boxOf, true))
    expect(big.top.right - big.top.left).toBe(360)
  })

  it('gives a 412 x 892 phone both DS screens 412 x 309 upright, 418 x 313.5 on its side', () => {
    const up = defaultLayout('nds', 'portrait', WIDE_UP, AVAILABLE.nds)
    const screens = phoneScreens('nds', WIDE_UP, reservedInset(up, WIDE_UP, boxOf, true))
    expect(screens.top).toEqual({ left: 0, top: 32, right: 412, bottom: 341 })
    expect(screens.bottom).toEqual({ left: 0, top: 417, right: 412, bottom: 726 })
    const side = defaultLayout('nds', 'landscape', WIDE_SIDE, AVAILABLE.nds)
    const apart = phoneScreens('nds', WIDE_SIDE, reservedInset(side, WIDE_SIDE, boxOf, true))
    expect(apart.top).toEqual({ left: 32, top: 0, right: 450, bottom: 313.5 })
    expect(apart.bottom).toEqual({ left: 458, top: 0, right: 876, bottom: 313.5 })
    const n3ds = defaultLayout('3ds', 'portrait', WIDE_UP, AVAILABLE['3ds'])
    const big = phoneScreens('3ds', WIDE_UP, reservedInset(n3ds, WIDE_UP, boxOf, true))
    expect(big.top.right - big.top.left).toBe(412)
  })

  for (const console of DUAL) {
    for (const area of PHONES) {
      const o = orientationOf(area)
      const name = `${console} ${o} ${area.width}×${area.height}${area.safe ? ' (camera)' : ''}`
      const layout = defaultLayout(console, o, area, AVAILABLE[console])
      const inset = reservedInset(layout, area, boxOf, true)
      const screens = phoneScreens(console, area, inset)
      const rect = (id: TouchControlId) => {
        const c = layout.controls.find(x => x.id === id)!
        return rectOf(c, area, boxOf(c.id))
      }

      if (o === 'portrait') {
        it(`${name}: L, menu and R in the strip, the rest under the touch screen`, () => {
          // The band the default declares is the one it keeps.
          expect(reservedBottom(layout, area, boxOf, 0.5)).toBe(defaultBand(console, o))
          for (const c of layout.controls) {
            const r = rect(c.id)
            // The strip is one control tall: its edges may touch the screens' to the rounding of
            // the stored fractions (4 decimals).
            const inner = { left: r.left, right: r.right, top: r.top + 0.5, bottom: r.bottom - 0.5 }
            expect(overlaps(inner, screens.top), `${c.id} on the top screen`).toBe(false)
            expect(overlaps(inner, screens.bottom), `${c.id} on the touch screen`).toBe(false)
            if (['l', 'menu', 'r'].includes(c.id)) {
              expect(r.top, c.id).toBeGreaterThanOrEqual(screens.top.bottom - 0.5)
              expect(r.bottom, c.id).toBeLessThanOrEqual(screens.bottom.top + 0.5)
            } else expect(r.top, c.id).toBeGreaterThanOrEqual(screens.bottom.bottom)
          }
          // The screens keep a useful part: each at least a fifth of the height.
          expect(screens.top.bottom - screens.top.top).toBeGreaterThan(area.height / 5)
        })
      } else {
        it(`${name}: the screens at the top, the controls over their lower part only`, () => {
          for (const s of [screens.top, screens.bottom]) {
            const middle = (s.top + s.bottom) / 2
            for (const c of layout.controls)
              if (overlaps(rect(c.id), s))
                expect(rect(c.id).top, `${c.id} over a screen's upper half`).toBeGreaterThan(middle)
          }
          // Side by side, as large as the width allows.
          expect(screens.bottom.right).toBeCloseTo(area.width - inset.right, 3)
        })
      }
    }
  }

  it('places the strip from the safe top: the menu moves with the camera', () => {
    const plain = defaultLayout('nds', 'portrait', { width: 360, height: 780 }, AVAILABLE.nds)
    const camera = defaultLayout('nds', 'portrait', S23_UP, AVAILABLE.nds)
    const menuY = (l: TouchLayout) => l.controls.find(c => c.id === 'menu')!.y * 780
    // The strip's middle, 308-344.
    expect(menuY(camera)).toBeCloseTo(326, 0)
    expect(menuY(camera) - menuY(plain)).toBeCloseTo(19, 0)
  })

  it('adds a control back at its place in the strip', () => {
    const layout = defaultLayout('nds', 'portrait', S23_UP, AVAILABLE.nds)
    const back = add(remove(layout, 'l'), 'l', 'nds', 'portrait', S23_UP)
    expect(back.controls.find(c => c.id === 'l')).toEqual(layout.controls.find(c => c.id === 'l'))
  })
})

describe('the placement in use', () => {
  const menu = {
    id: 'menu' as const,
    x: 0.1,
    y: 0.1,
    size: 100,
    opacity: 100,
    behaviour: 'tap' as const,
  }
  const main = { id: 'main', x: 0, y: 0.1, w: 1, h: 0.4 } as const
  const side = { id: 'main', x: 0.25, y: 0, w: 0.5, h: 0.7 } as const
  const upright = { width: 360, height: 780 }
  const sideways = { width: 780, height: 360 }

  it("activeLayout keeps each orientation's own screens", () => {
    const s = setup('gba', {
      portrait: { controls: [menu], screens: [main] },
      landscape: { controls: [menu], screens: [side] },
    })
    expect(activeLayout('gba', s, 'portrait', 'custom', upright).screens).toEqual([main])
    expect(activeLayout('gba', s, 'landscape', 'custom', sideways).screens).toEqual([side])
    expect(activeLayout('gba', s, 'portrait', 'default', upright).screens).toEqual([])
    const noControls = setup('gba', { portrait: { controls: [], screens: [main] } })
    expect(activeLayout('gba', noControls, 'portrait', 'custom', upright).screens).toEqual([])
  })

  it('playPlacement withdraws only when the controls are out of use', () => {
    const layout = { controls: [menu], screens: [main] }
    expect(playPlacement(layout, true)).toEqual([main])
    expect(playPlacement(layout, false)).toEqual([])
    const base = { touch: true, inGame: true, menuOpen: false, editing: false, controllers: 1 }
    expect(touchShown({ ...base, hideWithPad: true })).toBe(false)
    expect(touchShown({ ...base, hideWithPad: false })).toBe(true)
  })

  it('playInset keeps no edge while a placement is in use', () => {
    const band = { ...NO_INSET, bottom: 220, align: 'bottom' as const }
    expect(playInset({ controls: [menu], screens: [main] }, true, () => band)).toEqual(NO_INSET)
    expect(playInset({ controls: [menu], screens: [] }, true, () => band)).toEqual(band)
    expect(playInset({ controls: [menu], screens: [] }, false, () => band)).toEqual(NO_INSET)
  })
})
