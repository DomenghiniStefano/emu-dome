// TV mode (Trello #30): which layout the interface takes, decided in one place. Pure: the
// composable (composables/useUiMode.ts) feeds it the setting, the answers and the controllers.
//
// - `general.ui` (a per-device row of the PC only): Automatica / Modalità desktop / Modalità TV;
// - Automatica (owner, 2026-11-22): never switches by itself. When a controller connects (or is
//   connected at startup) outside TV, it asks (TvModeDialog: «Passare alla modalità TV?»); when
//   the last controller goes away in TV, it asks whether to go back to the desktop. The answer
//   holds for this run; with «Non chiederlo più» it is saved as Modalità TV / desktop. The window
//   being full screen plays no part any more. Desktop and TV never ask;
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
  /** Automatica: the layout the user answered for this run (TvModeDialog), none yet = desktop. */
  answered: UiLayout | null
}

/** The layout the interface takes now. */
export function decideUiLayout({ preference, device, answered }: UiInputs): UiLayout {
  if (device === 'phone') return 'desktop'
  switch (preference) {
    case 'desktop':
      return 'desktop'
    case 'tv':
      return 'tv'
    case 'auto':
      return answered ?? 'desktop'
  }
}

/** What the TV question asks: switch to TV (a controller came), or go back (the last one left). */
export type TvAsk = 'connect' | 'disconnect'

export interface AskInputs {
  preference: UiPreference
  device: SettingDevice | null
  /** The layout now. */
  tv: boolean
  /** Controllers connected after the change. */
  controllers: number
}

/**
 * Whether a controller connecting (or connected at startup) asks to switch to TV: Automatica, on a
 * known device that is not a phone, not in TV already.
 */
export function asksOnConnect({ preference, device, tv, controllers }: AskInputs): boolean {
  return preference === 'auto' && device !== null && device !== 'phone' && !tv && controllers > 0
}

/** Whether a controller going away asks to go back to the desktop: Automatica, in TV, none left. */
export function asksOnDisconnect({ preference, device, tv, controllers }: AskInputs): boolean {
  return preference === 'auto' && device !== 'phone' && tv && controllers === 0
}

/**
 * The question shown after the controllers changed: a reconnection closes the «go back» one
 * (TV stays), the last controller leaving closes the «switch» one; otherwise it stays.
 */
export function askAfter(open: TvAsk | null, inputs: AskInputs): TvAsk | null {
  if (open === 'disconnect' && inputs.controllers > 0) return null
  if (open === 'connect' && inputs.controllers === 0) return null
  return open
}

/** The `general.ui` value «Non chiederlo più» saves with an answer. */
export const rememberedPreference = (layout: UiLayout): UiPreference => layout

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
