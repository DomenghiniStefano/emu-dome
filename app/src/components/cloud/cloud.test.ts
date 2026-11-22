// The Cloud section's pure decisions: the status card's pill, the space bar, a game's action and
// tone, the Drive-only games, G05's minutes, a transfer's share.
import { describe, expect, it } from 'vitest'
import type { CloudGame, CloudProgress, CloudStatus, GameId } from '@/emu'
import {
  driveOnly,
  errorKey,
  gameAction,
  gameTone,
  minutesSince,
  progressOf,
  spaceBar,
  statusPill,
  transferShare,
} from './cloud'

const GB = 1024 ** 3
const status = (over: Partial<CloudStatus> = {}): CloudStatus => ({
  configured: true,
  supported: true,
  signedIn: true,
  account: null,
  quota: null,
  emudomeBytes: null,
  syncing: false,
  lastSync: 1,
  conflicts: [],
  pending: [],
  error: null,
  ...over,
})
const id = (hex: string) => `gba:${hex}` as GameId
const game = (over: Partial<CloudGame>): CloudGame => ({
  id: id('aa'),
  title: 'A',
  console: 'gba',
  size: 1,
  state: 'both',
  here: true,
  onDrive: true,
  savesOnDrive: true,
  devices: [],
  ...over,
})

describe('statusPill', () => {
  it('puts a run first, then a failure, the conflicts, the pending games', () => {
    const offline = { code: 'cloudOffline', params: { detail: 'x' } } as const
    expect(statusPill(status({ syncing: true, error: offline }))).toEqual({ kind: 'syncing' })
    expect(statusPill(status({ error: offline, conflicts: [id('a')] }))).toEqual({
      kind: 'error',
      error: offline,
    })
    expect(statusPill(status({ conflicts: [id('a'), id('b')], pending: [id('c')] }))).toEqual({
      kind: 'conflicts',
      n: 2,
    })
    expect(statusPill(status({ pending: [id('c')] }))).toEqual({ kind: 'pending', n: 1 })
  })

  it('says synced, or never synced before the first sync', () => {
    expect(statusPill(status())).toEqual({ kind: 'synced' })
    expect(statusPill(status({ lastSync: null }))).toEqual({ kind: 'never' })
  })
})

describe('spaceBar', () => {
  it('is null until Drive measured the space', () => {
    expect(spaceBar(status())).toBeNull()
  })

  it('splits the space into EmuDome, the other files and what is free', () => {
    const bar = spaceBar(
      status({ quota: { limit: 15 * GB, usage: 6 * GB }, emudomeBytes: 1.5 * GB }),
    )
    expect(bar).toEqual({
      emudome: 0.1,
      other: 0.3,
      emudomeBytes: 1.5 * GB,
      otherBytes: 4.5 * GB,
      free: 9 * GB,
      limit: 15 * GB,
    })
  })

  it('has no shares and no free space without a limit', () => {
    const bar = spaceBar(status({ quota: { limit: null, usage: 2 * GB }, emudomeBytes: GB }))
    expect(bar).toMatchObject({ emudome: 0, other: 0, free: null, limit: null, otherBytes: GB })
  })

  it('never runs past the bar, whatever Drive counted', () => {
    const bar = spaceBar(status({ quota: { limit: GB, usage: 3 * GB }, emudomeBytes: 5 * GB }))
    expect(bar).toMatchObject({ emudome: 1, other: 0, free: 0, emudomeBytes: 3 * GB })
    const none = spaceBar(status({ quota: { limit: GB, usage: GB / 2 }, emudomeBytes: null }))
    expect(none).toMatchObject({ emudome: 0, other: 0.5, emudomeBytes: 0 })
  })
})

describe('gameAction and gameTone', () => {
  it('uploads a game only here, downloads one only on Drive, removes one on Drive', () => {
    expect(gameAction(game({ state: 'localOnly', onDrive: false }))).toBe('upload')
    expect(gameAction(game({ state: 'driveOnly', here: false }))).toBe('download')
    expect(gameAction(game({ state: 'both' }))).toBe('remove')
    expect(gameAction(game({ state: 'conflict' }))).toBe('remove')
    expect(gameAction(game({ state: 'conflict', onDrive: false }))).toBeNull()
  })

  it('marks both places good and two versions to look at', () => {
    expect(gameTone('both')).toBe('ok')
    expect(gameTone('conflict')).toBe('warn')
    expect(gameTone('driveOnly')).toBe('quiet')
    expect(gameTone('localOnly')).toBe('quiet')
  })
})

describe('driveOnly', () => {
  it('keeps the games only on Drive and not in this library', () => {
    const only = game({ id: id('01'), state: 'driveOnly', here: false })
    const games = [only, game({ id: id('02') }), game({ id: id('03'), state: 'localOnly' })]
    expect(driveOnly(games)).toEqual([only])
  })
})

describe('minutesSince', () => {
  it('counts whole minutes, at least one', () => {
    expect(minutesSince(0, 12 * 60_000 + 20_000)).toBe(12)
    expect(minutesSince(1000, 1000)).toBe(1)
    // Another device's clock ahead of this one.
    expect(minutesSince(5000, 1000)).toBe(1)
  })
})

describe('transfers', () => {
  const p = (over: Partial<CloudProgress>): CloudProgress => ({
    phase: 'pulling',
    game: id('aa'),
    done: 0,
    total: 0,
    ...over,
  })

  it('has no share while the size is unknown', () => {
    expect(transferShare(null)).toBeNull()
    expect(transferShare(p({}))).toBeNull()
    expect(transferShare(p({ done: 25, total: 100 }))).toBe(25)
    expect(transferShare(p({ done: 150, total: 100 }))).toBe(100)
  })

  it('finds the transfer of a phase and a game', () => {
    const pull = p({})
    expect(progressOf(pull, 'pulling', id('aa'))).toBe(pull)
    expect(progressOf(pull, 'pulling', null)).toBe(pull)
    expect(progressOf(pull, 'pulling', id('bb'))).toBeNull()
    expect(progressOf(pull, 'pushing', null)).toBeNull()
    expect(progressOf(null, 'pulling', null)).toBeNull()
  })
})

describe('errorKey', () => {
  it('is the code, or null', () => {
    expect(errorKey(null)).toBeNull()
    expect(errorKey({ code: 'cloudAccessExpired' })).toBe('cloudAccessExpired')
  })
})
