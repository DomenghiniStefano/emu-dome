// Opening a game (a file, a library game, the one given at startup) through the game flow:
// G01 while the core loads, G02 if it cannot start, E04 if its in-game save is unreadable, the
// resume question, and starting again after E11.
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { AUTO_SLOT, emu, type LibraryGame } from '@/emu'
import { flow } from './gameFlow'
import { errorText } from '@/i18n/labels'
import { launchFailure, openedPath, type Launch } from './launch'
import { fileTitle } from './session'
import type { GameSession } from './useGameSession'
import type { Toasts } from '@/composables/useToasts'

export function useGameLaunch(session: GameSession, toasts: Toasts) {
  const { t } = useI18n()

  // The last game opened, to open it again after the emulator stopped (E11).
  let last: Launch | null = null
  // E04: the game whose in-game save was set aside, to open again after restoring a backup.
  let broken: Launch | null = null

  async function start(launch: Launch, resumeAuto = false): Promise<boolean> {
    const { opened, title, colour } = launch
    flow.launching = { title, colour }
    try {
      const info =
        opened.kind === 'path'
          ? await emu.openGame(opened.path)
          : await emu.library.open(opened.game.id)
      last = launch
      await session.begin(info, opened.kind === 'library' ? opened.game : null, resumeAuto)
      return true
    } catch (e) {
      const failure = launchFailure(e)
      if (failure.kind === 'saveBroken') {
        broken = launch
        flow.saveBroken = { title, backupAt: failure.backupAt }
      } else {
        flow.startError = { title, reason: failure.reason }
      }
      return false
    } finally {
      flow.launching = null
    }
  }

  /** A file: the one given, else the one the user picks. */
  async function openFile(given?: string) {
    const path =
      given ??
      (await open({
        multiple: false,
        filters: [{ name: t('library.dialogFilter'), extensions: ['gba', 'gb', 'gbc'] }],
      }))
    if (typeof path !== 'string') return
    await start({ opened: { kind: 'path', path }, title: fileTitle(path), colour: null })
  }

  /**
   * A game from the library (L01). With `slot` (V03 «Apri il gioco e carica») that quick save is
   * loaded once the game runs.
   */
  async function openLibraryGame(g: LibraryGame, slot?: number) {
    const started = await start({
      opened: { kind: 'library', game: g },
      title: g.title,
      colour: g.colour,
    })
    if (!started || slot === undefined) return
    // The quick save asked for wins over the resume question.
    flow.resumeAsk = false
    await emu.setPaused(false)
    await emu.loadState(slot)
  }

  /** The game given on the command line, if any. */
  async function openStartupGame() {
    const path = await emu.startupGame()
    if (path) await openFile(path)
  }

  async function answerResume(resume: boolean) {
    flow.resumeAsk = false
    if (resume) await emu.loadState(AUTO_SLOT)
    await emu.setPaused(false)
  }

  // ---- E04: the in-game save cannot be read ----
  function takeBroken(): Launch | null {
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
      await emu.saves.restoreLatest(openedPath(b.opened))
    } catch (e) {
      flow.startError = { title: b.title, reason: errorText(e) }
      return
    }
    await start(b)
  }
  /** «Gioca senza salvataggio»: the unreadable save is already set aside, so the game opens
   * without one. */
  async function playWithoutSave() {
    const b = takeBroken()
    if (b) await start(b)
  }
  const closeBroken = () => void takeBroken()

  // ---- E11: the emulator stopped ----
  async function crashToLibrary() {
    flow.crashed = null
    await emu.abandonGame()
    session.end()
  }
  /** «Riprendi dal salvataggio automatico»: the same game again, from its auto quick save. */
  async function crashResume() {
    const again = last
    const title = session.game.value?.title ?? ''
    flow.crashed = null
    await emu.abandonGame()
    session.end()
    if (again) await start({ ...again, title }, true)
  }
  async function crashCopy() {
    try {
      await navigator.clipboard.writeText(flow.crashed?.detail ?? '')
      toasts.notify(t('flow.detailsCopied'))
    } catch (e) {
      toasts.fail(String(e))
    }
  }

  return {
    openFile,
    openLibraryGame,
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
