// How a game ends: G14 «Chiusura» (closed on purpose, the auto quick save and the in-game save
// first), or E11 (the emulator stopped, or stopped answering), found by the stats poll that also
// feeds the HUD and E12 «lento».
import { onUnmounted } from 'vue'
import { emu, type EmuEvent, type Stats } from '@/emu'
import { ignore, reportError } from '@/lib/errors'
import { asksBeforeClose, CloseAnswer, saveOutcome } from './closeAsk'
import { flow, sample } from './gameFlow'
import { crashReport } from './session'
import type { GameState } from './gameState'

/** How long G14 «Chiusura» stays at least, so it can be read. */
const CLOSING_MIN_MS = 400
/** How often the stats are read while a game is open (HUD, E11 and E12 watchdogs). */
const STATS_EVERY_MS = 500
/** «Salva e chiudi» waits this long for its quick save (Rust's `CLOSE_TIMEOUT`: a 3DS state of
 * tens of MB on a slow disk), then keeps the game open. */
export const SAVE_WAIT_MS = 10_000

export function useGameEnding(state: GameState) {
  const { game, menuOpen, paused, stats, comboSlot } = state
  // True while closing on purpose: the core's `stopped` event is then expected.
  let closingNow = false
  // The answer «Chiudere senza salvare?» waits for, and the quick save «Salva e chiudi» waits for.
  let answer: ((a: CloseAnswer) => void) | null = null
  let pendingSave: { slot: number; done: (saved: boolean) => void } | null = null

  /** The game is gone (closed, or abandoned after E11): back to the shell. */
  function end() {
    game.value = null
    // A question still open goes with the game.
    answerClose(CloseAnswer.Cancel)
    pendingSave?.done(false)
  }

  /** The user's answer to «Chiudere senza salvare?». */
  function answerClose(a: CloseAnswer) {
    const waiting = answer
    answer = null
    waiting?.(a)
  }

  /** The core's events while «Salva e chiudi» waits for its quick save. */
  function onSaveEvent(ev: EmuEvent) {
    if (!pendingSave) return
    const outcome = saveOutcome(ev, pendingSave.slot)
    if (outcome) pendingSave.done(outcome === 'saved')
  }

  /** A quick save into `slot`, waited for: true once written. A failure reaches the user through
   * the core's error event (useGameEvents' toast); no answer in time, through the error sink. */
  function saveNow(slot: number): Promise<boolean> {
    return new Promise(resolve => {
      const timer = window.setTimeout(() => {
        reportError({ code: 'stateSaveFailed', params: { slot, cause: 'other' } })
        pendingSave?.done(false)
      }, SAVE_WAIT_MS)
      pendingSave = {
        slot,
        done: saved => {
          window.clearTimeout(timer)
          pendingSave = null
          resolve(saved)
        },
      }
      emu.saveState(slot).catch((e: unknown) => {
        reportError(e)
        pendingSave?.done(false)
      })
    })
  }

  /**
   * The quick menu's «Chiudi il gioco» (desktop, TV, the phone's strip) and the search's other
   * game: with progress not in a quick save (Rust's `Stats.unsaved`) «Chiudere senza salvare?»
   * first, then G14. True once the game closed; false when it stays open (the question's
   * «Annulla», a quick save that failed, the game gone meanwhile).
   */
  async function requestClose(): Promise<boolean> {
    const playing = game.value
    if (!playing || flow.closeAsk || flow.ending) return false
    let now: Stats | null = null
    try {
      now = await emu.stats()
    } catch (e) {
      // Only the bridge can fail: asked all the same (closeAsk.ts).
      reportError(e)
    }
    if (game.value !== playing) return false
    if (!asksBeforeClose(now)) {
      await close()
      return true
    }
    flow.closeAsk = { autosave: playing.autosaveOnExit, saving: false }
    const chosen = await new Promise<CloseAnswer>(r => (answer = r))
    if (game.value !== playing || chosen === CloseAnswer.Cancel) {
      flow.closeAsk = null
      return false
    }
    if (chosen === CloseAnswer.Save) {
      if (flow.closeAsk) flow.closeAsk.saving = true
      const saved = await saveNow(comboSlot.value)
      if (!saved || game.value !== playing) {
        flow.closeAsk = null
        return false
      }
    }
    flow.closeAsk = null
    await close()
    return true
  }

  /** G14: auto quick save and in-game save, then back to the section you came from. */
  async function close() {
    if (!game.value) return
    flow.closing = { title: game.value.title, autosave: game.value.autosaveOnExit }
    menuOpen.value = false
    closingNow = true
    const shown = Date.now()
    try {
      await emu.closeGame()
    } catch (e) {
      // Only the bridge can fail (`close_game` answers nothing): back to the shell all the same.
      reportError(e)
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

  /** The core stopped: E11, unless the game is being closed on purpose (here or by the flow). */
  function stopped() {
    if (!closingNow && !flow.ending) crashed()
  }

  /** One read of the stats; dropped if the game changed meanwhile. */
  async function poll() {
    const playing = game.value
    if (!playing) return
    const s = await emu.stats()
    if (game.value !== playing) return
    stats.value = s
    const running =
      !paused.value &&
      !menuOpen.value &&
      !flow.resumeAsk &&
      !flow.crashed &&
      !flow.slow &&
      !flow.ending
    const seen = sample(s, running)
    if (seen === 'hang') crashed()
    else if (seen === 'slow') flow.slow = { fps: s.emuFps, target: s.targetFps }
  }

  /**
   * The stats poll: the HUD's FPS, and the E11 / E12 watchdogs while the game should run. One read
   * at a time: a slow answer is not overtaken by the next tick.
   */
  let statsTimer: number | undefined
  let polling = false
  function watchStats() {
    statsTimer = window.setInterval(() => {
      if (polling) return
      polling = true
      // Logged only: `stats` cannot fail in Rust (it reads shared memory, a hung core included),
      // so a failure is the bridge's, and the next tick asks again.
      ignore(
        poll().finally(() => (polling = false)),
        'stats read failed',
      )
    }, STATS_EVERY_MS)
  }
  onUnmounted(() => window.clearInterval(statsTimer))

  return {
    end,
    close,
    requestClose,
    answerClose,
    onSaveEvent,
    crashed,
    stopped,
    watchStats,
  }
}
