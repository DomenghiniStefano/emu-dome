// The touch controls' layouts (Trello #29, C05): where each control sits, per console and
// orientation. A layout is each control's centre as fractions of the game area (the window), its
// size and opacity in per cent and what it does under the finger (`TouchLayout`, generated from
// emudome_core::touch, which also says which controls a console has and the size / opacity
// limits: `TouchSetup`). The defaults are written here, anchored to the window's edges in CSS
// pixels so the clusters keep their shape on every phone, and turned into fractions for the area
// they are drawn in. Moving, resizing, snapping to the 8 px grid and the band a portrait layout
// keeps for itself are here too, pure. A control's own box (its size at 100%) is the design
// system's (`size-dpad`, `size-pad-face`…): the screens measure it and pass it in.
import type {
  ConsoleId,
  TouchControl,
  TouchControlId,
  TouchLayout,
  TouchOrientation,
  TouchSetup,
} from '@/emu'

/** The area the fractions are of, in CSS pixels. */
export interface Area {
  width: number
  height: number
}

/** A control's own box at 100%, CSS pixels. */
export interface Box {
  w: number
  h: number
}

/** The editor's snapping grid (owner's card: 8 px). */
export const GRID_PX = 8
/** The smallest a touch target gets, CSS pixels (README: 44). */
export const TOUCH_MIN_PX = 44
/** A size step of the editor (per cent). */
export const SIZE_STEP = 5
/** The size and opacity a new control starts with. */
export const DEFAULT_SIZE = 100
export const DEFAULT_OPACITY = 100

/** Where a default control sits: an anchor of the area (0, ½ or 1 of each side) and the offset
 * of its centre from it, in CSS pixels. */
type Anchor = readonly [ax: 0 | 0.5 | 1, ay: 0 | 0.5 | 1, dx: number, dy: number]
type Placement = Partial<Record<TouchControlId, Anchor>>

/** The consoles that share a default layout. */
type Family = 'gb' | 'gba' | 'nds' | '3ds'
const FAMILY: Record<ConsoleId, Family> = {
  gb: 'gb',
  gbc: 'gb',
  gba: 'gba',
  nds: 'nds',
  '3ds': '3ds',
}

/** Four face buttons around a centre (the DS / 3DS diamond): A right, B below, X above, Y left. */
function diamond(ax: 0 | 1, ay: 0 | 1, cx: number, cy: number, arm = 48): Placement {
  return {
    a: [ax, ay, cx + arm, cy],
    b: [ax, ay, cx, cy + arm],
    x: [ax, ay, cx, cy - arm],
    y: [ax, ay, cx - arm, cy],
  }
}

// Landscape: the game in the middle, the controls in the bands on either side; the DS / 3DS keep
// the middle free (its lower half is the touch screen), so their Select / Start / menu sit in the
// side bands too. Portrait: the controls in a band at the bottom, the game above it.
const DEFAULTS: Record<Family, Record<TouchOrientation, Placement>> = {
  gb: {
    landscape: {
      dpad: [0, 1, 104, -110],
      b: [1, 1, -140, -100],
      a: [1, 1, -64, -150],
      select: [0.5, 1, -50, -30],
      start: [0.5, 1, 50, -30],
      menu: [0.5, 0, 0, 30],
    },
    portrait: {
      dpad: [0, 1, 96, -150],
      b: [1, 1, -118, -122],
      a: [1, 1, -60, -190],
      select: [0.5, 1, -50, -36],
      start: [0.5, 1, 50, -36],
      menu: [0.5, 1, 0, -300],
    },
  },
  gba: {
    landscape: {
      l: [0, 0, 60, 38],
      r: [1, 0, -60, 38],
      dpad: [0, 1, 104, -110],
      b: [1, 1, -140, -100],
      a: [1, 1, -64, -150],
      select: [0.5, 1, -50, -30],
      start: [0.5, 1, 50, -30],
      menu: [0.5, 0, 0, 30],
    },
    portrait: {
      l: [0, 1, 60, -300],
      r: [1, 1, -60, -300],
      dpad: [0, 1, 96, -150],
      b: [1, 1, -118, -122],
      a: [1, 1, -60, -190],
      select: [0.5, 1, -50, -36],
      start: [0.5, 1, 50, -36],
      menu: [0.5, 1, 0, -300],
    },
  },
  nds: {
    landscape: {
      l: [0, 0, 60, 38],
      r: [1, 0, -60, 38],
      select: [0, 0, 158, 38],
      start: [1, 0, -158, 38],
      dpad: [0, 1, 104, -110],
      ...diamond(1, 1, -110, -120),
      menu: [1, 0, -60, 106],
    },
    portrait: {
      l: [0, 1, 60, -300],
      r: [1, 1, -60, -300],
      dpad: [0, 1, 96, -150],
      ...diamond(1, 1, -104, -160),
      select: [0.5, 1, -50, -36],
      start: [0.5, 1, 50, -36],
      menu: [0.5, 1, 0, -300],
    },
  },
  '3ds': {
    landscape: {
      l: [0, 0, 60, 38],
      r: [1, 0, -60, 38],
      select: [0, 0, 158, 38],
      start: [1, 0, -158, 38],
      circlePad: [0, 0, 84, 134],
      dpad: [0, 1, 96, -88],
      ...diamond(1, 1, -110, -120),
      menu: [1, 0, -60, 106],
    },
    portrait: {
      l: [0, 1, 60, -400],
      r: [1, 1, -60, -400],
      circlePad: [0, 1, 90, -300],
      dpad: [0, 1, 96, -132],
      ...diamond(1, 1, -104, -200),
      select: [0.5, 1, -50, -30],
      start: [0.5, 1, 50, -30],
      menu: [0.5, 1, 0, -400],
    },
  },
}

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi)
/** Fractions are stored with 4 decimals (`Layouts::to_items`). */
const round4 = (v: number) => Math.round(v * 10_000) / 10_000

/** A control of `id` at the centre (`cx`, `cy`) px of `area`, with the standard look. */
function placed(id: TouchControlId, cx: number, cy: number, area: Area): TouchControl {
  return {
    id,
    x: round4(clamp(cx / area.width, 0, 1)),
    y: round4(clamp(cy / area.height, 0, 1)),
    size: DEFAULT_SIZE,
    opacity: DEFAULT_OPACITY,
    behaviour: 'tap',
  }
}

/** Where `id` sits by default on `console` held `orientation` (null: no default place). */
function defaultPlace(
  console: ConsoleId,
  orientation: TouchOrientation,
  id: TouchControlId,
  area: Area,
): TouchControl | null {
  const anchor = DEFAULTS[FAMILY[console]][orientation][id]
  if (!anchor) return null
  const [ax, ay, dx, dy] = anchor
  return placed(id, ax * area.width + dx, ay * area.height + dy, area)
}

/** The default layout of `console` held `orientation` in `area`: the controls `available` (the
 * console's, `TouchSetup.available`) in their order. */
export function defaultLayout(
  console: ConsoleId,
  orientation: TouchOrientation,
  area: Area,
  available: readonly TouchControlId[],
): TouchLayout {
  return {
    controls: available.flatMap(id => defaultPlace(console, orientation, id, area) ?? []),
  }
}

/** `touch.tport` / `touch.tland`: the default layout, or the user's own. */
export type LayoutChoice = 'default' | 'custom'

/** A setting's value as a layout choice: anything unknown is the default. */
export const layoutChoice = (value: unknown): LayoutChoice =>
  value === 'custom' ? 'custom' : 'default'

/** The orientation of an area: portrait when taller than wide. */
export const orientationOf = (area: Area): TouchOrientation =>
  area.height > area.width ? 'portrait' : 'landscape'

/** The layout drawn: the user's own when chosen and saved (only the controls the console has),
 * else the default. */
export function activeLayout(
  console: ConsoleId,
  setup: TouchSetup,
  orientation: TouchOrientation,
  choice: LayoutChoice,
  area: Area,
): TouchLayout {
  const saved = setup[orientation].controls.filter(c => setup.available.includes(c.id))
  if (choice === 'custom' && saved.length > 0) return { controls: saved }
  return defaultLayout(console, orientation, area, setup.available)
}

// ---- Geometry ----

/** A control's rectangle in `area` at its size, CSS pixels. */
export interface Rect {
  left: number
  top: number
  right: number
  bottom: number
}

export function rectOf(c: TouchControl, area: Area, box: Box): Rect {
  const w = (box.w * c.size) / 100
  const h = (box.h * c.size) / 100
  const cx = c.x * area.width
  const cy = c.y * area.height
  return { left: cx - w / 2, top: cy - h / 2, right: cx + w / 2, bottom: cy + h / 2 }
}

export const overlaps = (a: Rect, b: Rect) =>
  a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom

/** The smallest size (per cent) that keeps `box` at least 44 px on its shorter side, on the
 * size grid and within the limits. */
export function minSize(box: Box, limits: Pick<TouchSetup, 'sizeMin' | 'sizeMax'>): number {
  const short = Math.min(box.w, box.h)
  const touch = short > 0 ? Math.ceil(((TOUCH_MIN_PX / short) * 100) / SIZE_STEP) * SIZE_STEP : 0
  return clamp(Math.max(limits.sizeMin, touch), limits.sizeMin, limits.sizeMax)
}

/** `c` with its centre at (`cx`, `cy`) px, snapped to the grid when `snap`, its box kept inside
 * the area. */
export function moveTo(
  c: TouchControl,
  cx: number,
  cy: number,
  area: Area,
  box: Box,
  snap: boolean,
): TouchControl {
  const w = (box.w * c.size) / 100
  const h = (box.h * c.size) / 100
  const grid = (v: number) => (snap ? Math.round(v / GRID_PX) * GRID_PX : v)
  const x = clamp(
    grid(cx),
    Math.min(w / 2, area.width / 2),
    Math.max(area.width - w / 2, area.width / 2),
  )
  const y = clamp(
    grid(cy),
    Math.min(h / 2, area.height / 2),
    Math.max(area.height - h / 2, area.height / 2),
  )
  return { ...c, x: round4(x / area.width), y: round4(y / area.height) }
}

/** `c` moved by one grid step (the d-pad and the arrows in the editor). */
export function nudge(
  c: TouchControl,
  dx: -1 | 0 | 1,
  dy: -1 | 0 | 1,
  area: Area,
  box: Box,
): TouchControl {
  return moveTo(
    c,
    c.x * area.width + dx * GRID_PX,
    c.y * area.height + dy * GRID_PX,
    area,
    box,
    true,
  )
}

/** `c` at `size` per cent (on the size grid, at least the touch minimum, within the limits),
 * kept inside the area. */
export function resizeTo(
  c: TouchControl,
  size: number,
  area: Area,
  box: Box,
  limits: Pick<TouchSetup, 'sizeMin' | 'sizeMax'>,
): TouchControl {
  const stepped = Math.round(size / SIZE_STEP) * SIZE_STEP
  const next = { ...c, size: clamp(stepped, minSize(box, limits), limits.sizeMax) }
  return moveTo(next, c.x * area.width, c.y * area.height, area, box, false)
}

/** The size a pinch gives: the size it started at times how far the fingers spread. */
export const pinchSize = (start: number, startDistance: number, distance: number) =>
  startDistance > 0 ? (start * distance) / startDistance : start

/** `c`'s opacity, within the limits (per cent). */
export function withOpacity(
  c: TouchControl,
  opacity: number,
  limits: Pick<TouchSetup, 'opacityMin' | 'opacityMax'>,
): TouchControl {
  return { ...c, opacity: clamp(Math.round(opacity), limits.opacityMin, limits.opacityMax) }
}

// ---- Editing a layout ----

/** `layout` with `c` in place of the control of the same id. */
export function replace(layout: TouchLayout, c: TouchControl): TouchLayout {
  return { controls: layout.controls.map(x => (x.id === c.id ? c : x)) }
}

/** `layout` without `id` (the menu stays: every layout keeps it). */
export function remove(layout: TouchLayout, id: TouchControlId): TouchLayout {
  if (id === 'menu') return layout
  return { controls: layout.controls.filter(c => c.id !== id) }
}

/** The controls of the console that `layout` does not show, in the console's order. */
export const missing = (layout: TouchLayout, available: readonly TouchControlId[]) =>
  available.filter(id => !layout.controls.some(c => c.id === id))

/** `layout` with `id` back: at its default place, else in the middle of the area. */
export function add(
  layout: TouchLayout,
  id: TouchControlId,
  console: ConsoleId,
  orientation: TouchOrientation,
  area: Area,
): TouchLayout {
  if (layout.controls.some(c => c.id === id)) return layout
  const c =
    defaultPlace(console, orientation, id, area) ??
    placed(id, area.width / 2, area.height / 2, area)
  return { controls: [...layout.controls, c] }
}

/**
 * How much of the bottom of a portrait area the layout keeps for itself, CSS pixels: from the
 * top of the highest control in the lower half (and a grid step of air) to the bottom. The game
 * is drawn above it (`setInset`). 0 in landscape or with no control there.
 */
export function reservedBottom(
  layout: TouchLayout,
  area: Area,
  boxOf: (id: TouchControlId) => Box,
): number {
  if (orientationOf(area) !== 'portrait') return 0
  const tops = layout.controls.filter(c => c.y >= 0.5).map(c => rectOf(c, area, boxOf(c.id)).top)
  if (tops.length === 0) return 0
  return Math.max(0, Math.round(area.height - Math.min(...tops) + GRID_PX))
}
