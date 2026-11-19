// The keyboard's fixed shortcuts in game: the one table of them. App.vue carries them out,
// C01 lists them under «Tasti della tastiera», C03 on the keyboard's tab, and the G12 overlay
// will read them from here too. The same table is written in docs/combo-tasti.md, section 5.
//
// The controller's shortcuts are `DEFAULT_COMBOS` in Rust (crates/emudome-core/src/hotkeys.rs);
// a key does what the hotkey with the same id does, plus fullscreen, which only the keyboard
// (and the quick menu) has.
import type { HotkeyId, HotkeyStatus } from '@/emu'

/** What a key does: a controller hotkey's action, or fullscreen (keyboard only). */
export type KeyAction = HotkeyId | 'fullscreen'

export interface KeyShortcut {
  /** `KeyboardEvent.code`. */
  code: string
  action: KeyAction
  /** Reserved keys (`later`) are listed and do nothing until their feature exists. */
  status: HotkeyStatus
}

/**
 * Esc menu · F1 save · F4 load · F2 / F3 previous / next slot · Tab fast forward (held) ·
 * Backspace rewind (held; toggled with `a11y.toggle`) · F9 FPS · F11 full screen · F12 screenshot.
 * F1 and F4 act on the current slot, the one F2 / F3 and L3 + ← → move.
 */
export const KEYBOARD_SHORTCUTS: readonly KeyShortcut[] = [
  { code: 'Escape', action: 'menu', status: 'active' },
  { code: 'F1', action: 'save', status: 'active' },
  { code: 'F4', action: 'load', status: 'active' },
  { code: 'F2', action: 'prev_slot', status: 'active' },
  { code: 'F3', action: 'next_slot', status: 'active' },
  { code: 'Tab', action: 'ff_hold', status: 'active' },
  // Select's default key moved from Backspace to the right Shift (bindings.rs, Trello #42).
  { code: 'Backspace', action: 'rewind', status: 'active' },
  { code: 'F9', action: 'fps', status: 'active' },
  { code: 'F11', action: 'fullscreen', status: 'active' },
  { code: 'F12', action: 'screenshot', status: 'active' },
]

/** The working shortcut of a key, if it has one. */
export function keyShortcut(code: string): KeyShortcut | undefined {
  return KEYBOARD_SHORTCUTS.find(k => k.code === code && k.status === 'active')
}

/** One line of a shortcut list: the previous / next slot pair shares one line («Cambia slot»). */
export interface ShortcutLine<T> {
  /** The line's action: `slot` for the pair. */
  action: KeyAction | 'slot'
  items: T[]
}

/** Groups a list in the table's order into lines, the slot pair as one line where it first appears. */
export function shortcutLines<T>(items: T[], action: (item: T) => KeyAction): ShortcutLine<T>[] {
  const isSlot = (a: KeyAction) => a === 'prev_slot' || a === 'next_slot'
  return items.reduce<ShortcutLine<T>[]>((lines, item) => {
    const a = action(item)
    if (!isSlot(a)) return [...lines, { action: a, items: [item] }]
    const pair = lines.find(l => l.action === 'slot')
    if (!pair) return [...lines, { action: 'slot', items: [item] }]
    pair.items.push(item)
    return lines
  }, [])
}
