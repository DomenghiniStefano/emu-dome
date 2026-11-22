// The three speeds of the quick menu (G07): Normale, Avanti veloce (`speed.ffval`) and
// Rallentatore (`speed.slow`), and the game following G16 when the speed in use changes.
import type { Speed } from '@/emu'
import { FF_DEFAULT, NORMAL, SLOW_DEFAULT, sameSpeed, speedFromOption } from '@/lib/speeds'

export const SpeedChoiceId = {
  Normal: 'normal',
  FastForward: 'ff',
  SlowMotion: 'slow',
} as const
export type SpeedChoiceId = (typeof SpeedChoiceId)[keyof typeof SpeedChoiceId]

export interface SpeedChoice {
  id: SpeedChoiceId
  speed: Speed
}

/** The speeds the two settings' options give (their defaults when unset or unreadable). */
export function modeSpeeds(ffOption: unknown, slowOption: unknown): { ff: Speed; slow: Speed } {
  return {
    ff: speedFromOption(ffOption) ?? FF_DEFAULT,
    slow: speedFromOption(slowOption) ?? SLOW_DEFAULT,
  }
}

export function speedChoices(ff: Speed, slow: Speed): SpeedChoice[] {
  return [
    { id: SpeedChoiceId.Normal, speed: NORMAL },
    { id: SpeedChoiceId.FastForward, speed: ff },
    { id: SpeedChoiceId.SlowMotion, speed: slow },
  ]
}

/** The choice the game runs at now; '' when none (a speed set elsewhere). */
export function chosenSpeed(choices: SpeedChoice[], current: Speed): SpeedChoiceId | '' {
  return choices.find(c => sameSpeed(current, c.speed))?.id ?? ''
}

/**
 * G16 changed the speed of the mode in use: the speed the game must follow at once, else null.
 * Fast forward is checked first, as the menu always did.
 */
export function followedSpeed(
  current: Speed,
  now: { ff: Speed; slow: Speed },
  before: { ff: Speed; slow: Speed },
): Speed | null {
  if (!sameSpeed(now.ff, before.ff) && sameSpeed(current, before.ff)) return now.ff
  if (!sameSpeed(now.slow, before.slow) && sameSpeed(current, before.slow)) return now.slow
  return null
}

/**
 * «Avanti veloce» / «Rallentatore» as a toggle (the shortcuts', chosen from the search): the mode's
 * speed, or back to normal when the game already runs at it.
 */
export function toggledSpeed(current: Speed, mode: Speed): Speed {
  return sameSpeed(current, mode) ? NORMAL : mode
}
