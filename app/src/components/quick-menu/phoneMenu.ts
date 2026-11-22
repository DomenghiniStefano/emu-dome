// The phone's quick menu (owner, 2026-11-22, proposal «A» of the mockup): what its first level's
// strip shows and what its second level «Altro» lists. Pure: QuickMenu.vue carries them out.
import type { Component } from 'vue'
import {
  ArrowDownUp,
  Camera,
  Ellipsis,
  FastForward,
  Gamepad2,
  Gauge,
  Laptop,
  Power,
  RefreshCw,
  Rewind,
  Rows2,
  SaveAll,
  Volume2,
  VolumeX,
} from '@lucide/vue'
import type { ScreenStatus } from '@/emu'
import { QuickMenuView } from './view'

export const StripId = {
  /** Rewinds while held (`speed.rew` on for this game only). */
  Rewind: 'rewind',
  /** Fast forward on / off at the chosen speed. */
  FastForward: 'ff',
  /** Sound off / on for this game. */
  Mute: 'mute',
  /** The second level. */
  More: 'more',
} as const
export type StripId = (typeof StripId)[keyof typeof StripId]

export interface StripButton {
  id: StripId
  icon: Component
  /** Message key of its name: a verb (owner: never an adjective). */
  label: string
  /** On now (fast forward running, sound off): lit, `aria-pressed`. Undefined: not a toggle. */
  pressed?: boolean
}

export interface StripContext {
  /** The resolved `speed.rew` of the open game. */
  rewind: boolean
  /** The game runs at the fast forward speed. */
  fastForward: boolean
  muted: boolean
}

/** The strip under the three tiles: «Riavvolgi» first when rewind is on for the game. */
export function stripButtons(c: StripContext): StripButton[] {
  const list: StripButton[] = []
  if (c.rewind) list.push({ id: StripId.Rewind, icon: Rewind, label: 'qm.verb.rewind' })
  list.push(
    {
      id: StripId.FastForward,
      icon: FastForward,
      label: 'qm.verb.ff',
      pressed: c.fastForward,
    },
    c.muted
      ? { id: StripId.Mute, icon: Volume2, label: 'qm.verb.unmute', pressed: true }
      : { id: StripId.Mute, icon: VolumeX, label: 'qm.verb.mute', pressed: false },
    { id: StripId.More, icon: Ellipsis, label: 'qm.moreLabel' },
  )
  return list
}

export const MoreId = {
  Saves: 'saves',
  Speed: 'speed',
  Screens: 'screens',
  Controls: 'controls',
  Screenshot: 'shot',
  SwapScreens: 'swap',
  CloseLid: 'lid',
  Restart: 'restart',
  Close: 'close',
} as const
export type MoreId = (typeof MoreId)[keyof typeof MoreId]

export interface MoreRow {
  id: MoreId
  icon: Component
  /** Message key of its name. */
  label: string
  /** The page it opens (a chevron at its end); none: an action. */
  page?: QuickMenuView
  /** A row that stays on (swap, the lid): whether it is on now. */
  pressed?: boolean
  /** «Chiudi il gioco»: last, apart, in danger ink. */
  danger?: boolean
}

export interface MoreContext {
  /** NDS and 3DS. */
  dualScreen: boolean
  /** The open game's screens (null until known: the lid then waits, listed). */
  screens: ScreenStatus | null
}

/** The rows of «Altro», top to bottom; «Chiudi il gioco» is always the last. */
export function moreRows(c: MoreContext): MoreRow[] {
  const list: MoreRow[] = [
    { id: MoreId.Saves, icon: SaveAll, label: 'qm.savesPage', page: QuickMenuView.Slots },
    { id: MoreId.Speed, icon: Gauge, label: 'qm.speedPage', page: QuickMenuView.Speed },
    {
      id: MoreId.Screens,
      icon: Rows2,
      label: 'qm.row.screens.name',
      page: QuickMenuView.Screens,
    },
    {
      id: MoreId.Controls,
      icon: Gamepad2,
      label: 'qm.row.controls.name',
      page: QuickMenuView.Controls,
    },
    { id: MoreId.Screenshot, icon: Camera, label: 'qm.screenshot' },
  ]
  if (c.dualScreen) {
    list.push({
      id: MoreId.SwapScreens,
      icon: ArrowDownUp,
      label: 'qm.swapScreens',
      pressed: c.screens?.swapped ?? false,
    })
    // Only where the core has a lid (melonDS DS; Azahar has none).
    if (c.screens?.lidSupported ?? true)
      list.push({
        id: MoreId.CloseLid,
        icon: Laptop,
        label: 'qm.closeLid',
        pressed: c.screens?.lidClosed ?? false,
      })
  }
  list.push(
    { id: MoreId.Restart, icon: RefreshCw, label: 'qm.reset' },
    { id: MoreId.Close, icon: Power, label: 'qm.close', danger: true },
  )
  return list
}

/** «Rendi rapido» can make this slot the quick one: a numbered slot the shortcuts reach (1–9,
 * `hotkeys::FIRST_SLOT..=LAST_SLOT`), not the one already quick. */
export const QUICK_SLOT_FIRST = 1
export const QUICK_SLOT_LAST = 9
export function canMakeQuick(slot: number, quick: number): boolean {
  return slot >= QUICK_SLOT_FIRST && slot <= QUICK_SLOT_LAST && slot !== quick
}

/** The steps of «slot successivo» that move the quick slot from `from` to `to` (both in 1–9),
 * the shorter way round, and its direction. */
export function quickSlotSteps(from: number, to: number): { forward: boolean; count: number } {
  const n = QUICK_SLOT_LAST - QUICK_SLOT_FIRST + 1
  const ahead = (((to - from) % n) + n) % n
  return ahead <= n - ahead ? { forward: true, count: ahead } : { forward: false, count: n - ahead }
}
