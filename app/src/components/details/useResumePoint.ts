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
    resume.value = await loadResumePoint(g)
  }
  watch(
    () => `${game.value?.path}:${game.value?.lastPlayed}`,
    () => void reload(),
  )

  return { resume, reload }
}
