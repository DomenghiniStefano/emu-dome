// «Giochi su Google Drive» (Trello #31): the list, read when the tab opens and again after each
// sync, and its three actions. «Carica» sends the game file to Drive, «Scarica» brings it into
// this device's library, «Rimuovi da Drive» moves Drive's copy to its trash (the saves stay).
// One action per game at a time; each answer replaces the game's row.
import { onMounted, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type CloudGame, type GameId } from '@/emu'
import { useToasts } from '@/composables/useToasts'
import type { GameAction } from './cloud'
import { applyGame, cloud, failCloud, loadGames } from './cloudState'

const DONE: Record<GameAction, string> = {
  upload: 'cloud.games.uploaded',
  download: 'cloud.games.downloaded',
  remove: 'cloud.games.removed',
}

export function useCloudGames() {
  const { t } = useI18n()
  const toasts = useToasts()
  /** The games with an action running. */
  const busy = reactive(new Set<GameId>())

  async function reload() {
    try {
      await loadGames()
    } catch (e) {
      failCloud(e)
    }
  }
  onMounted(reload)
  // A sync that ended (here or at a game's start / close) may have changed what is on Drive.
  watch(
    () => cloud.status?.lastSync,
    () => void reload(),
  )

  async function act(g: CloudGame, action: GameAction) {
    if (busy.has(g.id)) return
    busy.add(g.id)
    try {
      const run = {
        upload: emu.cloud.upload,
        download: emu.cloud.download,
        remove: emu.cloud.remove,
      }
      applyGame(await run[action](g.id))
      toasts.notify(t(DONE[action], { title: g.title }))
    } catch (e) {
      failCloud(e)
    } finally {
      busy.delete(g.id)
    }
  }

  return { busy, act, reload }
}
