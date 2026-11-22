// E13 «Telefono quasi pieno» (Trello #29): the room left where the saves are written, as Rust
// reads it (`storage_status`: free bytes, the disk's size, and whether that is low: under
// 500 MB). Asked when the component using it mounts (the shell: at startup, and again when it
// shows after a game closed) and whenever the library changes (a game added or downloaded).
// The shell mounts its user on the phone only. An older answer than the latest request is dropped
// (rule 7); a failed read shows no warning, since nothing the user can do follows from it.
import { onMounted, ref } from 'vue'
import { emu, type StorageStatus } from '@/emu'
import { useListeners } from '@/composables/useListeners'
import { ignore } from '@/lib/errors'
import { latest } from '@/lib/latest'

export function useStorageStatus() {
  const status = ref<StorageStatus | null>(null)
  const loads = latest()

  function refresh() {
    const fresh = loads.next()
    ignore(
      emu.storage.status().then(s => {
        if (fresh()) status.value = s
      }),
      'E13: the room left was not read; no warning',
    )
  }

  const listeners = useListeners()
  onMounted(() => {
    refresh()
    ignore(
      listeners.add(emu.library.onChanged(refresh)),
      'E13: library changes not followed; the room is read again when the shell shows',
    )
  })

  return { status, refresh }
}
