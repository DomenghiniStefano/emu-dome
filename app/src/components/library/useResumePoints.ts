// Where «Riprendi» picks up for each «Continua a giocare» game: the newest quick save (auto or
// a slot), else the in-game save. Read again whenever the games played last change.
import { ref, watch, type Ref } from 'vue'
import { emu, type LibraryGame, type SettingsScope } from '../../emu'
import { resumePoint, type ResumePoint } from '../../gameDetails'
import { errorText } from '../../i18n/labels'

const GLOBAL: SettingsScope = { level: 'global' }
/** `saves.slots` when the setting cannot be read. */
const DEFAULT_SLOTS = 10

export function useResumePoints(recent: Ref<LibraryGame[]>, onError: (text: string) => void) {
  const resume = ref<Record<string, ResumePoint>>({})

  async function load() {
    const games = recent.value
    if (games.length === 0) return
    let slots = DEFAULT_SLOTS
    try {
      const row = (await emu.settings.page('saves', GLOBAL)).find(r => r.key === 'saves.slots')
      slots = Number(row?.value ?? DEFAULT_SLOTS) || DEFAULT_SLOTS
    } catch (e) {
      onError(errorText(e))
    }
    const points = await Promise.all(
      games.map(async (g): Promise<[string, ResumePoint]> => {
        try {
          return [g.id, resumePoint(await emu.saves.game(g.path, slots))]
        } catch {
          // Saves unreadable for now: open the game as «Gioca» would.
          return [g.id, { slot: null, thumbUrl: null, none: true }]
        }
      }),
    )
    resume.value = Object.fromEntries(points)
  }
  watch(
    () => recent.value.map(g => `${g.id}:${g.lastPlayed}`).join(),
    () => void load(),
    { immediate: true },
  )

  return resume
}
