// S06's optional imports (DeSmuME, another folder): what a row says and does at each moment.
import type { FolderSaves } from '@/emu'

/** A row's state: nothing picked yet, a folder looked at, or its saves imported. */
export interface ImportState {
  found: FolderSaves | null
  /** Saves imported; null: not imported yet. */
  imported: number | null
}

/** The row's line, as message keys and their values: the hint until a folder is picked, then
 * what was found there (and how much matches the library), then how many were imported. */
export type ImportLine =
  | { kind: 'hint' }
  | { kind: 'imported'; n: number }
  | { kind: 'found'; n: number; folder: string; matched: number | null }

export function importLine(state: ImportState): ImportLine {
  if (state.imported !== null) return { kind: 'imported', n: state.imported }
  const found = state.found
  if (!found) return { kind: 'hint' }
  return {
    kind: 'found',
    n: found.found,
    folder: found.folder,
    matched: found.found === 0 ? null : found.matches.length,
  }
}

/** The row's button imports once a folder offered saves for library games, else picks one. */
export const canImport = (state: ImportState) =>
  state.imported === null && (state.found?.matches.length ?? 0) > 0
