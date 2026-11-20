// What the `bios.*` rows show (built-in replacement, your file verified, not recognised), from
// the backend's check of the files in place. One list for every row, refreshed when a row's
// value changes (a file chosen or the row reset).
import { ref } from 'vue'
import { emu, type BiosStatus } from '@/emu'

const statuses = ref<Partial<Record<string, BiosStatus>>>({})

/** Reads every row's status again. */
export async function refreshBiosStatus(): Promise<void> {
  const rows = await emu.bios.status()
  statuses.value = Object.fromEntries(rows.map(r => [r.key, r.status]))
}

/** The status of a `bios.*` row; `builtin` until known or for a row without a backend. */
export function biosStatus(key: string): BiosStatus {
  return statuses.value[key] ?? 'builtin'
}
