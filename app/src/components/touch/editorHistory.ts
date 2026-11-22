// The C06 editor's undo / redo (Trello #29), pure: the layouts before and after the one shown.
// A gesture (a drag, a pinch, a slider held) is one step: the editor commits when it ends.

export interface History<T> {
  past: readonly T[]
  present: T
  future: readonly T[]
}

/** How many steps back the editor keeps. */
export const HISTORY_LIMIT = 100

export const historyOf = <T>(present: T): History<T> => ({ past: [], present, future: [] })

/** `next` becomes the present (the old one a step back, the steps forward forgotten); the same
 * value (deep-equal) changes nothing. */
export function commit<T>(h: History<T>, next: T): History<T> {
  if (JSON.stringify(next) === JSON.stringify(h.present)) return h
  return { past: [...h.past, h.present].slice(-HISTORY_LIMIT), present: next, future: [] }
}

export function undo<T>(h: History<T>): History<T> {
  const before = h.past.at(-1)
  if (before === undefined) return h
  return { past: h.past.slice(0, -1), present: before, future: [h.present, ...h.future] }
}

export function redo<T>(h: History<T>): History<T> {
  const [next, ...rest] = h.future
  if (next === undefined) return h
  return { past: [...h.past, h.present], present: next, future: rest }
}

export const canUndo = (h: History<unknown>) => h.past.length > 0
export const canRedo = (h: History<unknown>) => h.future.length > 0
