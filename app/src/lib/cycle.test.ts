import { describe, expect, it } from 'vitest'
import { cycle } from './cycle'

describe('cycle', () => {
  const list = ['a', 'b', 'c'] as const

  it('steps forward and back', () => {
    expect(cycle(list, 'b', 1)).toBe('c')
    expect(cycle(list, 'b', -1)).toBe('a')
  })

  it('wraps around at both ends', () => {
    expect(cycle(list, 'c', 1)).toBe('a')
    expect(cycle(list, 'a', -1)).toBe('c')
  })

  it('stays on an item the list does not have, and on a list of one', () => {
    expect(cycle<string>(list, 'z', 1)).toBe('z')
    expect(cycle(['only'], 'only', 1)).toBe('only')
  })
})
