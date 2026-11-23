import { describe, expect, it } from 'vitest'
import type { CloudStatus } from '@/emu'
import { driveRow } from './driveRow'

const status = (over: Partial<CloudStatus> = {}): CloudStatus => ({
  configured: true,
  supported: true,
  signedIn: false,
  account: null,
  quota: null,
  emudomeBytes: null,
  syncing: false,
  lastSync: null,
  conflicts: [],
  pending: [],
  error: null,
  ...over,
})

describe('driveRow (S06, #112)', () => {
  it('waits for the status', () => {
    expect(driveRow(null, false)).toEqual({ kind: 'unknown' })
  })

  it('keeps the button off with the reason where sign-in is not possible', () => {
    expect(driveRow(status({ configured: false }), false)).toEqual({
      kind: 'blocked',
      reason: { code: 'cloudNotConfigured' },
    })
    expect(driveRow(status({ supported: false }), false)).toEqual({
      kind: 'blocked',
      reason: { code: 'cloudSignInUnsupported' },
    })
  })

  it('offers the sign-in, then waits for the browser', () => {
    expect(driveRow(status(), false)).toEqual({ kind: 'signedOut' })
    expect(driveRow(status(), true)).toEqual({ kind: 'signingIn' })
  })

  it('names the account once signed in', () => {
    const account = { name: null, email: 'a@example.com' } as CloudStatus['account']
    expect(driveRow(status({ signedIn: true, account }), false)).toEqual({
      kind: 'signedIn',
      name: 'a@example.com',
    })
    expect(driveRow(status({ signedIn: true }), false)).toEqual({ kind: 'signedIn', name: null })
  })
})
