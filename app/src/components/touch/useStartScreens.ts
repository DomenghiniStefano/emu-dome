// Where the C06 editor starts the game screens (Trello #134) when the layout has no placement of
// its own: every screen of the console as fractions of the whole window, composed by Rust
// (`emu.screens.phoneStart`) with the settings of `scope`, as the game would show them. Asked
// again when the window changes; only the newest answer is applied (rule 7), errors go to the
// sink (rule 6).
import { ref, watch, type Ref } from 'vue'
import { emu, type ConsoleId, type CustomScreen, type SettingsScope } from '@/emu'
import { insetShares, NO_INSET } from '@/components/game/inset'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import type { Area } from './touchLayout'

export function useStartScreens(
  console: ConsoleId,
  scope: SettingsScope,
  area: Ref<Area>,
  onError: ErrorSink,
): Ref<CustomScreen[]> {
  const screens = ref<CustomScreen[]>([])
  const asks = latest()

  async function ask({ width, height }: Area) {
    const fresh = asks.next()
    try {
      const next = await emu.screens.phoneStart(scope, console, {
        width,
        height,
        inset: insetShares(NO_INSET, width, height),
      })
      if (fresh()) screens.value = next
    } catch (e) {
      if (fresh()) onError(e)
    }
  }
  watch(
    () => [area.value.width, area.value.height] as const,
    ([width, height]) => void ask({ width, height }),
    { immediate: true },
  )

  return screens
}
