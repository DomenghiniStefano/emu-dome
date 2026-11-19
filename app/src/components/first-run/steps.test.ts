import { describe, expect, it } from 'vitest'
import { mainActionKey, stepAfterChange } from './steps'

describe('stepAfterChange', () => {
  it('stays on a step that is still there', () => {
    expect(stepAfterChange('games', 1, ['welcome', 'games', 'controller', 'ready'])).toBe('games')
  })
  it('moves to the step now at its place when it went away', () => {
    expect(stepAfterChange('controller', 2, ['welcome', 'games', 'ready'])).toBe('ready')
  })
  it('falls back to the last step past the end', () => {
    expect(stepAfterChange('controller', 5, ['welcome', 'ready'])).toBe('ready')
    expect(stepAfterChange('controller', 0, [])).toBe('ready')
  })
})

describe('mainActionKey', () => {
  it('starts, goes on, finishes', () => {
    expect(mainActionKey(0, 3)).toBe('firstRun.start')
    expect(mainActionKey(1, 3)).toBe('firstRun.next')
    expect(mainActionKey(2, 3)).toBe('firstRun.finish')
  })
})
