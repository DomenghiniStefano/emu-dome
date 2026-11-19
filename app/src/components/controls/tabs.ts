// The tabs one step below C01: remap (C02), shortcuts (C03), sticks and triggers (C04).
import type { PadFamily } from '@/emu'

export const ControlsTab = {
  Remap: 'remap',
  Shortcuts: 'shortcuts',
  Sticks: 'sticks',
} as const
export type ControlsTab = (typeof ControlsTab)[keyof typeof ControlsTab]

/** The keyboard has no sticks. */
export const tabsFor = (family: PadFamily): ControlsTab[] =>
  family === 'keyboard'
    ? [ControlsTab.Remap, ControlsTab.Shortcuts]
    : [ControlsTab.Remap, ControlsTab.Shortcuts, ControlsTab.Sticks]

/** A tab that `list` has, else the first (remap). */
export const tabOrRemap = (list: ControlsTab[], wanted: ControlsTab): ControlsTab =>
  list.includes(wanted) ? wanted : ControlsTab.Remap
