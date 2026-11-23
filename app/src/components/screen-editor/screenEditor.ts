// The C09 screens editor's geometry (Trello #28): a custom layout (`screens.custom`) is each
// screen's rectangle as fractions of the game area, back to front. Moving, resizing, snapping,
// the numeric fields and the order are here, pure; where a layout puts the screens to begin with
// («Parti da») is Rust's (`emu.screens.presetCustom`), never worked out again here.
import type { ConsoleId, CustomLayout, CustomScreen, ScreenId } from '@/emu'

/** The two screens an editor moves (`main` is a one-screen console's). */
export type EditorScreen = Exclude<ScreenId, 'main'>
export const EDITOR_SCREENS: EditorScreen[] = ['top', 'bottom']

/**
 * The snapping grid: the area cut in this many steps each way. The d-pad moves by one step, a
 * drag snaps to it when «Aggancia alla griglia» is on. The grid drawn behind the screens
 * (`bg-editor-grid`, design-system/styles/utilities.css) uses the same count.
 */
export const GRID = 32
/** The smallest a screen gets, as a share of the area's width. */
export const MIN_SIZE = 0.08

/** A screen's own pixels at 1×, by console (the aspect «Mantieni le proporzioni» keeps). */
const SCREEN_PIXELS: Record<'nds' | '3ds', Record<EditorScreen, [number, number]>> = {
  nds: { top: [256, 192], bottom: [256, 192] },
  '3ds': { top: [400, 240], bottom: [320, 240] },
}

/** The area the fractions are of, in CSS pixels (the window: the editor hides the menu's bar). */
export interface Area {
  width: number
  height: number
}

/** Width over height of the picture a rectangle shows: the other screen's when swapped. */
export function aspect(console: ConsoleId, id: EditorScreen, swapped: boolean): number {
  const table = console === '3ds' ? SCREEN_PIXELS['3ds'] : SCREEN_PIXELS.nds
  const [w, h] = table[swapped ? other(id) : id]
  return w / h
}

export const other = (id: EditorScreen): EditorScreen => (id === 'top' ? 'bottom' : 'top')

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi)
/** Fractions are stored with 4 decimals (`CustomLayout::to_items`). */
export const round4 = (v: number) => Math.round(v * 10_000) / 10_000
const snapTo = (v: number) => Math.round(v * GRID) / GRID

/** The height (fraction of the area's height) that keeps `ratio` for a width `w`. */
export function heightFor(w: number, ratio: number, area: Area): number {
  return (w * area.width) / (ratio * area.height)
}

/** The width that keeps `ratio` for a height `h`. */
export function widthFor(h: number, ratio: number, area: Area): number {
  return (h * ratio * area.height) / area.width
}

/** `s` kept inside the area: its size first (at most the area, at least the minimum), then where. */
export function contain(s: CustomScreen): CustomScreen {
  const w = clamp(s.w, MIN_SIZE, 1)
  const h = clamp(s.h, MIN_SIZE / 2, 1)
  return {
    ...s,
    w: round4(w),
    h: round4(h),
    x: round4(clamp(s.x, 0, 1 - w)),
    y: round4(clamp(s.y, 0, 1 - h)),
  }
}

/** `s` moved to (x, y), snapped to the grid when `snap`, kept inside. */
export function moveTo(s: CustomScreen, x: number, y: number, snap: boolean): CustomScreen {
  return contain({ ...s, x: snap ? snapTo(x) : x, y: snap ? snapTo(y) : y })
}

/** One d-pad step: one grid step that way. */
export function step(s: CustomScreen, dx: -1 | 0 | 1, dy: -1 | 0 | 1): CustomScreen {
  return moveTo(s, s.x + dx / GRID, s.y + dy / GRID, false)
}

export interface Proportion {
  /** «Mantieni le proporzioni»: the height follows the width. */
  keep: boolean
  /** Width over height of the screen's picture (`aspect`). */
  ratio: number
  area: Area
}

/**
 * `s` with its corner at `w` × `h` (its top-left stays put), snapped when `snap`. Keeping the
 * proportions the width leads and the height follows, both shrunk together to stay inside.
 */
export function resizeTo(
  s: CustomScreen,
  w: number,
  h: number,
  p: Proportion,
  snap: boolean,
): CustomScreen {
  let nw = clamp(snap ? snapTo(s.x + w) - s.x : w, MIN_SIZE, 1 - s.x)
  if (!p.keep) {
    const nh = clamp(snap ? snapTo(s.y + h) - s.y : h, MIN_SIZE / 2, 1 - s.y)
    return contain({ ...s, w: nw, h: nh })
  }
  let nh = heightFor(nw, p.ratio, p.area)
  if (s.y + nh > 1) {
    nh = 1 - s.y
    nw = widthFor(nh, p.ratio, p.area)
  }
  return contain({ ...s, w: nw, h: nh })
}

/** One right-stick step: one grid step wider (right) or narrower (left), taller (down) or
 * shorter (up); keeping the proportions, up / down change the width as well. */
export function grow(s: CustomScreen, dx: -1 | 0 | 1, dy: -1 | 0 | 1, p: Proportion): CustomScreen {
  if (p.keep) {
    const by = (dx !== 0 ? dx : dy) / GRID
    return resizeTo(s, s.w + by, s.h, p, false)
  }
  return resizeTo(s, s.w + dx / GRID, s.h + dy / GRID, p, false)
}

/** The screen of `layout` named `id`. */
export const screenOf = (layout: CustomLayout, id: EditorScreen) =>
  layout.screens.find(s => s.id === id)

/** `layout` with `s` in place of the screen of the same id. */
export function replace(layout: CustomLayout, s: CustomScreen): CustomLayout {
  return { screens: layout.screens.map(x => (x.id === s.id ? s : x)) }
}

/** `layout` with `id` last: in front («In primo piano»). */
export function toFront(layout: CustomLayout, id: EditorScreen): CustomLayout {
  const s = screenOf(layout, id)
  if (!s) return layout
  return { screens: [...layout.screens.filter(x => x.id !== id), s] }
}

/** The screen in front: the last one. */
export const front = (layout: CustomLayout): EditorScreen | null =>
  (layout.screens.at(-1)?.id as EditorScreen | undefined) ?? null

/** L1 / R1, PageUp / PageDown: the screen before or after `id` in the layout, wrapping. */
export function pick(layout: CustomLayout, id: EditorScreen, dir: -1 | 1): EditorScreen {
  const ids = EDITOR_SCREENS.filter(x => screenOf(layout, x))
  const i = ids.indexOf(id)
  if (ids.length === 0) return id
  return ids[(i + dir + ids.length) % ids.length] ?? id
}

/** The panel stands on the side away from the selected screen: left when the screen's centre is
 * right of the middle (a little past it). */
export function panelSide(s: CustomScreen | undefined): 'left' | 'right' {
  return s && s.x + s.w / 2 > 0.55 ? 'left' : 'right'
}

/** A fraction as the panel's field shows it: per cent, one decimal. */
export const percent = (v: number) => Math.round(v * 1000) / 10

/** A field's text («12,5», «40») as a fraction; null when it is no number. */
export function fromPercent(text: string): number | null {
  const v = Number(text.trim().replace(',', '.'))
  return text.trim() !== '' && Number.isFinite(v) ? v / 100 : null
}

/** The four numeric fields of the panel. */
export type Field = 'x' | 'y' | 'w' | 'h'

/** `s` with one field typed in (a fraction), the other side following when proportions are kept. */
export function setField(
  s: CustomScreen,
  field: Field,
  value: number,
  p: Proportion,
): CustomScreen {
  switch (field) {
    case 'x':
    case 'y':
      return moveTo(s, field === 'x' ? value : s.x, field === 'y' ? value : s.y, false)
    case 'w':
      return resizeTo(s, value, s.h, p, false)
    case 'h':
      return p.keep
        ? resizeTo(s, widthFor(value, p.ratio, p.area), value, p, false)
        : resizeTo(s, s.w, value, p, false)
  }
}
