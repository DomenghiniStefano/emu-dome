// The game being played: its state (gameState.ts) and what the quick menu and the keys do to it.
// The controller combos and the core's events are useGameEvents', how a game ends (G14, E11)
// useGameEnding's; this file puts them together as one session. Opening a game is
// useGameLaunch's.
import { ref, watch } from 'vue'
import {
  AUTO_SLOT,
  emu,
  type CustomScreen,
  type EmuEvent,
  type GameInfo,
  type LibraryGame,
  type Speed,
} from '@/emu'
import { flow, gameStarted } from './gameFlow'
import { report } from '@/lib/errors'
import { loadKeymap } from '@/lib/keymap'
import { afterRefusal, effectivePlacement, phoneSend, type PhoneView } from './placement'
import { insetShares, NO_INSET, sameInset, type InsetPx } from './inset'
import { createGameState } from './gameState'
import { useGameEnding } from './useGameEnding'
import { useGameEvents } from './useGameEvents'
import { useGameSettings } from './useGameSettings'
import type { Toasts } from '@/composables/useToasts'
import type { CoverRequest } from '@/components/cover/useCoverRequest'

/** `useAsCover`: the screenshot toast's «Usa come copertina» (Trello #85, App.vue's request). */
export function useGameSession(toasts: Toasts, useAsCover?: (r: CoverRequest) => void) {
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
    videoInset,
  } = state
  const { settings, load: loadSettings, concerns: settingsConcern } = useGameSettings(game, toasts)
  const ending = useGameEnding(state)
  const events = useGameEvents(state, settings, toasts, {
    setMenu,
    stopped: ending.stopped,
    useAsCover,
  })

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

  /** Goes up at every launch (`begin`): a consumer that has sent something per game sends it again. */
  const launches = ref(0)

  /** The phone's placement of the screens (Trello #134): the touch layer's saved one and the
   * editor's while it is open; the editor's wins. It goes to Rust with the inset it is fitted in
   * (`setInset`): what one change sets (TouchLayer and TouchEditor send the placement, then its
   * inset) is sent once that change is done, as one call, and only what Rust does not hold yet. */
  let layerPlacement: CustomScreen[] = []
  let editorPlacement: CustomScreen[] | null = null
  let wantedShares = insetShares(NO_INSET, 1, 1)
  let sent: PhoneView = { placement: [], inset: wantedShares }
  /** The placement Rust refused as invalid last: not sent again (its inset still is). */
  let refused: CustomScreen[] | null = null
  let queued = false
  function setPlacement(from: 'layer' | 'editor', screens: CustomScreen[] | null) {
    if (from === 'layer') layerPlacement = screens ?? []
    else editorPlacement = screens
    queueView()
  }
  function queueView() {
    if (queued) return
    queued = true
    queueMicrotask(sendView)
  }
  function sendView() {
    queued = false
    const next: PhoneView = {
      placement: effectivePlacement(layerPlacement, editorPlacement),
      inset: wantedShares,
    }
    const kind = phoneSend(sent, next, refused)
    if (!kind) return
    const before = sent
    sent = next
    const call =
      kind === 'pair'
        ? emu.screens.setPhone(next.placement, next.inset)
        : emu.setVideoInset(next.inset)
    // Refused: what Rust holds now (afterRefusal), unless something newer was sent meanwhile or a
    // new game started.
    report(
      call.catch((e: unknown) => {
        if (sent === next) ({ sent, refused } = afterRefusal(before, next, e))
        throw e
      }),
    )
  }

  /** The core runs `info`: a fresh session, then the auto quick save or the resume question. */
  async function begin(info: GameInfo, from: LibraryGame | null, resumeAuto = false) {
    // The new Rust session holds no placement: forget what was sent and what it came from, so a
    // late `setPlacement` of the old game's layer or editor sends nothing of the old game's.
    layerPlacement = []
    editorPlacement = null
    sent = { placement: [], inset: sent.inset }
    refused = null
    launches.value += 1
    playingGame.value = from
    game.value = info
    // `speed.spstart`: Rust started the game at it.
    speed.value = { ...info.speed }
    menuOpen.value = false
    paused.value = false
    undoLoadSlot.value = null
    // The game's own quick slot (#110), already current in Rust.
    comboSlot.value = info.quickSlot
    events.reset()
    muted.value = false
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

  /** The phone quick menu's «Rendi rapido»: `slot` becomes the game's quick slot (the one the
   * shortcuts and «Salva / Carica rapido» use), recorded for this game by Rust (#110), like the
   * slot steps (L3 + ← →, F2 / F3). */
  async function setQuickSlot(slot: number) {
    try {
      const playing = game.value?.id
      const now = await emu.controls.setSlot(slot)
      // Another game opened meanwhile: its own quick slot stays.
      if (game.value?.id === playing) comboSlot.value = now
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /** The phone quick menu's «Riavvolgi», held: the game runs backwards while held (a paused
   * session holds rewind still, so it runs meanwhile), then pauses again under the menu. Presses
   * and releases go to Rust in order, so a quick tap never leaves rewind on. */
  let rewinds = Promise.resolve()
  function holdRewind(on: boolean) {
    rewinds = rewinds.then(async () => {
      try {
        if (on) {
          await emu.setPaused(false)
          await emu.rewind(true)
        } else {
          await emu.rewind(false)
          if (menuOpen.value) await emu.setPaused(true)
        }
      } catch (e) {
        toasts.failWith(e)
      }
    })
    return rewinds
  }

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

  /** The quick menu or the touch controls keep `px` clear of the game: the canvas keeps clear
   * of it by itself, the native video is told the shares of the window (again on every change:
   * a turned phone sends its new edges, measured in the new window), with the phone's placement
   * when that changed too. */
  function setInset(px: InsetPx) {
    if (!sameInset(px, videoInset.value)) videoInset.value = px
    wantedShares = insetShares(px, window.innerWidth, window.innerHeight)
    queueView()
  }

  /** The FPS counter shown or hidden: F9 and L3 + X do it in Rust (`emu://hotkey` `fps`), the
   * search from the quick menu here. */
  function toggleFps() {
    state.showFps.value = !state.showFps.value
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
    requestClose: ending.requestClose,
    answerClose: ending.answerClose,
    crashed: ending.crashed,
    setMenu,
    toggleMenu,
    setSpeed,
    save,
    setQuickSlot,
    holdRewind,
    loadFromMenu: events.loadFromMenu,
    undoLoad: events.undoLoad,
    restart,
    setMuted,
    toggleFullscreen,
    setInset,
    setPlacement,
    launches,
    toggleFps,
    onHotkey: events.onHotkey,
    screenshot,
    onEmuEvent: (ev: EmuEvent) => {
      events.onEmuEvent(ev)
      // «Salva e chiudi» waits for its quick save (useGameEnding).
      ending.onSaveEvent(ev)
    },
    watchStats: ending.watchStats,
  }
}
export type GameSession = ReturnType<typeof useGameSession>
