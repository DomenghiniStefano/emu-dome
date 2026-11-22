// What the status card does (Trello #31): «Sincronizza ora» (every game's saves both ways, the
// settings, the library list, the space), and «Scegli la versione di …», which reads the game's
// two versions again and asks G04 (CloudConflictDialog).
import { emu, type GameId } from '@/emu'
import { applyStatus, askConflict, failCloud } from './cloudState'

export function useCloudSync() {
  async function syncNow() {
    try {
      applyStatus(await emu.cloud.syncNow())
    } catch (e) {
      failCloud(e)
    }
  }

  async function choose(id: GameId) {
    try {
      const c = await emu.cloud.conflict(id)
      // Null: already resolved (on another device, or by a sync meanwhile); the status follows.
      if (c) askConflict(c)
    } catch (e) {
      failCloud(e)
    }
  }

  return { syncNow, choose }
}
