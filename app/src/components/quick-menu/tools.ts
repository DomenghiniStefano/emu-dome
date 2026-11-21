// The tools of the quick menu (Strumenti): which ones a game gets, and why one cannot be used.
import type { Component } from 'vue'
import { ArrowDownUp, Camera, Laptop, Maximize, RefreshCw, VolumeX } from '@lucide/vue'
import type { ScreenStatus } from '@/emu'

export const ToolId = {
  Screenshot: 'shot',
  Mute: 'mute',
  Fullscreen: 'full',
  SwapScreens: 'swap',
  CloseLid: 'lid',
  Restart: 'restart',
} as const
export type ToolId = (typeof ToolId)[keyof typeof ToolId]

export interface Tool {
  id: ToolId
  icon: Component
  /** Message key of the label. */
  label: string
  /** False: shown disabled, and named in the note under the tools (`why`, or «not yet»). */
  available: boolean
  /** Why an unavailable tool cannot be used here (message key); none: no backend yet. */
  why?: string
  /** A tool that stays on (Muto, A schermo intero, Scambia, the lid): whether it is on now. */
  pressed?: boolean
}

export interface ToolContext {
  /** The phone has no full screen to toggle. */
  phone: boolean
  /** NDS and 3DS: Scambia gli schermi and Chiudi il coperchio. */
  dualScreen: boolean
  muted: boolean
  fullscreen: boolean
  /** The open game's screens (null until known: the lid then waits, enabled). */
  screens: ScreenStatus | null
}

/** The tools in the order the menu shows them. */
export function quickTools(c: ToolContext): Tool[] {
  const list: Tool[] = [
    { id: ToolId.Screenshot, icon: Camera, label: 'qm.screenshot', available: true },
    { id: ToolId.Mute, icon: VolumeX, label: 'qm.mute', available: true, pressed: c.muted },
  ]
  if (!c.phone)
    list.push({
      id: ToolId.Fullscreen,
      icon: Maximize,
      label: 'qm.fullscreen',
      available: true,
      pressed: c.fullscreen,
    })
  if (c.dualScreen) {
    // melonDS DS closes the lid, Azahar has none (`lidSupported`): disabled with the reason.
    const lid = c.screens?.lidSupported ?? true
    list.push(
      {
        id: ToolId.SwapScreens,
        icon: ArrowDownUp,
        label: 'qm.swapScreens',
        available: true,
        pressed: c.screens?.swapped ?? false,
      },
      {
        id: ToolId.CloseLid,
        icon: Laptop,
        label: 'qm.closeLid',
        available: lid,
        ...(lid ? {} : { why: 'qm.lidUnsupported' }),
        pressed: c.screens?.lidClosed ?? false,
      },
    )
  }
  list.push({ id: ToolId.Restart, icon: RefreshCw, label: 'qm.reset', available: true })
  return list
}

/** The label keys of the tools that have no backend yet, for the note under them. */
export function toolsNotYet(tools: Tool[]): string[] {
  return tools.filter(x => !x.available && !x.why).map(x => x.label)
}

/** Why the other unavailable tools cannot be used (message keys), for the same note. */
export function toolReasons(tools: Tool[]): string[] {
  return tools.flatMap(x => (!x.available && x.why ? [x.why] : []))
}
