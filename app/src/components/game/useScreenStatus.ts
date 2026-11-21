// The open game's screens as Rust has them (Trello #28): the layout shown, the cycle, swapped,
// the lid. Every change arrives on `emu://screens` (from the quick menu, the controller's L3 +
// Y / R3 / Select, a `screens.*` setting); the commands here only ask for one. Read by the quick
// menu (tools, G09) and the G15 «Coperchio chiuso» card.
import { onUnmounted, ref } from 'vue'
import { emu, type ScreenLayoutId, type ScreenStatus } from '@/emu'
import { latest } from '@/lib/latest'
import type { Toasts } from '@/composables/useToasts'

export function useScreenStatus(toasts: Pick<Toasts, 'failWith'>) {
  /** Null until the first answer. */
  const status = ref<ScreenStatus | null>(null)
  // An event is newer than any answer still on its way: it makes that answer stale (rule 7).
  const reads = latest()
  let stop: (() => void) | null = null

  async function load() {
    const fresh = reads.next()
    const now = await emu.screens.status()
    if (fresh()) status.value = now
  }

  /** Listens to the changes, then reads where things are now. */
  async function listen() {
    stop = await emu.screens.onChanged(s => {
      reads.next()
      status.value = s
    })
    await load()
  }
  onUnmounted(() => stop?.())

  /** Runs a screens command; the new status arrives as an event, a failure is toasted. */
  async function run(command: () => Promise<unknown>) {
    try {
      await command()
    } catch (e) {
      toasts.failWith(e)
    }
  }

  return {
    status,
    listen,
    /** «Scambia gli schermi»: a toggle. */
    swap: () => run(emu.screens.swap),
    /** «Chiudi il coperchio» / «Apri il coperchio». */
    toggleLid: () => run(emu.screens.toggleLid),
    /** G09: show `layout`. */
    setLayout: (layout: ScreenLayoutId) => run(() => emu.screens.setLayout(layout)),
  }
}
export type ScreenStatusState = ReturnType<typeof useScreenStatus>
