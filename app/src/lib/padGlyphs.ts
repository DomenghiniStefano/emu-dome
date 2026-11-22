// The glyphs of the connected controller for the TV layout's hints (Trello #30: the button bar,
// the hero's actions). The routing works on positions (Rust `hotkeys.rs`: A = the right face
// button, B = the bottom one, X = the top one, Y = the left one, as on a Nintendo pad), so each
// action is a position, and each position is drawn as the controller of that family prints it:
// a letter or a name (Xbox «A», Nintendo «X», «LB»), PlayStation's four shapes as icons. A
// generic controller and none at all get the positions' Nintendo letters, the app's own names.
// Pure: the component turns a message key into text and an icon name into a Lucide icon.
import type { PadFamily, Pos } from '@/emu'

/** What the TV layout names a button for. */
export type PadAction = 'confirm' | 'back' | 'more' | 'search' | 'prev' | 'next'

/** `pad.confirm`: the right face button confirms (Nintendo's way) or the bottom one (Xbox's). */
export type ConfirmSide = 'right' | 'bottom'

/** A `pad.confirm` value as a side: anything but `bottom` (the default too) is the right one. */
export const confirmSide = (value: unknown): ConfirmSide =>
  value === 'bottom' ? 'bottom' : 'right'

/** The position that carries out `action`. */
export function actionPos(action: PadAction, confirm: ConfirmSide): Pos {
  switch (action) {
    case 'confirm':
      return confirm === 'right' ? 'east' : 'south'
    case 'back':
      return confirm === 'right' ? 'south' : 'east'
    case 'more':
      return 'north'
    case 'search':
      return 'west'
    case 'prev':
      return 'l1'
    case 'next':
      return 'r1'
  }
}

/** PlayStation's face shapes, as Lucide icon names. */
export type PadShape = 'cross' | 'circle' | 'square' | 'triangle'

/** A glyph: a message key for a letter or a name, or a shape drawn as an icon. */
export type PadGlyph = { key: string } | { shape: PadShape }

const SHAPES: Partial<Record<Pos, PadShape>> = {
  south: 'cross',
  east: 'circle',
  west: 'square',
  north: 'triangle',
}

/** The Nintendo letters of the positions (`controls.button.*`): generic controllers and none. */
const NINTENDO: Partial<Record<Pos, string>> = {
  east: 'a',
  south: 'b',
  north: 'x',
  west: 'y',
  l1: 'l',
  r1: 'r',
}

/** How the controller of `family` prints `pos`; null for no controller. */
export function padGlyph(family: PadFamily | null, pos: Pos): PadGlyph {
  switch (family) {
    case 'playstation': {
      const shape = SHAPES[pos]
      return shape ? { shape } : { key: `controls.pos.common.${pos}` }
    }
    case 'xbox':
    case 'switch':
      return { key: `controls.pos.${family}.${pos}` }
    case 'generic':
    case 'keyboard':
    case null: {
      const letter = NINTENDO[pos]
      return { key: letter ? `controls.button.${letter}` : `controls.pos.common.${pos}` }
    }
  }
}
