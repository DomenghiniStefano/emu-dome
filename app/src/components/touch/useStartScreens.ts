// Where the C06 editor starts the game screens (Trello #134) when the layout has no placement of
// its own: every screen of the console where Rust puts them now (`emu.screens.phoneStart`: over a
// game where it shows them, else the settings of `scope`), inside the edges the layout keeps clear of the game, as the game (or the
// Settings preview) is drawn; given back as fractions of the whole window. Asked again when the
// window or those edges change; only the newest answer is applied (rule 7), errors go to the
// sink (rule 6).
import { ref, watch, type Ref } from 'vue'
import { emu, type ConsoleId, type CustomScreen, type SettingsScope } from '@/emu'
import { sameInset, type InsetPx } from '@/components/game/inset'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import { startArea } from './editorScreens'
import type { Area } from './touchLayout'

export function useStartScreens(
  console: ConsoleId,
  scope: SettingsScope,
  area: Ref<Area>,
  inset: Ref<InsetPx>,
  onError: ErrorSink,
): Ref<CustomScreen[]> {
  const screens = ref<CustomScreen[]>([])
  const asks = latest()
  /** What was asked last: the same window and edges are not asked again. */
  const asked = { area: null as Area | null, inset: null as InsetPx | null }

  const same = (a: Area, px: InsetPx) =>
    asked.area?.width === a.width &&
    asked.area.height === a.height &&
    asked.inset !== null &&
    sameInset(asked.inset, px)

  async function ask(a: Area, px: InsetPx) {
    if (same(a, px)) return
    asked.area = { width: a.width, height: a.height }
    asked.inset = px
    const fresh = asks.next()
    try {
      const next = await emu.screens.phoneStart(scope, console, startArea(a, px))
      if (fresh()) screens.value = next
    } catch (e) {
      if (fresh()) onError(e)
    }
  }
  watch([area, inset], ([a, px]) => void ask(a, px), { immediate: true })

  return screens
}
