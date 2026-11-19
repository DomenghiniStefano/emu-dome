// A row's capture done (C02 remap, C03 shortcuts): the source is assigned, or, when another row
// has it, the conflict waits for «Scambiali» or «Annulla». The rules are remapTable.ts's.
import { ref, type Ref } from 'vue'
import {
  captureOutcome,
  swapChanges,
  type Binding,
  type BindingChanges,
  type BindingConflict,
} from './remapTable'

export function useBindingConflict<K extends string, S extends string>(options: {
  rows: () => readonly Binding<K, S>[]
  write: (changes: BindingChanges<K, S>) => Promise<void>
  /** `source` went to `id`, alone or by a swap (the tester says so). */
  assigned?: (id: K, source: S) => void
}) {
  const conflict: Ref<BindingConflict<K, S> | null> = ref(null)

  /** The capture of `id` ended on `source`. */
  async function assign(id: K, source: S) {
    const outcome = captureOutcome(options.rows(), id, source)
    switch (outcome.kind) {
      case 'unchanged':
        return
      case 'conflict':
        conflict.value = outcome.conflict
        return
      case 'assign':
        options.assigned?.(id, source)
        await options.write(outcome.changes)
    }
  }

  /** «Scambiali». */
  async function swap() {
    const c = conflict.value
    if (!c) return
    conflict.value = null
    options.assigned?.(c.id, c.source)
    await options.write(swapChanges(options.rows(), c))
  }

  return { conflict, assign, swap }
}
