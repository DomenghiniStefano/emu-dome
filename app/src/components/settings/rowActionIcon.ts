// The icon on a settings row's own button (Trello #104, the owner 2026-11-22: chosen by the kind of
// the row, not by its key): a binding list opens Controls, a screen layout the C09 editor, the touch
// controls the C06 editor, a file row a picker, an action row does what its words say. Kinds with a
// control of their own (a switch, a list, a slider…) have no such button, so no icon.
import type { Component } from 'vue'
import { ArrowRight, File as FileIcon, Gamepad2, Hand, Rows2 } from '@lucide/vue'
import type { SettingKind } from '@/emu'

const ICONS: Partial<Record<SettingKind['type'], Component>> = {
  bindings: Gamepad2,
  screenLayout: Rows2,
  touchLayout: Hand,
  file: FileIcon,
  action: ArrowRight,
}

/** The icon of the button a row of this kind shows, if it shows one. */
export function rowActionIcon(kind: SettingKind): Component | undefined {
  return ICONS[kind.type]
}
