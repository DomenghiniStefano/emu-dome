// First run S06: the optional imports of saves (DeSmuME's folder, any other folder). A row picks
// a folder, then imports the saves that match library games.
import { ref } from 'vue'
import { emu, type FolderImport } from '../emu'
import type { ErrorSink } from '../i18n/labels'
import type { ImportState } from '../lib/first-run/imports'
import { library } from '../library'

export function useFirstRunImports(onError: ErrorSink) {
  const imports = ref<Record<FolderImport, ImportState>>({
    desmume: { found: null, imported: null },
    any: { found: null, imported: null },
  })
  const busy = ref(false)

  async function work(job: () => Promise<void>) {
    busy.value = true
    try {
      await job()
    } catch (e) {
      onError(e)
    } finally {
      busy.value = false
    }
  }

  /** Looks at a folder the user picks: how many saves, and which match library games. */
  const pickFolder = (what: FolderImport) =>
    work(async () => {
      const found = await emu.saves.importFolder(
        what,
        library.games.map(g => g.path),
      )
      if (found) imports.value = { ...imports.value, [what]: { found, imported: null } }
    })

  /** Imports the matching saves the folder offered. */
  const importAll = (what: FolderImport) => {
    const found = imports.value[what].found
    if (!found) return Promise.resolve()
    return work(async () => {
      const n = await emu.saves.importMany(found.matches)
      imports.value = { ...imports.value, [what]: { found, imported: n } }
    })
  }

  return { imports, busy, pickFolder, importAll }
}
