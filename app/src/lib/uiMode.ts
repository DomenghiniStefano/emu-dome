// TV mode (Trello #30): which layout the interface takes, decided in one place. Pure: the
// composable (composables/useUiMode.ts) feeds it the setting, the window and the controllers.
//
// - `general.ui` (a per-device row of the PC only): Automatica / Modalità desktop / Modalità TV;
// - Automatica: TV while the window is full screen and at least one controller is connected;
// - a phone is never in TV mode (the row is not offered there): its layout is the touch one.
import type { PadInfo, SettingDevice } from '@/emu'

export const UI_PREFERENCES = ['auto', 'desktop', 'tv'] as const
export type UiPreference = (typeof UI_PREFERENCES)[number]

export type UiLayout = 'desktop' | 'tv'

/** A `general.ui` value as a preference: anything unknown (or not read yet) is Automatica. */
export function uiPreference(value: unknown): UiPreference {
  return UI_PREFERENCES.find(p => p === value) ?? 'auto'
}

export interface UiInputs {
  preference: UiPreference
  /** What the settings schema says this device is; null until it is read. */
  device: SettingDevice | null
  fullscreen: boolean
  /** How many controllers are connected. */
  controllers: number
}

/** The layout the interface takes now. */
export function decideUiLayout({
  preference,
  device,
  fullscreen,
  controllers,
}: UiInputs): UiLayout {
  if (device === 'phone') return 'desktop'
  switch (preference) {
    case 'desktop':
      return 'desktop'
    case 'tv':
      return 'tv'
    case 'auto':
      return fullscreen && controllers > 0 ? 'tv' : 'desktop'
  }
}

/** The connected controllers after `pad` connected, changed (battery) or disconnected. */
export function padsAfter(
  pads: readonly PadInfo[],
  pad: PadInfo,
  change: 'connected' | 'changed' | 'disconnected',
): PadInfo[] {
  const others = pads.filter(p => p.id !== pad.id)
  switch (change) {
    case 'connected':
      return [...others, pad]
    case 'changed':
      return pads.some(p => p.id === pad.id)
        ? pads.map(p => (p.id === pad.id ? pad : p))
        : [...pads]
    case 'disconnected':
      return others
  }
}

/** The controller the TV layout names and draws the glyphs of: the one connected last. */
export function activePad(pads: readonly PadInfo[]): PadInfo | null {
  return pads.at(-1) ?? null
}

/** The screen the TV sizes are drawn for (the owner's mockup). */
export const TV_DESIGN = { width: 1920, height: 1080 } as const

/**
 * How much the TV sizes grow on a window of `width` × `height` CSS pixels: its ratio to the
 * mockup's screen on the tighter side, never under 1 (the 18px minimum stays).
 */
export function tvScale(width: number, height: number): number {
  return Math.max(1, Math.min(width / TV_DESIGN.width, height / TV_DESIGN.height))
}
