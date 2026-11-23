// The game flow around the emulator (step 4, boards «Stati» G01 G02 G06 G13 G14 and «Errori»
// E11 E12): what is on screen while a game starts, fails, runs, closes, stops or runs slow.
// App.vue drives it (open / close / stats / events); GameFlowOverlays.vue draws it.
import { reactive } from 'vue'
import type { ConsoleId, Stats } from '@/emu'
import type { StartProblem } from './launch'

/** E11: frames stand still this long while the game should run → the core stopped answering. */
export const HANG_MS = 5000
/** E12: this long under 90 % of the target FPS at 1× → suggest a lighter setting (once per game). */
export const SLOW_MS = 10_000
/** G06: the hint shows on the first few games only. */
const HINT_GAMES = 3
const HINT_KEY = 'emudome.menuHintShown'

export const flow = reactive({
  /** G01: the game is starting (cover colour, title). */
  launching: null as null | { title: string; colour: string | null },
  /** G02: the game could not start: why, and the file or folder it is about. */
  startError: null as null | ({ title: string } & StartProblem),
  /** E04: the in-game save could not be read and was set aside; `backupAt` = newest backup. */
  saveBroken: null as null | { title: string; backupAt: number | null },
  /** G06: «Esc o Home per il menu rapido», fades after 4 s. */
  hint: false,
  /** «Riprendi» asked at start (`saves.resume` = ask, an auto quick save exists). */
  resumeAsk: false,
  /** G14: closing (auto quick save, in-game save). */
  closing: null as null | { title: string; autosave: boolean },
  /** The close flow runs (#86, `emu://closing`: the window, the app or the game closing). */
  ending: false,
  /** E11: the emulator stopped. `at` = Unix ms of the last auto quick save, if any. */
  crashed: null as null | { console: ConsoleId; at: number | null; detail: string },
  /** E12: slower than normal. */
  slow: null as null | { fps: number; target: number },
})

// ---- Watchdogs, fed by the stats poll ----
let lastFrames = -1
let lastChange = 0
let slowSince: number | null = null
let slowShown = false

/** A game started: watchdogs from zero, the hint the first few times. */
export function gameStarted() {
  lastFrames = -1
  lastChange = Date.now()
  slowSince = null
  slowShown = false
  flow.crashed = null
  flow.slow = null
  let shown = 0
  try {
    shown = Number(localStorage.getItem(HINT_KEY) ?? '0') || 0
    localStorage.setItem(HINT_KEY, String(shown + 1))
  } catch {
    // Storage unavailable: show the hint, it is only a convenience.
  }
  flow.hint = shown < HINT_GAMES
  if (flow.hint) window.setTimeout(() => (flow.hint = false), 4000)
}

/**
 * One stats sample while a game is open. `running` = the game should advance (not paused,
 * no menu, no dialog). Returns true when the core looks stuck (E11).
 */
export function sample(s: Stats, running: boolean, now = Date.now()): 'hang' | 'slow' | null {
  if (!running || !s.running) {
    lastChange = now
    slowSince = null
    lastFrames = s.frames
    return null
  }
  if (s.frames !== lastFrames) {
    lastFrames = s.frames
    lastChange = now
  } else if (now - lastChange >= HANG_MS) {
    return 'hang'
  }
  const atNormalSpeed = s.speed?.kind === 'factor' && s.speed.value === 1
  if (atNormalSpeed && s.targetFps > 0 && s.emuFps > 0 && s.emuFps < s.targetFps * 0.9) {
    slowSince ??= now
    if (!slowShown && now - slowSince >= SLOW_MS) {
      slowShown = true
      return 'slow'
    }
  } else {
    slowSince = null
  }
  return null
}
