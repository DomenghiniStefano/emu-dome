// What a touch button does under the finger (Trello #29, the C06 editor's «Comportamento»), pure:
// - tap («Tocco»): pressed while the finger is on it;
// - hold («Tieni premuto»): a tap holds it down, the next tap lets it go;
// - turbo: pressed and released over and over while the finger is on it. The core reads the
//   buttons once per frame (a snapshot before each frame), so each half of the cycle lasts
//   several frames and none is missed.
// The core has no turbo of its own (the shell's «turbo» is fast forward): the repeat is here,
// sending the same `input` presses as a tap.
import type { Button, TouchBehaviour } from '@/emu'

/** Half a turbo cycle, ms: about 12 presses a second, each held for 2–3 frames at 60 FPS. */
export const TURBO_HALF_MS = 40

/** A repeating timer, replaceable in tests. Returns how to stop it. */
export type Every = (ms: number, tick: () => void) => () => void

const realEvery: Every = (ms, tick) => {
  const id = setInterval(tick, ms)
  return () => clearInterval(id)
}

export interface Presser {
  /** A finger went down on `button`: true when that is a press to feel (the haptic tap). */
  down(button: Button, behaviour: TouchBehaviour): boolean
  /** The finger left it. */
  up(button: Button, behaviour: TouchBehaviour): void
  /** Whether `button` is held down by a «hold» tap. */
  latched(button: Button): boolean
  /** Lets go of everything (the controls going away, the quick menu opening). */
  releaseAll(): void
}

/** `send` reaches the game (`emu.input`); it is called only when a button's state changes. */
export function createPresser(
  send: (button: Button, pressed: boolean) => void,
  every: Every = realEvery,
): Presser {
  const pressed = new Set<Button>()
  const latched = new Set<Button>()
  const turbos = new Map<Button, () => void>()

  function set(button: Button, on: boolean) {
    if (on === pressed.has(button)) return
    if (on) pressed.add(button)
    else pressed.delete(button)
    send(button, on)
  }

  function stopTurbo(button: Button) {
    turbos.get(button)?.()
    turbos.delete(button)
  }

  return {
    down(button, behaviour) {
      switch (behaviour) {
        case 'tap':
          set(button, true)
          return true
        case 'hold':
          if (latched.delete(button)) set(button, false)
          else {
            latched.add(button)
            set(button, true)
          }
          return true
        case 'turbo': {
          if (turbos.has(button)) return false
          set(button, true)
          turbos.set(
            button,
            every(TURBO_HALF_MS, () => set(button, !pressed.has(button))),
          )
          return true
        }
      }
    },
    up(button, behaviour) {
      switch (behaviour) {
        case 'tap':
          return set(button, false)
        case 'hold':
          return
        case 'turbo':
          stopTurbo(button)
          return set(button, false)
      }
    },
    latched: button => latched.has(button),
    releaseAll() {
      for (const b of [...turbos.keys()]) stopTurbo(b)
      latched.clear()
      for (const b of [...pressed]) set(b, false)
    },
  }
}
