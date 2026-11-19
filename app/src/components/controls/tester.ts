// The button tester's pure parts: what was just pressed, and which game buttons a source gives.
import type { Button, RemapRow } from '../../emu'

/** The sources held `now` that were not held before, in `now`'s order. */
export const newlyHeld = <T extends string>(before: ReadonlySet<T>, now: Iterable<T>): T[] =>
  [...now].filter(source => !before.has(source))

/** The game buttons a source (a position or a key) gives, in the table's order. */
export const buttonsUsing = (rows: RemapRow[], source: string): Button[] =>
  rows.filter(r => r.source === source).map(r => r.button)
