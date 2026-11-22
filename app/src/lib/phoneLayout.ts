// How the shell lays out its sections on each device (Trello #29). Pure: composables/useDeviceLayout.ts
// feeds it what the settings schema and the window say.
//
// - The desktop: the rail on the left (TV mode's tabs are lib/uiMode.ts's decision).
// - A phone held upright: a tab bar at the bottom, in reach of the thumb.
// - A phone turned on its side: the rail as a column on the camera's side, leaving the notch free;
//   it moves to the other side when the phone is turned the other way.
import type { SettingDevice } from '@/emu'

/** Where the shell's sections sit. */
export const ShellNav = {
  /** The desktop's rail on the left. */
  Rail: 'rail',
  /** The phone upright: a tab bar at the bottom. */
  Tabs: 'tabs',
  /** The phone on its side: a column on the camera's side. */
  Column: 'column',
} as const
export type ShellNav = (typeof ShellNav)[keyof typeof ShellNav]

export function shellNav(device: SettingDevice | null, portrait: boolean): ShellNav {
  if (device !== 'phone') return ShellNav.Rail
  return portrait ? ShellNav.Tabs : ShellNav.Column
}

/** The window's edge the camera is on, with the phone on its side. */
export type CameraSide = 'left' | 'right'

/**
 * The camera's side from `screen.orientation.type`. A phone's natural orientation is upright, its
 * camera at the top: turned a quarter anticlockwise (`landscape-primary`, angle 90) the camera is
 * on the left, turned clockwise (`landscape-secondary`, angle 270) on the right. Anything else
 * (upright, or not reported) gives the left, the mockup's side.
 */
export function cameraSide(orientation: string | undefined): CameraSide {
  return orientation === 'landscape-secondary' ? 'right' : 'left'
}
