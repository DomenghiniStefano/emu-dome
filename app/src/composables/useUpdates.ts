// EmuDome's own update (Trello #44), shared by the settings (`general.updates` locked in a build
// that cannot update itself, `about.upd`'s line and «Controlla ora») and X06, the shell's card.
// One status for the whole app, kept in step with Rust (`updates://changed` sends it after every
// change) while at least one component uses it. A failure reaches the user (useErrorSink); an
// answer older than the latest event is dropped (rule 7).
import { onMounted, onScopeDispose, ref } from 'vue'
import { emu, type UpdateStatus } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { latest } from '@/lib/latest'
import { listenerSet, type ListenerSet } from '@/composables/useListeners'
import { updateUnavailable } from '@/lib/updates'

const status = ref<UpdateStatus | null>(null)
const loads = latest()
let users = 0
let listeners: ListenerSet | null = null

function apply(s: UpdateStatus) {
  status.value = s
  updateUnavailable.value = s.state.state === 'unavailable'
}

export function useUpdates() {
  const failWith = useErrorSink()

  onMounted(async () => {
    users++
    if (users > 1) return
    listeners = listenerSet()
    try {
      await listeners.add(
        emu.updates.onChanged(s => {
          // Newer than any status still on its way.
          loads.next()
          apply(s)
        }),
      )
      const fresh = loads.next()
      const s = await emu.updates.status()
      if (fresh()) apply(s)
    } catch (e) {
      failWith(e)
    }
  })
  onScopeDispose(() => {
    users = Math.max(0, users - 1)
    if (users === 0) {
      listeners?.stop()
      listeners = null
    }
  })

  /** Runs one of the updater's commands; its answer is the newest status unless an event came. */
  async function run(call: () => Promise<UpdateStatus>) {
    const fresh = loads.next()
    try {
      const s = await call()
      if (fresh()) apply(s)
    } catch (e) {
      failWith(e)
    }
  }

  return {
    status,
    /** «Controlla ora». */
    check: () => run(emu.updates.check),
    /** X06 «Più tardi». */
    later: () => run(emu.updates.later),
    /** X06 «Riavvia e aggiorna». */
    install: () => run(emu.updates.install),
  }
}
