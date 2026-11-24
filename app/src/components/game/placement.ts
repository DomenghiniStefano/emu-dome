// The phone's own placement of the game's screens, as Rust holds it for the open game: the touch
// layer's saved one, or the editor's while it is open (it wins, then the layer's comes back).
import type { CustomScreen } from '@/emu'

/** What Rust should hold: the editor's placement while it is open (`null` when closed), else the
 * layer's. Empty: none, the game's layouts apply. */
export function effectivePlacement(
  layer: readonly CustomScreen[],
  editor: readonly CustomScreen[] | null,
): CustomScreen[] {
  return [...(editor ?? layer)]
}

export function samePlacement(a: readonly CustomScreen[], b: readonly CustomScreen[]): boolean {
  return (
    a.length === b.length &&
    a.every(
      (s, i) =>
        s.id === b[i]?.id && s.x === b[i].x && s.y === b[i].y && s.w === b[i].w && s.h === b[i].h,
    )
  )
}
