// The game library as the interface sees it (boards L01–L03, L11, E01): one reactive copy of
// `emu.library.get`, kept up to date by `library://changed` (the rows that changed, or the whole
// library), plus the scan progress from `library://scan`. App.vue starts it once
// (`startLibrary`); the screens read `library`.
//
// The games list is replaced as a whole on every change and never changed in place, so it is kept
// out of Vue's deep reactivity (`markRaw`): assigning a new list is what the screens react to, and
// a library of thousands of games is not wrapped in proxies object by object on each change.
import { computed, markRaw, reactive } from 'vue'
import {
  emu,
  type ConsoleId,
  type GameFamily,
  type GameId,
  type Library,
  type LibraryChanged,
  type LibraryFolder,
  type LibraryGame,
} from '@/emu'
import { errorText } from '@/i18n/labels'
import { familyIndex } from '@/lib/families'
import { latest } from '@/lib/latest'

/**
 * What L11 «N giochi aggiunti» shows after a scan that added games, or that skipped files of a
 * folder the user just picked (#127: an encrypted 3DS game, a file not recognised).
 */
export interface ScanSummary {
  added: number
  /** New games per console, counted from the ids that were not there before the scan. */
  perConsole: Partial<Record<ConsoleId, number>>
  /** Files with a game's extension the scan did not take, by reason. */
  skipped: { encrypted: number; unrecognised: number }
}

const NONE_SKIPPED: ScanSummary['skipped'] = { encrypted: 0, unrecognised: 0 }

/**
 * A folder the user picked is being scanned: its skipped files are said once it ends. The scans
 * EmuDome starts by itself (at start, a folder changing) find the same files again each time and
 * say nothing about them.
 */
let picked = false

export const library = reactive({
  /** False until the first `get` answered (or failed). */
  loaded: false,
  /** Replaced whole, never changed in place (not deeply reactive). */
  games: markRaw([]) as LibraryGame[],
  folders: [] as LibraryFolder[],
  /** The regional versions grouped by Rust (#102), replaced whole like `games`. */
  families: markRaw([]) as GameFamily[],
  scanning: false,
  /** Latest `progress` of the running scan. */
  progress: null as { checked: number; found: number } | null,
  summary: null as ScanSummary | null,
  /** Text of the latest failure of `get`, already translated. */
  error: null as string | null,
})

/** The families indexed (lib/families.ts): rebuilt only when `library.families` is replaced. */
export const libraryFamilies = computed(() => familyIndex(library.families))

/**
 * What a library folder (`path`, as `library.folders` holds it) shows: the name Rust sends (the
 * path, or a picked Android folder's display name), else `unnamed` (a picked folder whose name
 * is not known, or a folder the library has not listed yet). Never the stored `content://` URI.
 */
export function folderLabel(folders: LibraryFolder[], path: string, unnamed: string): string {
  return folders.find(f => f.path === path)?.name ?? unnamed
}

let before: Set<GameId> | null = null

// Rule 7: only the newest answer counts; a change event is newer than any `get` still on its way.
const reloads = latest()

function apply(l: Library) {
  library.games = markRaw(l.games)
  library.families = markRaw(l.families)
  library.folders = l.folders
  library.scanning = l.scanning
  library.error = null
  library.loaded = true
}

export async function reloadLibrary() {
  const fresh = reloads.next()
  try {
    const l = await emu.library.get()
    if (!fresh()) return
    apply(l)
  } catch (e) {
    if (!fresh()) return
    library.error = errorText(e)
    library.loaded = true
  }
}

/**
 * `library://changed`: the whole library replaces ours, the families replace ours, or each row
 * sent replaces the row with its id (a new list either way; ids the list does not show are left
 * out).
 */
export function applyLibraryChange(change: LibraryChanged) {
  // Any `get` still answering is older than this.
  reloads.next()
  if (change.kind === 'all') return apply(change.library)
  if (change.kind === 'families') {
    library.families = markRaw(change.families)
    return
  }
  const rows = new Map(change.games.map(g => [g.id, g]))
  library.games = markRaw(library.games.map(g => rows.get(g.id) ?? g))
}

async function scanDone(added: number, skippedNow: ScanSummary['skipped']) {
  await reloadLibrary()
  library.scanning = false
  library.progress = null
  const skipped = picked ? skippedNow : NONE_SKIPPED
  picked = false
  if (added <= 0 && skipped.encrypted + skipped.unrecognised === 0) {
    before = null
    return
  }
  const perConsole: ScanSummary['perConsole'] = {}
  if (added > 0)
    for (const g of library.games) {
      if (before && before.has(g.id)) continue
      perConsole[g.console] = (perConsole[g.console] ?? 0) + 1
    }
  before = null
  library.summary = mergeSummary(library.summary, { added, perConsole, skipped })
}

/**
 * L11 counts every scan since it was last closed (#127): the first run scans each folder added
 * (and each file picked) on its own, so the banner it leaves says all the games they added, not
 * the last scan's. Skipped files are counted again by every scan (each scan walks every folder),
 * so the newest count of them is the one said.
 */
export function mergeSummary(shown: ScanSummary | null, next: ScanSummary): ScanSummary {
  if (!shown) return next
  const perConsole: ScanSummary['perConsole'] = { ...shown.perConsole }
  for (const [c, n] of Object.entries(next.perConsole) as [ConsoleId, number][])
    perConsole[c] = (perConsole[c] ?? 0) + n
  const skipped =
    next.skipped.encrypted + next.skipped.unrecognised > 0 ? next.skipped : shown.skipped
  return { added: shown.added + next.added, perConsole, skipped }
}

/**
 * Subscribes to the library events and loads it. Resolves one unlistener for both; if one
 * subscription fails, the other is stopped and the failure thrown.
 */
export async function startLibrary(): Promise<() => void> {
  const subscriptions = [
    emu.library.onChanged(applyLibraryChange),
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
          void scanDone(e.added, {
            encrypted: e.skippedEncrypted,
            unrecognised: e.skippedUnrecognised,
          })
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
  // Set before: the scan of a small folder can end before the picker's answer arrives.
  picked = true
  try {
    if ((await emu.library.chooseFolder()) === null) picked = false
    return null
  } catch (e) {
    picked = false
    return errorText(e)
  }
}

/** E01 «Scegli un'altra cartella»: a new folder replaces the unreachable one. */
export async function replaceFolder(old: string): Promise<string | null> {
  picked = true
  try {
    const chosen = await emu.library.chooseFolder()
    if (chosen === null) picked = false
    if (chosen !== null && chosen !== old) await emu.library.removeFolder(old)
    return null
  } catch (e) {
    picked = false
    return errorText(e)
  }
}
