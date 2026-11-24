// The C06 editor's game screens (Trello #134): beside the controls, the console's screens are
// elements the editor picks, drags and resizes. A screen's rectangle is fractions of the whole
// window (`CustomScreen`); the layout keeps every screen of the console or none (none: the game
// is placed by the screens settings, the editor starts from Rust's `phoneStart`). Pure; the
// rectangle rules (inside the window, the minimum, the proportions) are the C09 editor's.
import type { CustomScreen, ScreenId, TouchControlId, TouchLayout, TouchScreenShape } from '@/emu'
import { contain, resizeTo } from '@/components/screen-editor/screenEditor'
import { GRID_PX, type Area } from './touchLayout'

/** What the editor picks: a screen or a control. */
export type ElementId = ScreenId | TouchControlId

const SCREEN_IDS: readonly ElementId[] = ['main', 'top', 'bottom'] satisfies ScreenId[]
export const isScreen = (id: ElementId): id is ScreenId => SCREEN_IDS.includes(id)

/** The screens drawn: the layout's own placement, else where the editor starts them. */
export function screensShown(layout: TouchLayout, start: readonly CustomScreen[]): CustomScreen[] {
  return layout.screens.length > 0 ? layout.screens : [...start]
}

/** Every element L1 / R1 walk through: the screens first, then the controls. */
export function elementIds(layout: TouchLayout, start: readonly CustomScreen[]): ElementId[] {
  return [...screensShown(layout, start).map(s => s.id), ...layout.controls.map(c => c.id)]
}

/** The element before (`-1`) or after (`1`) `current`, wrapping; null when there is none. */
export function nextElement(
  ids: readonly ElementId[],
  current: ElementId | null,
  dir: -1 | 1,
): ElementId | null {
  if (ids.length === 0) return null
  const i = current === null ? -1 : ids.indexOf(current)
  return ids[(i + dir + ids.length) % ids.length] ?? null
}

/** `layout` with `s` in place of its screen: every screen goes into the layout with it. */
export function withScreen(
  layout: TouchLayout,
  start: readonly CustomScreen[],
  s: CustomScreen,
): TouchLayout {
  return { ...layout, screens: screensShown(layout, start).map(x => (x.id === s.id ? s : x)) }
}

const otherScreen = (id: ScreenId): ScreenId =>
  id === 'top' ? 'bottom' : id === 'bottom' ? 'top' : id

/** Width over height of the picture the frame `id` shows: the other screen's when swapped. */
export function ratioOf(
  shapes: readonly TouchScreenShape[],
  id: ScreenId,
  swapped: boolean,
): number {
  const shown = swapped ? otherScreen(id) : id
  const shape = shapes.find(s => s.id === shown) ?? shapes.find(s => s.id === id)
  return shape && shape.height > 0 ? shape.width / shape.height : 1
}

const toGrid = (px: number) => Math.round(px / GRID_PX) * GRID_PX

/** `s` with its top-left corner at (`leftPx`, `topPx`), snapped to the 8 px grid when `snap`,
 * kept inside the window. */
export function moveScreen(
  s: CustomScreen,
  leftPx: number,
  topPx: number,
  area: Area,
  snap: boolean,
): CustomScreen {
  const at = (px: number) => (snap ? toGrid(px) : px)
  return contain({ ...s, x: at(leftPx) / area.width, y: at(topPx) / area.height })
}

/** One d-pad step: one grid step (8 px) that way. */
export function nudgeScreen(
  s: CustomScreen,
  dx: -1 | 0 | 1,
  dy: -1 | 0 | 1,
  area: Area,
): CustomScreen {
  return moveScreen(
    s,
    s.x * area.width + dx * GRID_PX,
    s.y * area.height + dy * GRID_PX,
    area,
    false,
  )
}

/** `s` at `widthPx` × `heightPx` (its top-left stays put); keeping the proportions the width
 * leads and the height follows `ratio`. Never under the minimum, never out of the window. */
export function resizeScreen(
  s: CustomScreen,
  widthPx: number,
  heightPx: number,
  keep: boolean,
  ratio: number,
  area: Area,
): CustomScreen {
  return resizeTo(s, widthPx / area.width, heightPx / area.height, { keep, ratio, area }, false)
}

/** The size slider's value: the screen's width, per cent of the window's. */
export const screenSizePct = (s: CustomScreen): number => Math.round(s.w * 100)

/** How close (CSS px) a screen's centre is to the window's to show its centre guide. */
const CENTRE_PX = 1

/** The window-centre guides a screen meets: `across` its centre on the window's vertical middle
 * line (centred left to right), `down` on the horizontal one (centred top to bottom). */
export function centreGuides(s: CustomScreen, area: Area): { across: boolean; down: boolean } {
  const near = (start: number, size: number, whole: number) =>
    Math.abs((start + size / 2) * whole - whole / 2) < CENTRE_PX
  return { across: near(s.x, s.w, area.width), down: near(s.y, s.h, area.height) }
}
