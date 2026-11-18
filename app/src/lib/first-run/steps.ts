// The first run's steps (#35): which one shows, where «Continua» and «Indietro» go, what the
// main button says. Pure; useFirstRunSteps holds the state.
import type { FirstRunStep } from '../../emu'

/** Before Rust answers: the steps without a controller. */
export const DEFAULT_STEPS: FirstRunStep[] = ['welcome', 'games', 'ready']

/** The step to show once the list changed (a controller came or went): the same one when it is
 * still there, else the one now at its place (unplugged on S04 → S06). */
export function stepAfterChange(
  current: FirstRunStep,
  index: number,
  next: FirstRunStep[],
): FirstRunStep {
  if (next.includes(current)) return current
  return next[Math.min(index, next.length - 1)] ?? 'ready'
}

/** The main button's message key: start on the first step, finish on the last. */
export function mainActionKey(index: number, count: number): string {
  if (index === 0) return 'firstRun.start'
  return index === count - 1 ? 'firstRun.finish' : 'firstRun.next'
}
