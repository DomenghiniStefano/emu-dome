// «Scarica e gioca» for a game only on Google Drive (Trello #31): the file comes down into this
// device's library (`download`, its progress on `cloud://progress`), then the game opens as soon
// as the library lists it (the backend adds it and scans). A library that does not list it in
// time says so instead of opening: the game is there, the next look at the library shows it.
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type CloudGame, type GameId, type LibraryGame } from '@/emu'
import { useToasts } from '@/composables/useToasts'
import { library, reloadLibrary } from '@/lib/library'
import { applyGame, failCloud } from './cloudState'

/** How long the library may take to list a downloaded game before the game is not opened. */
export const LIBRARY_WAIT_MS = 20_000

/**
 * The library's row of `id` once it lists it with its file here (a game removed from this device
 * stays listed, unavailable, until the download's scan finds it again, #29); null after `ms`.
 */
export function libraryRow(id: GameId, ms = LIBRARY_WAIT_MS): Promise<LibraryGame | null> {
  const here = (g: LibraryGame) => g.id === id && g.available
  const now = library.games.find(here)
  if (now) return Promise.resolve(now)
  return new Promise(resolve => {
    const timer = window.setTimeout(() => {
      stop()
      resolve(null)
    }, ms)
    const stop = watch(
      () => library.games,
      games => {
        const row = games.find(here)
        if (!row) return
        window.clearTimeout(timer)
        stop()
        resolve(row)
      },
    )
  })
}

export function useDriveDownload(play: (g: LibraryGame) => void) {
  const { t } = useI18n()
  const toasts = useToasts()
  /** The game coming down now. */
  const downloading = ref<GameId | null>(null)

  async function downloadAndPlay(g: CloudGame) {
    if (downloading.value) return
    downloading.value = g.id
    try {
      applyGame(await emu.cloud.download(g.id))
      void reloadLibrary()
      const row = await libraryRow(g.id)
      if (row) play(row)
      else toasts.notify(t('cloud.drive.waiting', { title: g.title }))
    } catch (e) {
      failCloud(e)
    } finally {
      downloading.value = null
    }
  }

  return { downloading, downloadAndPlay }
}
