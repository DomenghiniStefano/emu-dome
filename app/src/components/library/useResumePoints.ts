// Where «Riprendi» picks up for each «Continua a giocare» game: the newest quick save (auto or
// a slot), else the in-game save. Read again whenever the games played last change.
import { ref, watch, type Ref } from 'vue'
import type { LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import {
  DEFAULT_SLOT_COUNT,
  NOTHING_TO_RESUME,
  loadResumePoint,
  loadSlotCount,
} from '@/components/details/gameSaves'

export function useResumePoints(recent: Ref<LibraryGame[]>, onError: ErrorSink) {
  /** By game id. */
  const resume = ref<Record<string, ResumePoint>>({})

  // Rule 7: the games played last changed while their points loaded: the older answer is dropped.
  const loads = latest()

  async function load() {
    const games = recent.value
    if (games.length === 0) return
    const fresh = loads.next()
    let slots = DEFAULT_SLOT_COUNT
    try {
      slots = await loadSlotCount()
    } catch (e) {
      if (fresh()) onError(e)
    }
    const points = await Promise.all(
      // Unreadable saves: the card says «Gioca» (the details' hero keeps «Riprendi»: two
      // looks of one state, kept as they were).
      games.map(async g => [g.id, (await loadResumePoint(g, slots)) ?? NOTHING_TO_RESUME] as const),
    )
    if (fresh()) resume.value = Object.fromEntries(points)
  }
  watch(
    () => recent.value.map(g => `${g.id}:${g.lastPlayed}`).join(),
    () => void load(),
    { immediate: true },
  )

  return resume
}
