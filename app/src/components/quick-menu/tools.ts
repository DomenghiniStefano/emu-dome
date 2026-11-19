// The tools of the quick menu (Strumenti): which ones a game gets, and which have no backend yet.
import type { Component } from 'vue'
import { ArrowDownUp, Camera, Laptop, Maximize, RefreshCw, VolumeX } from '@lucide/vue'

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
  /** False: no backend yet, shown disabled and named in the note under the tools. */
  available: boolean
  /** A tool that stays on (Muto, A schermo intero): whether it is on now. */
  pressed?: boolean
}

export interface ToolContext {
  /** The phone has no full screen to toggle. */
  phone: boolean
  /** NDS and 3DS: Scambia schermi and Chiudi lo sportello. */
  dualScreen: boolean
  muted: boolean
  fullscreen: boolean
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
  if (c.dualScreen)
    list.push(
      { id: ToolId.SwapScreens, icon: ArrowDownUp, label: 'qm.swapScreens', available: false },
      { id: ToolId.CloseLid, icon: Laptop, label: 'qm.closeLid', available: false },
    )
  list.push({ id: ToolId.Restart, icon: RefreshCw, label: 'qm.reset', available: true })
  return list
}

/** The label keys of the tools that are not available yet, for the note under them. */
export function toolsNotYet(tools: Tool[]): string[] {
  return tools.filter(x => !x.available).map(x => x.label)
}
