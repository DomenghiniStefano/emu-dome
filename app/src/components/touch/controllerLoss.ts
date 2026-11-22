// E14 «Controller scollegato» (owner, 2026-11-22), pure: what the phone does when the last
// controller goes away while a game is played. `touch.tlost` (per device):
// - «Pausa e domanda» (`pause`, the default): the game pauses and asks (ControllerLostDialog),
//   «Aspetta il controller» or «Continua con il tocco»; the question closes by itself, and the
//   game goes on, when a controller comes back;
// - «Passa al tocco» (`touch`): no pause, the touch controls come back with a short toast.
// Only on a phone: the desktop has the TV question of #30 for the same event
// (lib/uiMode.ts `asksOnDisconnect`, which never asks on a phone), so the two never show
// together, and the shell's generic «controller disconnected» toast stays quiet when either
// speaks.
import type { SettingDevice } from '@/emu'

export type LostChoice = 'pause' | 'touch'

/** A `touch.tlost` value: anything unknown is the default, «Pausa e domanda». */
export const lostChoice = (value: unknown): LostChoice => (value === 'touch' ? 'touch' : 'pause')

export interface LossInputs {
  /** What the settings schema says this device is (null: not read yet). */
  device: SettingDevice | null
  inGame: boolean
  /** The quick menu is open: the game is paused already and the menu has the input. */
  menuOpen: boolean
  /** Something else holds the game (a game-flow question, the game closing, the C06 editor). */
  busy: boolean
  /** Controllers still connected after this one left. */
  left: number
}

/** What happens: the question, the toast, or nothing (not a phone, not playing, one is left). */
export function onControllerLost(i: LossInputs, choice: LostChoice): 'ask' | 'toast' | null {
  if (i.device !== 'phone' || !i.inGame || i.menuOpen || i.busy || i.left > 0) return null
  return choice === 'pause' ? 'ask' : 'toast'
}
