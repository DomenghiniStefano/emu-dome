// L04 «Riprendi» / «Gioca»: where the game picks up (the newest quick save, else the in-game
// save), read again whenever the game is played.
import { ref, watch, type Ref } from 'vue'
import type { LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import { loadResumePoint } from './gameSaves'

export function useResumePoint(game: Ref<LibraryGame | null>) {
  const resume = ref<ResumePoint | null>(null)

  async function reload() {
    const g = game.value
    if (!g) return
    const point = await loadResumePoint(g)
    // Rule 7: the details moved to another game meanwhile: its own reload shows.
    if (game.value?.id === g.id) resume.value = point
  }
  watch(
    () => `${game.value?.id}:${game.value?.lastPlayed}`,
    () => void reload(),
  )

  return { resume, reload }
}
