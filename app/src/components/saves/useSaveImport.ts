// V05, importing a save: the file picked (and recognised), the game it goes to (found from the
// file name unless the game was chosen first, as from L05 «Importa…»), that game's saves (to warn
// about the in-game save it replaces), the «Scegli il gioco» dialog, and the import itself.
import { ref, type Ref } from 'vue'
import { emu, type GameSaves, type ImportCandidate, type LibraryGame } from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { library } from '@/lib/library'
import { loadGameSaves } from '@/components/details/gameSaves'
import { matchImportTarget } from './savesModel'

export function useSaveImport(
  slotCount: Ref<number>,
  fail: ErrorSink,
  /** The save went in: show that game. */
  imported: (game: LibraryGame) => Promise<void>,
) {
  const candidate = ref<ImportCandidate | null>(null)
  const target = ref<LibraryGame | null>(null)
  /** The game was chosen before the file (L05 «Importa…»): the file name does not change it. */
  const targetChosen = ref(false)
  const targetFound = ref(false)
  const targetSaves = ref<GameSaves | null>(null)
  const choosingGame = ref(false)
  const importing = ref(false)

  /** A new import, nothing chosen. */
  function reset() {
    candidate.value = null
    target.value = null
    targetChosen.value = false
    targetSaves.value = null
  }

  async function setTarget(g: LibraryGame) {
    target.value = g
    try {
      targetSaves.value = await loadGameSaves(g, slotCount.value)
    } catch (e) {
      fail(e)
    }
  }

  /** L05 «Importa…»: the import for that game (ROM path). */
  async function resetFor(path: string) {
    reset()
    const g = library.games.find(x => x.path === path)
    if (!g) return
    targetChosen.value = true
    await setTarget(g)
  }

  async function pickFile() {
    try {
      const c = await emu.saves.importPick()
      if (!c) return
      candidate.value = c
      if (targetChosen.value) return
      const match = matchImportTarget(c.fileName, library.games)
      targetFound.value = match !== null
      if (match) await setTarget(match)
    } catch (e) {
      fail(e)
    }
  }

  async function chooseTarget(g: LibraryGame) {
    choosingGame.value = false
    targetFound.value = false
    targetChosen.value = true
    await setTarget(g)
  }

  async function run() {
    const c = candidate.value
    const g = target.value
    if (!c || !g || importing.value) return
    importing.value = true
    try {
      await emu.saves.import(c.path, g.path)
      await imported(g)
    } catch (e) {
      fail(e)
    } finally {
      importing.value = false
    }
  }

  return {
    candidate,
    target,
    targetFound,
    targetSaves,
    choosingGame,
    importing,
    reset,
    resetFor,
    pickFile,
    chooseTarget,
    run,
  }
}
