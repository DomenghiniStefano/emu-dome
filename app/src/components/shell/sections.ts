// The shell's sections (desktop mode, board L01): the rail shows them in this order,
// with Settings on its own at the bottom; the phone's tab bar in the same order (#29). The id is
// also the `shell.<id>` message key.
import type { Component } from 'vue'
import { Cloud, Gamepad2, Library, Save, Settings2 } from '@lucide/vue'

export const SECTIONS = ['library', 'saves', 'controls', 'cloud', 'settings'] as const
export type Section = (typeof SECTIONS)[number]

/** Each section's icon, on the rail and on the phone's tab bar. */
export const SECTION_ICONS: Record<Section, Component> = {
  library: Library,
  saves: Save,
  controls: Gamepad2,
  cloud: Cloud,
  settings: Settings2,
}
