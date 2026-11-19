// L07 Controlli: what the tab says about a game's controls, from the `pad` page's rows at the
// game's level. Pure: the translations come in as a `Translator`.
import type { ConsoleId, SettingRow } from '../../emu'
import { msg } from '../../settingsModel'
import type { Translator } from './translator'

/** The `pad` page's rows by key. */
export type PadRows = Record<string, SettingRow>

/** Nintendo DS and 3DS: the bottom screen is touched. */
export function hasTouchScreen(console: ConsoleId): boolean {
  return console === 'nds' || console === '3ds'
}

/** Whether the buttons are remapped for this game itself (`pad.remap` stored at its level). */
export function hasOwnRemap(rows: PadRows): boolean {
  return rows['pad.remap']?.source === 'game'
}

/** How the bottom screen is touched (`pad.pointer`) as its option is named; the mouse by default. */
export function pointerLabel(rows: PadRows, { t, te }: Translator): string {
  const v = rows['pad.pointer']?.value
  const id = typeof v === 'string' ? v : 'mouse'
  const key = msg.option('pad.pointer', id)
  return te(key) ? t(key) : id
}
