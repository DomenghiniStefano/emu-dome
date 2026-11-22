import { describe, expect, it } from 'vitest'
import type { UpdateState, UpdateStatus } from '@/emu'
import { UPDATE_ROWS, installing, stateVersion, updateLine } from './updates'

const status = (state: UpdateState, checkedAt: number | null = null): UpdateStatus => ({
  state,
  checkedAt,
  prompt: false,
})

describe('updates', () => {
  it('names the version of the states that have one', () => {
    expect(stateVersion({ state: 'ready', version: '0.0.2' })).toBe('0.0.2')
    expect(stateVersion({ state: 'available', version: '0.1.0' })).toBe('0.1.0')
    expect(stateVersion({ state: 'upToDate' })).toBeNull()
    expect(stateVersion({ state: 'failed', error: { code: 'updateNotReady' } })).toBeNull()
  })

  it('is on its way while it downloads or installs', () => {
    expect(installing({ state: 'downloading', version: '1' })).toBe(true)
    expect(installing({ state: 'installing', version: '1' })).toBe(true)
    expect(installing({ state: 'ready', version: '1' })).toBe(false)
  })

  it('about.upd says where the update stands, a failure in its own words', () => {
    const when = (ms: number) => `at ${ms}`
    expect(updateLine(status({ state: 'upToDate' }, 5), when)).toEqual({
      key: 'settings.about.upd.status.upToDate',
      params: { when: 'at 5' },
    })
    expect(updateLine(status({ state: 'ready', version: '0.0.2' }), when)).toEqual({
      key: 'settings.about.upd.status.ready',
      params: { version: '0.0.2' },
    })
    expect(updateLine(status({ state: 'idle' }), when)).toEqual({
      key: 'settings.about.upd.status.idle',
    })
    const error = { code: 'updateCheckFailed', params: { detail: 'offline' } } as const
    expect(updateLine(status({ state: 'failed', error }), when)).toEqual({ error })
  })

  it('locks only the rows about updating itself', () => {
    expect(UPDATE_ROWS).toEqual(['general.updates', 'about.upd'])
  })
})
