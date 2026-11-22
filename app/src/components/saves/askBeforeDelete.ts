// `general.confirmdel` («Chiedi prima di eliminare»): whether deleting a save asks first (X04).
// Read when the user deletes, so it is always the current value; one that cannot be read asks,
// the side that loses nothing.
import type { SettingRow } from '@/emu'
import { readSetting } from '@/lib/settingsModel'

type ReadRow = (key: 'general.confirmdel') => Promise<SettingRow | undefined>

/** True unless the setting is off. */
export async function askBeforeDelete(read: ReadRow = readSetting): Promise<boolean> {
  try {
    return (await read('general.confirmdel'))?.value !== false
  } catch {
    // Deliberate: an unreadable setting falls back to asking, the safe default, and the
    // delete itself still reports its own failure.
    return true
  }
}
