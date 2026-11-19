import { describe, expect, it, vi } from 'vitest'
import { captureOutcome, swapChanges, type Binding } from './remapTable'
import { useBindingConflict } from './useBindingConflict'

// C02's game buttons, or C03's shortcuts: the same rules.
type Id = 'a' | 'b' | 'x'
const rows: Binding<Id, string>[] = [
  { id: 'a', source: 'south' },
  { id: 'b', source: 'east' },
  { id: 'x', source: null },
]

describe('captureOutcome', () => {
  it('assigns a free source to the row', () => {
    expect(captureOutcome(rows, 'a', 'north')).toEqual({
      kind: 'assign',
      changes: { a: 'north' },
    })
    expect(captureOutcome(rows, 'x', 'west')).toEqual({ kind: 'assign', changes: { x: 'west' } })
  })

  it('asks to swap when another row has the source', () => {
    expect(captureOutcome(rows, 'a', 'east')).toEqual({
      kind: 'conflict',
      conflict: { id: 'a', other: 'b', source: 'east' },
    })
    expect(captureOutcome(rows, 'x', 'south')).toEqual({
      kind: 'conflict',
      conflict: { id: 'x', other: 'a', source: 'south' },
    })
  })

  it('changes nothing for the source the row already has, or for a row that is not there', () => {
    expect(captureOutcome(rows, 'a', 'south')).toEqual({ kind: 'unchanged' })
    expect(captureOutcome(rows, 'z' as Id, 'north')).toEqual({ kind: 'unchanged' })
  })
})

describe('swapChanges', () => {
  it('gives the source to the row and the row’s old source to the other', () => {
    expect(swapChanges(rows, { id: 'a', other: 'b', source: 'east' })).toEqual({
      a: 'east',
      b: 'south',
    })
  })

  it('leaves the other row with nothing when the row had nothing', () => {
    expect(swapChanges(rows, { id: 'x', other: 'a', source: 'south' })).toEqual({
      x: 'south',
      a: null,
    })
  })
})

describe('useBindingConflict', () => {
  const setup = () => {
    const write = vi.fn(async () => {})
    const assigned = vi.fn()
    const c = useBindingConflict<Id, string>({ rows: () => rows, write, assigned })
    return { c, write, assigned }
  }

  it('writes a free source at once and says it', async () => {
    const { c, write, assigned } = setup()
    await c.assign('a', 'north')
    expect(write).toHaveBeenCalledWith({ a: 'north' })
    expect(assigned).toHaveBeenCalledWith('a', 'north')
    expect(c.conflict.value).toBeNull()
  })

  it('holds a conflict, writes nothing until «Scambiali», then swaps', async () => {
    const { c, write, assigned } = setup()
    await c.assign('a', 'east')
    expect(write).not.toHaveBeenCalled()
    expect(c.conflict.value).toEqual({ id: 'a', other: 'b', source: 'east' })
    await c.swap()
    expect(c.conflict.value).toBeNull()
    expect(write).toHaveBeenCalledWith({ a: 'east', b: 'south' })
    expect(assigned).toHaveBeenCalledWith('a', 'east')
  })

  it('does nothing for an unchanged source, or a swap without a conflict', async () => {
    const { c, write } = setup()
    await c.assign('a', 'south')
    await c.swap()
    expect(write).not.toHaveBeenCalled()
  })
})
