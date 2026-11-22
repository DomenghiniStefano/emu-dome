// The touch controls' small pieces shared by the overlay and the C06 editor (Trello #29): which
// game button a control presses, and the `touch.*` values read from a settings page.
import type { Button, SettingRow, TouchControlId } from '@/emu'
import { layoutChoice } from './touchLayout'
import { touchSkin } from './skins'
import { DEFAULT_PREFS, hideAfter, type TouchPrefs } from './touchVisibility'
import { lostChoice, type LostChoice } from './controllerLoss'

const BUTTON: Partial<Record<TouchControlId, Button>> = {
  a: 'a',
  b: 'b',
  x: 'x',
  y: 'y',
  l: 'l',
  r: 'r',
  start: 'start',
  select: 'select',
}

/** The game button a control presses; null for the d-pad, the stick and the menu. */
export const gameButton = (id: TouchControlId): Button | null => BUTTON[id] ?? null

const num = (v: unknown, fallback: number) => (typeof v === 'number' ? v : fallback)

/** The `touch.*` values of a page's rows (the touch page at the session's scope). */
export function prefsFrom(rows: readonly SettingRow[]): TouchPrefs & { lost: LostChoice } {
  const value = (key: SettingRow['key']) => rows.find(r => r.key === key)?.value
  return {
    portrait: layoutChoice(value('touch.tport')),
    landscape: layoutChoice(value('touch.tland')),
    skin: touchSkin(value('touch.tskin')),
    opacity: num(value('touch.topac'), DEFAULT_PREFS.opacity),
    haptics: num(value('touch.thapt'), DEFAULT_PREFS.haptics),
    hide: hideAfter(value('touch.thide')),
    hideWithPad: value('touch.tpad') !== false,
    lost: lostChoice(value('touch.tlost')),
  }
}
