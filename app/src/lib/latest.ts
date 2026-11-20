// Rule 7 (CLAUDE.md, engineering rules): an async answer is applied only if it is still current.
// A request takes a token when it starts; when its answer arrives, `fresh()` says whether a newer
// request started meanwhile (another scope, another game, a newer reload), and if so the answer
// (or its error) is dropped. Pure, no Vue.

/** True while no newer request of the same kind has started. */
export type Fresh = () => boolean

export interface Latest {
  /** Starts a request: every older one stops being fresh. */
  next: () => Fresh
  /** The request started last, without starting one: for a part of it that starts later. */
  now: () => Fresh
}

export function latest(): Latest {
  let last = 0
  const token =
    (id: number): Fresh =>
    () =>
      id === last
  return {
    next: () => token(++last),
    now: () => token(last),
  }
}

/** One `latest()` per key: requests for different keys never make each other stale. */
export function latestBy<K>(): { next: (key: K) => Fresh } {
  const byKey = new Map<K, Latest>()
  return {
    next: key => {
      let l = byKey.get(key)
      if (!l) byKey.set(key, (l = latest()))
      return l.next()
    },
  }
}
