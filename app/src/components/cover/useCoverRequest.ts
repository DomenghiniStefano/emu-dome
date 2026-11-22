// «Usa come copertina» on the screenshot toast (Trello #85): the request is kept here until the
// interface can show «Adatta la copertina» without covering the game being played: at once with
// the quick menu open, else the next time the quick menu opens or once the game has closed
// (App.vue decides when, CoverCropDialog shows it). One request at a time: a newer one replaces
// it. Created by App.vue and handed to the game session, never a module-level ref.
import { ref } from 'vue'
import type { GameId } from '@/emu'

export interface CoverRequest {
  game: GameId
  /** The screenshot's file name in the game's folder (`library_screenshots`). */
  name: string
}

export function createCoverRequest() {
  const pending = ref<CoverRequest | null>(null)
  return {
    pending,
    ask: (r: CoverRequest) => (pending.value = r),
    clear: () => (pending.value = null),
  }
}
export type CoverRequests = ReturnType<typeof createCoverRequest>

/** The screenshot's file name from the path `screenshotSaved` carries. */
export function screenshotName(path: string): string {
  return path.split(/[\\/]/).pop() ?? ''
}
