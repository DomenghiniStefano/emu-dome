// The game being played: its state (gameState.ts) and what the quick menu and the keys do to it.
// The controller combos and the core's events are useGameEvents', how a game ends (G14, E11)
// useGameEnding's; this file puts them together as one session. Opening a game is
// useGameLaunch's.
import { watch } from 'vue'
import { AUTO_SLOT, emu, type GameInfo, type LibraryGame, type Speed } from '@/emu'
import { flow, gameStarted } from './gameFlow'
import { report } from '@/lib/errors'
import { loadKeymap } from '@/lib/keymap'
import { insetShare } from './session'
import { NORMAL } from '@/lib/speeds'
import { createGameState } from './gameState'
import { useGameEnding } from './useGameEnding'
import { useGameEvents } from './useGameEvents'
import { useGameSettings } from './useGameSettings'
import type { Toasts } from '@/composables/useToasts'

export function useGameSession(toasts: Toasts) {
  const state = createGameState()
  const {
    game,
    playingGame,
    menuOpen,
    paused,
    speed,
    comboSlot,
    undoLoadSlot,
    muted,
    fullscreen,
    menuInset,
  } = state
  const { settings, load: loadSettings, concerns: settingsConcern } = useGameSettings(game, toasts)
  const ending = useGameEnding(state)
  const events = useGameEvents(state, settings, toasts, { setMenu, stopped: ending.stopped })

  // While the menu is open, Rust sends the controllers to the menu (emu://nav), not to the game.
  watch(menuOpen, open => report(emu.setMenuOpen(open)))
  // The keyboard's keys follow the open game's settings, or «Tutti i giochi» without one.
  watch(
    () => game.value !== null,
    inGame => void loadKeymap(inGame),
  )

  /** Whether fullscreen is on and the current slot, as Rust has them at startup. */
  async function loadDeviceState() {
    fullscreen.value = await emu.isFullscreen()
    comboSlot.value = await emu.controls.currentSlot()
  }

  /** The core runs `info`: a fresh session, then the auto quick save or the resume question. */
  async function begin(info: GameInfo, from: LibraryGame | null, resumeAuto = false) {
    playingGame.value = from
    game.value = info
    speed.value = { ...NORMAL }
    menuOpen.value = false
    paused.value = false
    undoLoadSlot.value = null
    events.reset()
    muted.value = false
    menuInset.value = 0
    gameStarted()
    void loadSettings()
    if (resumeAuto) {
      await emu.loadState(AUTO_SLOT)
    } else if (info.resumable) {
      // `saves.resume` = ask: hold the game until the answer.
      await emu.setPaused(true)
      flow.resumeAsk = true
    }
  }

  /** Opens or closes the quick menu once the core is paused or running again; a failure leaves
   * the menu as it was and is thrown to the caller. */
  async function setMenu(open: boolean) {
    // While the game closes (#86) the quick menu does not open.
    if (!game.value || (open && flow.ending)) return
    await emu.setPaused(open)
    menuOpen.value = open
    paused.value = false
  }
  const toggleMenu = () => setMenu(!menuOpen.value)

  async function setSpeed(s: Speed) {
    speed.value = s
    await emu.setSpeed(s)
  }

  /** Salva in the quick menu (the event `stateSaved` follows). */
  const save = (slot: number) => emu.saveState(slot)

  /** G11 «Riavvia»: the game starts over and resumes. */
  async function restart() {
    try {
      await emu.reset()
      await setMenu(false)
    } catch (e) {
      toasts.failWith(e)
    }
  }

  async function setMuted(on: boolean) {
    try {
      await emu.setMuted(on)
      muted.value = on
    } catch (e) {
      toasts.failWith(e)
    }
  }

  async function toggleFullscreen() {
    try {
      fullscreen.value = await emu.toggleFullscreen()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** G09's bar: the canvas keeps clear of it by itself, the native video is told the share. */
  function setInset(px: number) {
    menuInset.value = px
    report(emu.setVideoInset(insetShare(px, window.innerHeight)))
  }

  /** F12, L3 + B (in Rust) and quick menu › Strumenti: G13 «Schermata salvata» on `screenshotSaved`. */
  async function screenshot() {
    try {
      await emu.screenshot()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  return {
    ...state,
    settings,
    loadSettings,
    settingsConcern,
    loadDeviceState,
    begin,
    end: ending.end,
    close: ending.close,
    crashed: ending.crashed,
    setMenu,
    toggleMenu,
    setSpeed,
    save,
    loadFromMenu: events.loadFromMenu,
    undoLoad: events.undoLoad,
    restart,
    setMuted,
    toggleFullscreen,
    setInset,
    onHotkey: events.onHotkey,
    screenshot,
    onEmuEvent: events.onEmuEvent,
    watchStats: ending.watchStats,
  }
}
export type GameSession = ReturnType<typeof useGameSession>
