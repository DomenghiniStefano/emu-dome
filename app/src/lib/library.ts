// The game library as the interface sees it (boards L01–L03, L11, E01): one reactive copy of
// `emu.library.get`, refreshed on `library://changed`, plus the scan progress from
// `library://scan`. App.vue starts it once (`startLibrary`); the screens read `library`.
import { reactive } from 'vue'
import { emu, type ConsoleId, type GameId, type LibraryFolder, type LibraryGame } from '@/emu'
import { errorText } from '@/i18n/labels'
import { latest } from '@/lib/latest'

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

// Rule 7: `library://changed` comes in bursts (a scan, play time); only the newest answer counts.
const reloads = latest()

export async function reloadLibrary() {
  const fresh = reloads.next()
  try {
    const l = await emu.library.get()
    if (!fresh()) return
    library.games = l.games
    library.folders = l.folders
    library.scanning = l.scanning
    library.error = null
  } catch (e) {
    if (!fresh()) return
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

/**
 * Subscribes to the library events and loads it. Resolves one unlistener for both; if one
 * subscription fails, the other is stopped and the failure thrown.
 */
export async function startLibrary(): Promise<() => void> {
  const subscriptions = [
    emu.library.onChanged(() => void reloadLibrary()),
    emu.library.onScan(e => {
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
  const settled = await Promise.allSettled(subscriptions)
  const unlisten = settled.flatMap(r => (r.status === 'fulfilled' ? [r.value] : []))
  const stop = () => unlisten.forEach(off => off())
  const failed = settled.find(r => r.status === 'rejected')
  if (failed) {
    stop()
    throw failed.reason
  }
  await reloadLibrary()
  return stop
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
