// C01 «Scorciatoie da ricordare»: one line per shortcut, what to press and its short name. The
// controller's come from the bindings in force (`DEFAULT_COMBOS` with `pad.combos`), the
// keyboard's from its one table (keyboard.ts): never a list here.
import type { ComboRow, PadFamily, Pos } from '../../emu'
import { comboText, keyName, shortName, slotPairText } from '../../controls'
import { t } from '../../i18n'
import { KEYBOARD_SHORTCUTS, shortcutLines, type KeyAction } from '../../keyboard'

export interface RememberLine {
  key: KeyAction | 'slot'
  /** What to press («L3 + RB», «F1»). */
  chip: string
  name: string
}

function padChip(
  family: PadFamily,
  modifier: Pos,
  action: KeyAction | 'slot',
  items: ComboRow[],
): string {
  const pos = (id: string) => items.find(c => c.hotkey === id)?.pos ?? null
  if (action === 'slot') return slotPairText(family, modifier, pos('prev_slot'), pos('next_slot'))
  // The menu always opens with Home, whatever its combo.
  if (action === 'menu') return t('flow.hintHome')
  return comboText(family, modifier, items[0]?.pos ?? null)
}

/** A controller's lines: the shortcuts to remember that have a button (the menu always). */
export function padRememberLines(
  family: PadFamily,
  modifier: Pos,
  combos: ComboRow[],
): RememberLine[] {
  const shown = combos.filter(c => c.toRemember && (c.pos !== null || c.hotkey === 'menu'))
  return shortcutLines(shown, c => c.hotkey).map(l => ({
    key: l.action,
    chip: padChip(family, modifier, l.action, l.items),
    name: shortName(l.action),
  }))
}

/** The keyboard's lines: every key of its table, the slot pair on one line. */
export function keyboardRememberLines(): RememberLine[] {
  return shortcutLines([...KEYBOARD_SHORTCUTS], k => k.action).map(l => ({
    key: l.action,
    chip: l.items.map(k => keyName(k.code)).join(' / '),
    name: shortName(l.action),
  }))
}
