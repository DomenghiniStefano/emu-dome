// The keyboard's fixed shortcuts in game. The one table is Rust's (`KEYBOARD_SHORTCUTS`,
// crates/emudome-core/src/keyboard.rs), which also routes the keys in game; it reaches the
// interface generated (generated/keyboard-shortcuts.json). C01 lists them under «Tasti della
// tastiera», C03 on the keyboard's tab, and the G12 overlay will read them from here too. The
// same table is written in docs/combo-tasti.md, section 5.
//
// The controller's shortcuts are `DEFAULT_COMBOS` in Rust (crates/emudome-core/src/hotkeys.rs);
// a key does what the hotkey with the same id does, plus fullscreen, which only the keyboard
// (and the quick menu) has.
import type { HotkeyId, HotkeyStatus } from '@/emu'
import table from '@/generated/keyboard-shortcuts.json'

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
export const KEYBOARD_SHORTCUTS = table as readonly KeyShortcut[]

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
