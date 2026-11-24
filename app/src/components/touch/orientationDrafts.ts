// The C06 editor keeps one draft per orientation (the owner's request on #134's follow-up): turning
// the phone switches to the other orientation's draft and back again keeps everything, history
// and pick included; «Salva» writes every orientation changed in this session in one call. Like
// two fields of one setting, `portrait` and `landscape`. Pure.
import type { TouchLayout, TouchOrientation } from '@/emu'
import { canRedo, canUndo, type History } from './editorHistory'
import type { ElementId } from './editorScreens'

/** One orientation as the editor left it. */
export interface OrientationDraft {
  history: History<TouchLayout>
  selected: ElementId | null
}

/** Each orientation as the editor left it in this session (one not shown yet: none). */
export type Drafts = Partial<Record<TouchOrientation, OrientationDraft>>

const ORIENTATIONS: readonly TouchOrientation[] = ['portrait', 'landscape']

export const draftOf = (drafts: Drafts, o: TouchOrientation): OrientationDraft | undefined =>
  drafts[o]

export const withDraft = (
  drafts: Drafts,
  o: TouchOrientation,
  draft: OrientationDraft,
): Drafts => ({
  ...drafts,
  [o]: draft,
})

/** Some step was made in this orientation (even one undone since). */
const changed = (d: OrientationDraft | undefined) =>
  !!d && (canUndo(d.history) || canRedo(d.history))

/** The layouts «Salva» writes (`null`: that orientation is not written): every orientation changed
 * in this session; with none changed, the one on screen (`current`), as one «Salva» always did. */
export function layoutsToSave(
  drafts: Drafts,
  current: TouchOrientation,
): Record<TouchOrientation, TouchLayout | null> {
  const anyChanged = ORIENTATIONS.some(o => changed(drafts[o]))
  const written = (o: TouchOrientation) => (anyChanged ? changed(drafts[o]) : o === current)
  const layout = (o: TouchOrientation) => (written(o) ? (drafts[o]?.history.present ?? null) : null)
  return { portrait: layout('portrait'), landscape: layout('landscape') }
}

/** The orientation to suggest turning the phone to: the other one while it was not shown in this
 * session; `null` once both were. */
export function turnHint(
  shown: readonly TouchOrientation[],
  current: TouchOrientation,
): TouchOrientation | null {
  const other: TouchOrientation = current === 'portrait' ? 'landscape' : 'portrait'
  return shown.includes(other) ? null : other
}
