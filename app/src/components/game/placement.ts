// The phone's own placement of the game's screens, as Rust holds it for the open game: the touch
// layer's saved one, or the editor's while it is open (it wins, then the layer's comes back).
import type { CustomScreen, VideoInset } from '@/emu'
import { sameInset } from './inset'

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

/** What Rust holds of the phone's view: the placement and the edges (shares of the window) it is
 * fitted in. */
export interface PhoneView {
  placement: CustomScreen[]
  inset: VideoInset
}

/** What to send for `next` when Rust holds `sent`: both together when the placement changed (Rust
 * applies them as one, so no frame pairs a new placement with an old inset), the inset alone when
 * only it changed, nothing when neither did. A placement Rust `refused` as invalid is not sent
 * again: only its inset goes (Rust would refuse it the same way, the inset with it held back). */
export function phoneSend(
  sent: PhoneView,
  next: PhoneView,
  refused: readonly CustomScreen[] | null = null,
): 'pair' | 'inset' | null {
  const placementKept =
    samePlacement(sent.placement, next.placement) ||
    (refused !== null && samePlacement(refused, next.placement))
  if (!placementKept) return 'pair'
  return sameInset(sent.inset, next.inset) ? null : 'inset'
}

/** A call that sent `tried` was refused with `error`, Rust having held `before`. Refused as invalid
 * (`settingInvalidValue`: not the game's screens), Rust kept its placement and applied the inset,
 * and that placement is not to be sent again; any other refusal left Rust as it was, so the same
 * request goes again. */
export function afterRefusal(
  before: PhoneView,
  tried: PhoneView,
  error: unknown,
): { sent: PhoneView; refused: CustomScreen[] | null } {
  const invalid = (error as { code?: unknown } | null)?.code === 'settingInvalidValue'
  return invalid
    ? { sent: { placement: before.placement, inset: tried.inset }, refused: tried.placement }
    : { sent: before, refused: null }
}
