// The devices of the Controls section (C01–C04, E05): which one is in use, its name and the
// line under it, and what a capture listens to on it. Pure functions over the bridge's data.
import type { PadFamily, PadInfo, Pos } from '@/emu'
import type { CaptureDevice } from '@/lib/controls'
import { t } from '@/i18n'
import { gamepadName } from '@/i18n/labels'

/** A device as the section chooses it: a controller's id, or the keyboard. */
export type DeviceId = number | 'keyboard'
/** A device as the section shows it. */
export type Device = PadInfo | 'keyboard'

/** The device in use: the keyboard, the chosen controller, else the first one; null = none (E05). */
export function deviceInUse(pads: PadInfo[], chosen: DeviceId | null): Device | null {
  if (chosen === 'keyboard') return 'keyboard'
  return pads.find(p => p.id === chosen) ?? pads[0] ?? null
}

/** The remap table a device uses: its controller type, or the keyboard's. */
export const familyOf = (device: Device | null): PadFamily =>
  device === 'keyboard' || !device ? 'keyboard' : device.family

/** Whether `candidate` is the device in use. */
export function isSameDevice(inUse: Device, candidate: Device): boolean {
  if (candidate === 'keyboard' || inUse === 'keyboard') return candidate === inUse
  return inUse.id === candidate.id
}

/** The controllers after `pad` connected: added at the end, once. */
export function padConnected(pads: PadInfo[], pad: PadInfo): PadInfo[] {
  return pads.some(p => p.id === pad.id) ? pads : [...pads, pad]
}

/** The controllers after the one with `id` went. */
export function padGone(pads: PadInfo[], id: number): PadInfo[] {
  return pads.filter(p => p.id !== id)
}

/** The controllers after `pad` changed (its battery): replaced in place, never added. */
export function padChanged(pads: PadInfo[], pad: PadInfo): PadInfo[] {
  return pads.map(p => (p.id === pad.id ? pad : p))
}

/** «Collegato · batteria 80 %»: the line under a controller's name. */
export function connectionText(pad: PadInfo): string {
  const parts = [t('controls.connected')]
  if (pad.battery !== null) parts.push(t('controls.overview.battery', { percent: pad.battery }))
  return parts.join(' · ')
}

export const deviceName = (device: Device | null): string =>
  device === 'keyboard' || !device ? t('controls.keyboard') : gamepadName(device)

export const deviceDetail = (device: Device | null): string =>
  device === 'keyboard' || !device ? t('controls.keyboardDetail') : connectionText(device)

/** The menu's «back» position from `pad.confirm`: B is east unless A is the bottom button. */
export const menuBackPos = (confirm: unknown): Pos => (confirm === 'bottom' ? 'east' : 'south')

/** What a capture listens to on `device`; `back` (held 1 s) cancels it on a controller. */
export function captureDeviceOf(device: Device | null, back: Pos): CaptureDevice {
  return device === 'keyboard' || !device
    ? { kind: 'keyboard' }
    : { kind: 'pad', id: device.id, back }
}
