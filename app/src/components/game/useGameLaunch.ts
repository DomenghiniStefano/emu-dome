// Opening a game (a file, a library game, the one given at startup) through the game flow:
// G01 while the core loads, G02 if it cannot start, E04 if its in-game save is unreadable, the
// resume question, and starting again after E11.
import { useI18n } from 'vue-i18n'
import { AUTO_SLOT, emu, type GameId, type GameInfo, type LibraryGame } from '@/emu'
import { clearToLaunch } from '@/components/cloud/cloudState'
import { flow } from './gameFlow'
import { launchFailure, startProblem, type Launch } from './launch'
import type { GameSession } from './useGameSession'
import type { Toasts } from '@/composables/useToasts'

export function useGameLaunch(session: GameSession, toasts: Toasts) {
  const { t } = useI18n()

  // The last game opened, to open it again after the emulator stopped (E11).
  let last: Launch | null = null
  // E04: the game whose in-game save was set aside (its key, from Rust), to open again after
  // restoring a backup.
  let broken: { launch: Launch; game: GameId } | null = null

  // One launch at a time: a second one (a double click, the startup game still opening) is
  // dropped rather than racing the first to the core.
  let starting = false

  /** Opens the game; resolves whether it runs. `before`: a question first (G05), inside the one
   * launch at a time; false opens nothing. */
  async function start(
    launch: Launch,
    resumeAuto = false,
    before?: () => Promise<boolean>,
  ): Promise<boolean> {
    if (starting) return false
    starting = true
    try {
      if (before && !(await before())) return false
      const info = await openCore(launch)
      if (!info) return false
      last = launch
      const { opened } = launch
      // The game runs from here: what fails now (the auto quick save, the pause for the resume
      // question) is told over it, never as G02 «could not start».
      try {
        await session.begin(info, opened.kind === 'library' ? opened.game : null, resumeAuto)
      } catch (e) {
        toasts.failWith(e)
      }
      return true
    } finally {
      starting = false
    }
  }

  /** The core loads the game under G01; a failure is G02, or E04 for an unreadable save. */
  async function openCore(launch: Launch): Promise<GameInfo | null> {
    const { opened, title, colour } = launch
    flow.launching = { title, colour }
    try {
      return opened.kind === 'path'
        ? await emu.openGame(opened.path)
        : await emu.library.open(opened.game.id)
    } catch (e) {
      const failure = launchFailure(e)
      if (failure.kind === 'saveBroken') {
        broken = { launch, game: failure.game }
        flow.saveBroken = { title, backupAt: failure.backupAt }
      } else {
        flow.startError = { title, ...failure.problem }
      }
      return null
    } finally {
      flow.launching = null
    }
  }

  /** A file: the one given, else the one the user picks among the extensions Rust opens. */
  async function openFile(given?: string) {
    const path = given ?? (await pickFile())
    if (typeof path !== 'string') return
    await start({ opened: { kind: 'path', path }, title: await titleOf(path), colour: null })
  }

  /** Rust picks (the desktop's dialog, Android's document picker) and checks the extension. */
  async function pickFile(): Promise<string | null> {
    try {
      return await emu.pickGameFile(t('library.dialogFilter'))
    } catch (e) {
      toasts.failWith(e)
      return null
    }
  }

  /** The file's title from Rust (a document's display name, never its URI). */
  async function titleOf(path: string): Promise<string> {
    try {
      return (await emu.gameFileTitle(path)) ?? t('flow.untitledGame')
    } catch (e) {
      // Only the title of G01 / G02: the open itself still runs and reports its own error.
      console.warn('game file title', e)
      return t('flow.untitledGame')
    }
  }

  /**
   * A game from the library (L01). With `slot` (V03 «Apri il gioco e carica») that quick save is
   * loaded once the game runs.
   */
  async function openLibraryGame(g: LibraryGame, slot?: number) {
    const started = await start(
      { opened: { kind: 'library', game: g }, title: g.title, colour: g.colour },
      false,
      // G05 (#31): open on another device → asked first; «Annulla» opens nothing.
      () => clearToLaunch(g),
    )
    if (!started || slot === undefined) return
    // The quick save asked for wins over the resume question.
    flow.resumeAsk = false
    try {
      await emu.setPaused(false)
      await emu.loadState(slot)
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /**
   * Another library game while one is open (the search from the quick menu, #32): the open one
   * closes as «Chiudi il gioco» does (G14, the close flow #86 with its «Wait» / «Close anyway»),
   * then this one opens.
   */
  async function switchToLibraryGame(g: LibraryGame) {
    await session.close()
    await openLibraryGame(g)
  }

  /** The game given on the command line, if any; else, with `general.start` = «last game», the
   * library game played last (Rust decides, and answers none when a game was given). */
  async function openStartupGame() {
    const path = await emu.startupGame()
    if (path) return openFile(path)
    const last = await emu.startupLastGame()
    if (last) await openLibraryGame(last)
  }

  async function answerResume(resume: boolean) {
    flow.resumeAsk = false
    try {
      if (resume) await emu.loadState(AUTO_SLOT)
    } catch (e) {
      toasts.failWith(e)
    }
    // Played on even when the auto quick save could not be loaded: the game is open.
    try {
      await emu.setPaused(false)
    } catch (e) {
      toasts.failWith(e)
    }
  }

  // ---- E04: the in-game save cannot be read ----
  function takeBroken(): { launch: Launch; game: GameId } | null {
    const b = broken
    flow.saveBroken = null
    broken = null
    return b
  }
  /** «Usa il backup di …»: the newest backup becomes the in-game save, then the game opens. */
  async function useLatestBackup() {
    const b = takeBroken()
    if (!b) return
    try {
      await emu.saves.restoreLatest(b.game)
    } catch (e) {
      flow.startError = { title: b.launch.title, ...startProblem(e) }
      return
    }
    await start(b.launch)
  }
  /** «Gioca senza salvataggio»: the unreadable save is already set aside, so the game opens
   * without one. */
  async function playWithoutSave() {
    const b = takeBroken()
    if (b) await start(b.launch)
  }
  const closeBroken = () => void takeBroken()

  // ---- E11: the emulator stopped ----
  /** Lets go of the stopped core and leaves the game, whether or not Rust could let go: the
   * user is never kept on the crash dialog. */
  async function leaveCrashed() {
    flow.crashed = null
    try {
      await emu.abandonGame()
    } catch (e) {
      toasts.failWith(e)
    } finally {
      session.end()
    }
  }
  const crashToLibrary = () => leaveCrashed()
  /** «Riprendi dal salvataggio automatico»: the same game again, from its auto quick save. */
  async function crashResume() {
    const again = last
    const title = session.game.value?.title ?? ''
    await leaveCrashed()
    if (again) await start({ ...again, title }, true)
  }
  async function crashCopy() {
    try {
      await navigator.clipboard.writeText(flow.crashed?.detail ?? '')
      toasts.notify(t('flow.detailsCopied'))
    } catch (e) {
      toasts.failWith(e)
    }
  }

  return {
    openFile,
    openLibraryGame,
    switchToLibraryGame,
    openStartupGame,
    answerResume,
    useLatestBackup,
    playWithoutSave,
    closeBroken,
    crashToLibrary,
    crashResume,
    crashCopy,
  }
}
