// Where the search everywhere (X01, #32) is opened from, and what can be chosen there: the shell
// (no game open) or the quick menu (a game open, paused under it). Pure: the reasons are Messages,
// said by useSearchSources; what choosing does is the host's (useSearch.ts).
//
// In the shell a controller shortcut acts on a game that is not there, so only the keyboard's
// fullscreen runs (X05). In the quick menu the shortcuts run as the menu's own tiles do: Salva,
// Carica and the slot steps on the slot chosen in the menu; a held shortcut (fast forward held,
// rewind) and the pause (the menu already pauses the game) cannot, the screen shortcuts need a
// game with two screens, the lid a core that has one. A setting opens only where the quick menu
// shows it «in prova»; the game being played cannot be opened again.
import type { GameId, SettingKey, StateLoadBlock } from '@/emu'
import type { Message } from '@/i18n/message'
import type { KeyAction } from '@/lib/keyboard'

/** The game open under the quick menu, as the search needs it. */
export interface GameContext {
  /** The game being played. */
  playing: GameId
  /** NDS / 3DS: the screen shortcuts (swap, layout, lid). */
  dualScreen: boolean
  /** The core can close the lid. */
  lidSupported: boolean
  /** Why loading states is off for this game (Trello #93), else null. */
  loadBlock: StateLoadBlock | null
  /** The slot chosen in the menu (Salva and Carica act on it). */
  slot: { chosen: number; empty: boolean; auto: boolean }
  /** The settings the quick menu shows «in prova» on one of its pages. */
  settings: readonly SettingKey[]
}

/** Where the search is: `game` null in the shell. */
export interface SearchContext {
  game: GameContext | null
}

export const SHELL: SearchContext = { game: null }

const NEEDS_GAME: Message = { key: 'search.needsGame' }
const HELD: Message = { key: 'search.heldInGame' }
const TWO_SCREENS: Message = { key: 'search.twoScreensOnly' }

/** Why the action `id` cannot run here, or null when it can. */
export function actionReason(id: KeyAction, c: SearchContext): Message | null {
  const g = c.game
  if (!g) return id === 'fullscreen' ? null : NEEDS_GAME
  switch (id) {
    case 'menu':
    case 'next_slot':
    case 'prev_slot':
    case 'ff_toggle':
    case 'slow':
    case 'screenshot':
    case 'fps':
    case 'fullscreen':
      return null
    case 'save':
      return g.slot.auto ? { key: 'qm.autoUpdates' } : null
    case 'load':
      if (g.loadBlock !== null) return { key: `flow.loadBlock.${g.loadBlock}` }
      return g.slot.empty ? { key: 'qm.emptySlot', params: { slot: g.slot.chosen } } : null
    case 'ff_hold':
    case 'rewind':
      return HELD
    case 'pause':
      return { key: 'search.pausedInMenu' }
    case 'swap':
    case 'layout':
      return g.dualScreen ? null : TWO_SCREENS
    case 'lid':
      if (!g.dualScreen) return TWO_SCREENS
      return g.lidSupported ? null : { key: 'qm.lidUnsupported' }
  }
}

/** Why the setting `key` cannot be opened here, or null when it can. */
export function settingReason(key: SettingKey, c: SearchContext): Message | null {
  if (!c.game || c.game.settings.includes(key)) return null
  return { key: 'search.settingInLibrary' }
}

/** Why the library game `id` cannot be chosen here, or null when it can. */
export function gameReason(id: GameId, c: SearchContext): Message | null {
  return c.game?.playing === id ? { key: 'search.playingNow' } : null
}
