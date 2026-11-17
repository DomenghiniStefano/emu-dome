// The game library as the interface sees it (boards L01–L03, L11, E01): one reactive copy of
// `emu.library.get`, refreshed on `library://changed`, plus the scan progress from
// `library://scan`. App.vue starts it once (`startLibrary`); the screens read `library`.
import { reactive } from 'vue'
import { emu, type ConsoleId, type GameId, type LibraryFolder, type LibraryGame } from './emu'
import { errorText } from './i18n/labels'

/** What L11 «N giochi aggiunti» shows after a scan that added games. */
export interface ScanSummary {
  added: number
  /** New games per console, counted from the ids that were not there before the scan. */
  perConsole: Partial<Record<ConsoleId, number>>
}

export const library = reactive({
  /** False until the first `get` answered (or failed). */
  loaded: false,
  games: [] as LibraryGame[],
  folders: [] as LibraryFolder[],
  scanning: false,
  /** Latest `progress` of the running scan. */
  progress: null as { checked: number; found: number } | null,
  summary: null as ScanSummary | null,
  /** Text of the latest failure of `get`, already translated. */
  error: null as string | null,
})

let before: Set<GameId> | null = null

export async function reloadLibrary() {
  try {
    const l = await emu.library.get()
    library.games = l.games
    library.folders = l.folders
    library.scanning = l.scanning
    library.error = null
  } catch (e) {
    library.error = errorText(e)
  }
  library.loaded = true
}

async function scanDone(added: number) {
  await reloadLibrary()
  library.scanning = false
  library.progress = null
  if (added <= 0) {
    before = null
    return
  }
  const perConsole: ScanSummary['perConsole'] = {}
  for (const g of library.games) {
    if (before && before.has(g.id)) continue
    perConsole[g.console] = (perConsole[g.console] ?? 0) + 1
  }
  before = null
  library.summary = { added, perConsole }
}

/** Subscribes to the library events and loads it. Returns the unlisteners. */
export async function startLibrary(): Promise<Array<() => void>> {
  const unlisten = [
    await emu.library.onChanged(() => void reloadLibrary()),
    await emu.library.onScan(e => {
      switch (e.phase) {
        case 'started':
          before = new Set(library.games.map(g => g.id))
          library.scanning = true
          library.progress = null
          return
        case 'progress':
          library.scanning = true
          library.progress = { checked: e.checked, found: e.found }
          return
        case 'done':
          void scanDone(e.added)
      }
    }),
  ]
  await reloadLibrary()
  return unlisten
}

/** «Scegli una cartella…»: the backend adds the folder and scans it. Errors come back as text. */
export async function chooseFolder(): Promise<string | null> {
  try {
    await emu.library.chooseFolder()
    return null
  } catch (e) {
    return errorText(e)
  }
}

/** E01 «Scegli un'altra cartella»: a new folder replaces the unreachable one. */
export async function replaceFolder(old: string): Promise<string | null> {
  try {
    const chosen = await emu.library.chooseFolder()
    if (chosen !== null && chosen !== old) await emu.library.removeFolder(old)
    return null
  } catch (e) {
    return errorText(e)
  }
}
