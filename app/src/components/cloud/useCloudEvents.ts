// The sync's events, followed for the app's whole life (App.vue, Trello #31): the status, the
// transfers (G03 / G14), the conflicts (G04), and a game opened while another device has it open
// without G05 first (`cloud://in-use`: at startup, «Open with»). Stopped on unmount.
import { emu, type CloudInUseEvent } from '@/emu'
import { reportError } from '@/lib/errors'
import type { GameSession } from '@/components/game/useGameSession'
import { useListeners } from '@/composables/useListeners'
import {
  applyProgress,
  applyStatus,
  askConflict,
  askInUse,
  gameTitle,
  inUseToAsk,
  loadStatus,
} from './cloudState'

export function useCloudEvents(session: GameSession) {
  const listeners = useListeners()

  /**
   * G05 after the open: the game is paused under the quick menu while the question shows;
   * «Gioca comunque» plays on, «Annulla» closes it (its saves are written as on any close).
   */
  async function inUseOpened(e: CloudInUseEvent) {
    if (e.inUse.kind !== 'elsewhere' || !inUseToAsk(e.id)) return
    const open = () => session.game.value?.id === e.id
    const title = session.game.value?.title ?? gameTitle(e.id)
    try {
      if (open()) await session.setMenu(true)
    } catch (err) {
      reportError(err)
    }
    const play = await askInUse({
      id: e.id,
      title,
      device: e.inUse.name,
      since: e.inUse.since,
      opened: true,
    })
    if (!open()) return
    try {
      if (play) await session.setMenu(false)
      else await session.close()
    } catch (err) {
      reportError(err)
    }
  }

  /** Subscribes, then reads the status (it may have changed before the listener). */
  async function listen() {
    await listeners.add(
      emu.cloud.onStatus(applyStatus),
      emu.cloud.onProgress(applyProgress),
      emu.cloud.onConflict(askConflict),
      emu.cloud.onInUse(e => void inUseOpened(e)),
    )
    await loadStatus()
  }

  return { listen }
}
