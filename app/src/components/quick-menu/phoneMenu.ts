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
  Hand,
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
  /** «Chiudi il gioco» (Trello #121): the close flow, as the desktop's foot. */
  Close: 'close',
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
  /** Message key of its accessible name when the short label says less («Chiudi» for «Chiudi
   * il gioco»). */
  name?: string
  /** «Chiudi il gioco»: in danger ink. */
  danger?: boolean
}

export interface StripContext {
  /** The resolved `speed.rew` of the open game. */
  rewind: boolean
  /** The game runs at the fast forward speed. */
  fastForward: boolean
  muted: boolean
}

/** The strip under the three tiles: «Riavvolgi» first when rewind is on for the game, then
 * «Accelera», «Silenzia», «Chiudi il gioco» (one tap, Trello #121), «Altro» last. */
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
    { id: StripId.Close, icon: Power, label: 'qm.verb.close', name: 'qm.close', danger: true },
    { id: StripId.More, icon: Ellipsis, label: 'qm.moreLabel' },
  )
  return list
}

/** On its side the panel is 300 px wide: its strip holds four buttons in one row, and five
 * («Riavvolgi» on) would cut «Riavvolgi» and «Accelera», so they go in two rows, three above and
 * two below (Trello #121). Upright one row always fits. */
export const SIDE_STRIP_ONE_ROW = 4
export function stripTwoRows(side: boolean, count: number): boolean {
  return side && count > SIDE_STRIP_ONE_ROW
}

export const MoreId = {
  Saves: 'saves',
  Speed: 'speed',
  Screens: 'screens',
  Controls: 'controls',
  /** «Modifica la disposizione»: the C06 touch controls editor, one tap less than through
   * «Controlli» (Trello #111). */
  Layout: 'layout',
  Screenshot: 'shot',
  SwapScreens: 'swap',
  CloseLid: 'lid',
  Restart: 'restart',
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
}

export interface MoreContext {
  /** NDS and 3DS. */
  dualScreen: boolean
  /** The open game's screens (null until known: the lid then waits, listed). */
  screens: ScreenStatus | null
}

/** The rows of «Altro», top to bottom; «Riavvia il gioco» last. «Chiudi il gioco» is in the
 * strip (Trello #121), one tap from the first level. */
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
    {
      id: MoreId.Layout,
      icon: Hand,
      label: 'qm.editLayout',
      page: QuickMenuView.TouchEditor,
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
  list.push({ id: MoreId.Restart, icon: RefreshCw, label: 'qm.reset' })
  return list
}

/** «Rendi rapido» can make this slot the quick one: any numbered slot the page offers (1 to
 * `saves.slots`, the tenth too: the shortcuts cycle through 1–9 only, #149), not the automatic
 * one (0) nor the one already quick. */
export function canMakeQuick(slot: number, quick: number): boolean {
  return slot >= 1 && slot !== quick
}
