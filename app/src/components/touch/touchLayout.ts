// The touch controls' layouts (Trello #29, C05): where each control sits, per console and
// orientation. A layout is each control's centre as fractions of the game area (the window), its
// size and opacity in per cent and what it does under the finger (`TouchLayout`, generated from
// emudome_core::touch, which also says which controls a console has and the size / opacity
// limits: `TouchSetup`). The defaults are written here, anchored to the window's edges in CSS
// pixels so the clusters keep their shape on every phone, and turned into fractions for the area
// they are drawn in. Moving, resizing, snapping to the 8 px grid and the band a portrait layout
// keeps for itself are here too, pure. A control's own box (its size at 100%) is the design
// system's (`size-dpad`, `size-pad-face`…): the screens measure it and pass it in.
//
// DS and 3DS on a phone (owner, 2026-11-22): the two screens large and apart, composed by Rust
// (`screens.cycle` `auto`, emudome_core::screens). Upright the top screen under the camera, the
// strip between the screens holding L, the menu and R, the touch screen on the band of the d-pad,
// A B X Y, Select and Start; on its side the screens side by side at the top, the controls over
// the strip below and the screens' lower outer corners. The layout tells Rust its part with
// `reservedInset`: the band and the strip upright, the camera's edge on its side.
import type {
  ConsoleId,
  TouchControl,
  TouchControlId,
  TouchLayout,
  TouchOrientation,
  TouchSetup,
} from '@/emu'
import { NO_INSET, type InsetPx } from '@/components/game/inset'

/** The window's edges the system keeps (the camera's notch), CSS pixels. */
export interface SafeEdges {
  top: number
  right: number
  left: number
}

/** The area the fractions are of, in CSS pixels, and its safe edges (none when not given). */
export interface Area {
  width: number
  height: number
  safe?: SafeEdges
}

const NO_SAFE: SafeEdges = Object.freeze({ top: 0, right: 0, left: 0 })
const safeOf = (area: Area): SafeEdges => area.safe ?? NO_SAFE

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

/** Where a default control sits: an anchor of the area (0, ½ or 1 of each side, or across the
 * middle of the strip between a DS's screens upright, `STRIP`) and the offset of its centre from
 * it, in CSS pixels. */
type Anchor = readonly [ax: 0 | 0.5 | 1, ay: 0 | 0.5 | 1 | typeof STRIP, dx: number, dy: number]
type Placement = Partial<Record<TouchControlId, Anchor>>
/** A default, and for a two-screen console upright the band at the bottom it keeps (its
 * controls' highest top and a grid step, CSS px): the strip's middle is halfway between the top
 * of the window's safe part and the band (Rust puts the screens against both, `Align::Apart`). */
interface Defaults {
  place: Placement
  band?: number
}

/** The anchor in the middle of the strip between the two screens, upright. */
const STRIP = 'strip'
/** The strip between the two screens upright (L, menu and R, a touch target tall), and the line
 * between them on its side: `Inset::between`, CSS px. */
export const STRIP_PX = TOUCH_MIN_PX
export const SIDE_GAP_PX = GRID_PX
/** On its side, the least the screens keep from each edge of the window (the camera's edge
 * keeps its notch when larger). */
export const SIDE_EDGE_PX = 2 * GRID_PX
/** Upright, from this width the DS / 3DS Select and Start fit in a column between the d-pad and
 * the face buttons, which leaves the screens their full width on a 412 px phone; narrower, they
 * go under both and the band grows. */
export const DUAL_WIDE_PX = 404

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

// Landscape: the game in the middle, the controls in the columns on either side, Select / Start /
// menu included (#29: nothing over the picture; the game is fitted between the columns,
// `reservedInset`). Portrait: the controls in a band at the bottom, the game right above it.
// The two-screen consoles: `DUAL` (upright) and `DUAL_LANDSCAPE`.
const DEFAULTS: Record<'gb' | 'gba', Record<TouchOrientation, Placement>> = {
  gb: {
    landscape: {
      dpad: [0, 1, 104, -110],
      b: [1, 1, -140, -100],
      a: [1, 1, -64, -150],
      select: [0, 0, 60, 38],
      start: [1, 0, -60, 38],
      menu: [1, 0, -150, 38],
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
      select: [0, 0, 60, 96],
      start: [1, 0, -60, 96],
      menu: [1, 0, -150, 38],
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
}

/** The strip's L, menu and R, upright. */
const STRIP_ROW: Placement = {
  l: [0, STRIP, 60, 0],
  menu: [0.5, STRIP, 0, 0],
  r: [1, STRIP, -60, 0],
}
/** Upright from `DUAL_WIDE_PX`: Select over Start in the column between the left cluster (its
 * right edge 154 px in) and the face buttons (their left edge 168 px from the right). */
const SELECT_START_COLUMN: Placement = {
  select: [0.5, 1, -7, -139],
  start: [0.5, 1, -7, -87],
}
/** Upright and narrower: Select and Start side by side at the bottom, under both clusters. */
const SELECT_START_ROW: Placement = {
  select: [0.5, 1, -46, -30],
  start: [0.5, 1, 46, -30],
}

/** The two-screen consoles upright, wide and narrow (`DUAL_WIDE_PX`). On the S23 (412 × 892)
 * the DS band is 198 px: the screens 412 × 309 each, the strip 44 between them. */
const DUAL: Record<'nds' | '3ds', { wide: Defaults; narrow: Defaults }> = {
  nds: {
    wide: {
      band: 190 + GRID_PX,
      place: {
        ...STRIP_ROW,
        dpad: [0, 1, 80, -116],
        ...diamond(1, 1, -88, -110),
        ...SELECT_START_COLUMN,
      },
    },
    narrow: {
      band: 216 + GRID_PX,
      place: {
        ...STRIP_ROW,
        dpad: [0, 1, 84, -130],
        ...diamond(1, 1, -92, -136),
        ...SELECT_START_ROW,
      },
    },
  },
  // The Circle Pad over the d-pad, as on the console. On the S23 the band is 318 px: the top
  // screen 412 wide, the touch screen at the same scale.
  '3ds': {
    wide: {
      band: 310 + GRID_PX,
      place: {
        ...STRIP_ROW,
        circlePad: [0, 1, 80, -246],
        dpad: [0, 1, 80, -102],
        ...diamond(1, 1, -88, -180),
        ...SELECT_START_COLUMN,
      },
    },
    narrow: {
      band: 336 + GRID_PX,
      place: {
        ...STRIP_ROW,
        circlePad: [0, 1, 84, -272],
        dpad: [0, 1, 84, -130],
        ...diamond(1, 1, -92, -160),
        ...SELECT_START_ROW,
      },
    },
  },
}

/** The two-screen consoles on their side: over the strip under the screens and their lower outer
 * corners, L and R in the lower corners. DS: the d-pad beside L, the face buttons beside R,
 * Select and Start at the bottom middle with the menu above them. 3DS: the Circle Pad takes the
 * d-pad's place, the d-pad goes over L, and Select, Start and the menu stand in one column. */
const DUAL_LANDSCAPE: Record<'nds' | '3ds', Placement> = {
  nds: {
    l: [0, 1, 60, -30],
    r: [1, 1, -60, -30],
    dpad: [0, 1, 186, -82],
    ...diamond(1, 1, -200, -88),
    select: [0.5, 1, -44, -30],
    start: [0.5, 1, 44, -30],
    menu: [0.5, 1, 0, -82],
  },
  '3ds': {
    l: [0, 1, 60, -30],
    r: [1, 1, -60, -30],
    dpad: [0, 1, 84, -150],
    circlePad: [0, 1, 226, -72],
    ...diamond(1, 1, -200, -88),
    start: [0.5, 1, 0, -30],
    select: [0.5, 1, 0, -82],
    menu: [0.5, 1, 0, -134],
  },
}

/** Whether `console` has two screens (its default keeps them apart on a phone). */
export const isDualScreen = (console: ConsoleId): boolean =>
  FAMILY[console] === 'nds' || FAMILY[console] === '3ds'

/** The default of `console` held `orientation` in `area`. */
function defaultsOf(console: ConsoleId, orientation: TouchOrientation, area: Area): Defaults {
  const family = FAMILY[console]
  if (family === 'nds' || family === '3ds') {
    if (orientation === 'landscape') return { place: DUAL_LANDSCAPE[family] }
    return area.width >= DUAL_WIDE_PX ? DUAL[family].wide : DUAL[family].narrow
  }
  return { place: DEFAULTS[family][orientation] }
}

/** The band a two-screen console's default keeps upright, CSS px (0 for the others and on the
 * side): what `reservedBottom` measures on it with the design system's boxes. */
export const defaultBand = (console: ConsoleId, orientation: TouchOrientation, area: Area) =>
  defaultsOf(console, orientation, area).band ?? 0

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
  const { place, band = 0 } = defaultsOf(console, orientation, area)
  const anchor = place[id]
  if (!anchor) return null
  const [ax, ay, dx, dy] = anchor
  const y = ay === STRIP ? (safeOf(area).top + area.height - band) / 2 : ay * area.height
  return placed(id, ax * area.width + dx, y + dy, area)
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

/** Upright, the controls whose centre is below this share of the height are the bottom band's;
 * those above it (one the user put at the top) are not counted. Under a half so the 3DS's
 * shoulders, 400 px from the bottom, still count on a 740 px tall phone. */
const BAND_FROM = 0.4
/** The same for a two-screen console: L, the menu and R in the strip between the screens stand
 * above the middle (0.41 of the height on the S23) and are not the band's. */
const DUAL_BAND_FROM = 0.5

/**
 * How much of the bottom of a portrait area the layout keeps for itself, CSS pixels: from the
 * top of the highest control of the band (`BAND_FROM`, and a grid step of air) to the bottom. The
 * game is drawn right above it (`reservedInset`). 0 in landscape or with no control there.
 */
export function reservedBottom(
  layout: TouchLayout,
  area: Area,
  boxOf: (id: TouchControlId) => Box,
  from = BAND_FROM,
): number {
  if (orientationOf(area) !== 'portrait') return 0
  const tops = layout.controls.filter(c => c.y >= from).map(c => rectOf(c, area, boxOf(c.id)).top)
  if (tops.length === 0) return 0
  return Math.max(0, Math.round(area.height - Math.min(...tops) + GRID_PX))
}

/**
 * How much of each side of a landscape area the layout keeps for itself, CSS pixels: on the left
 * up to the right edge of the controls wholly in the left half, on the right from the left edge
 * of those wholly in the right half (and a grid step of air). A control across the middle is not
 * counted (a layout of the user's that puts one over the game keeps it there). 0 in portrait.
 */
export function reservedSides(
  layout: TouchLayout,
  area: Area,
  boxOf: (id: TouchControlId) => Box,
): { left: number; right: number } {
  if (orientationOf(area) !== 'landscape') return { left: 0, right: 0 }
  const middle = area.width / 2
  const rects = layout.controls.map(c => rectOf(c, area, boxOf(c.id)))
  const lefts = rects.filter(r => r.right <= middle).map(r => r.right)
  const rights = rects.filter(r => r.left >= middle).map(r => r.left)
  return {
    left: lefts.length ? Math.round(Math.max(...lefts) + GRID_PX) : 0,
    right: rights.length ? Math.round(area.width - Math.min(...rights) + GRID_PX) : 0,
  }
}

/**
 * The edges the layout keeps clear of the game (the video inset, `session.setInset`): upright the
 * band at the bottom, the game against it so no empty band separates the two; on the side the two
 * columns, the game centred between them.
 *
 * A two-screen console (`dualScreen`) keeps its screens apart and large instead: upright the
 * window's safe top, the band (`DUAL_BAND_FROM`) and the strip between the screens, the screens
 * against both ends (`apart`); on its side only the camera's edge and `SIDE_EDGE_PX` on each
 * side, the screens at the top with `SIDE_GAP_PX` between them and the controls over the rest.
 */
export function reservedInset(
  layout: TouchLayout,
  area: Area,
  boxOf: (id: TouchControlId) => Box,
  dualScreen = false,
): InsetPx {
  const safe = safeOf(area)
  if (dualScreen && orientationOf(area) === 'portrait') {
    return {
      left: safe.left,
      top: safe.top,
      right: safe.right,
      bottom: reservedBottom(layout, area, boxOf, DUAL_BAND_FROM),
      align: 'apart',
      between: STRIP_PX,
    }
  }
  if (dualScreen) {
    return {
      left: Math.max(safe.left, SIDE_EDGE_PX),
      top: safe.top,
      right: Math.max(safe.right, SIDE_EDGE_PX),
      bottom: 0,
      align: 'top',
      between: SIDE_GAP_PX,
    }
  }
  if (orientationOf(area) === 'portrait') {
    const bottom = reservedBottom(layout, area, boxOf)
    return bottom > 0 ? { ...NO_INSET, bottom, align: 'bottom' } : NO_INSET
  }
  const { left, right } = reservedSides(layout, area, boxOf)
  return left > 0 || right > 0 ? { ...NO_INSET, left, right } : NO_INSET
}
