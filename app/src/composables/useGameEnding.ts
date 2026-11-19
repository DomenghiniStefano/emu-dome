// How a game ends: G14 «Chiusura» (closed on purpose, the auto quick save and the in-game save
// first), or E11 (the emulator stopped, or stopped answering), found by the stats poll that also
// feeds the HUD and E12 «lento».
import { onUnmounted } from 'vue'
import { emu } from '../emu'
import { flow, sample } from '../gameFlow'
import { crashReport } from '../lib/shell/session'
import type { GameState } from './gameState'

/** How long G14 «Chiusura» stays at least, so it can be read. */
const CLOSING_MIN_MS = 400
/** How often the stats are read while a game is open (HUD, E11 and E12 watchdogs). */
const STATS_EVERY_MS = 500

export function useGameEnding(state: GameState) {
  const { game, menuOpen, paused, stats } = state
  // True while closing on purpose: the core's `stopped` event is then expected.
  let closingNow = false

  /** The game is gone (closed, or abandoned after E11): back to the shell. */
  const end = () => (game.value = null)

  /** G14: auto quick save and in-game save, then back to the section you came from. */
  async function close() {
    if (!game.value) return
    flow.closing = { title: game.value.title, autosave: game.value.autosaveOnExit }
    menuOpen.value = false
    closingNow = true
    const shown = Date.now()
    try {
      await emu.closeGame()
    } finally {
      // Long enough to read, never longer than the work.
      await new Promise(r =>
        window.setTimeout(r, Math.max(0, CLOSING_MIN_MS - (Date.now() - shown))),
      )
      closingNow = false
      flow.closing = null
      end()
    }
  }

  /** E11: the emulator stopped, or stopped answering. */
  function crashed() {
    if (!game.value || flow.crashed) return
    menuOpen.value = false
    flow.crashed = crashReport(game.value, stats.value, new Date())
  }

  /** The core stopped: E11, unless the game is being closed on purpose. */
  function stopped() {
    if (!closingNow) crashed()
  }

  /** The stats poll: the HUD's FPS, and the E11 / E12 watchdogs while the game should run. */
  let statsTimer: number | undefined
  function watchStats() {
    statsTimer = window.setInterval(async () => {
      if (!game.value) return
      const s = await emu.stats()
      stats.value = s
      const running =
        !paused.value && !menuOpen.value && !flow.resumeAsk && !flow.crashed && !flow.slow
      const seen = sample(s, running)
      if (seen === 'hang') crashed()
      else if (seen === 'slow') flow.slow = { fps: s.emuFps, target: s.targetFps }
    }, STATS_EVERY_MS)
  }
  onUnmounted(() => window.clearInterval(statsTimer))

  return { end, close, crashed, stopped, watchStats }
}
