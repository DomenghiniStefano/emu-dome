// The C06 editor opened from Settings (no game, Trello #111): where the console's screens would
// be drawn if a game opened now, around the controls being laid out. Rust composes them
// (`emu.screens.preview`) with the console's settings and the edges the layout keeps clear, as it
// composes the game itself; the editor draws them as placeholders. Asked again when the window
// or the edges change; only the newest answer is applied (rule 7).
import { ref, watch, type Ref } from 'vue'
import { emu, type ConsoleId, type ScreenPlacement } from '@/emu'
import { insetShares, sameInset, type InsetPx } from '@/components/game/inset'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import type { Area } from './touchLayout'

export function useScreenPreview(
  console: ConsoleId,
  area: Ref<Area>,
  inset: Ref<InsetPx>,
  onError: ErrorSink,
) {
  const placements = ref<ScreenPlacement[]>([])
  const asks = latest()
  let askedFor: { width: number; height: number; inset: InsetPx } | null = null

  async function ask() {
    const { width, height } = area.value
    const px = inset.value
    if (askedFor?.width === width && askedFor.height === height && sameInset(askedFor.inset, px))
      return
    askedFor = { width, height, inset: px }
    const fresh = asks.next()
    try {
      const next = await emu.screens.preview({ level: 'console', console }, console, {
        width,
        height,
        inset: insetShares(px, width, height),
      })
      if (fresh()) placements.value = next
    } catch (e) {
      if (fresh()) onError(e)
    }
  }
  watch([area, inset], () => void ask(), { immediate: true })

  return placements
}
