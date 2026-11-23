// The sync as the interface holds it: a failure told once while it lasts, G14's «Copiati» after a
// push, G04's questions (a newer one for the same game replaces the older), G05 before an open
// (asked only while signed in, never blocking on a failed check, an answer for the newest
// question only), and the newest status answer winning.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { CloudGame, CloudGameConflict, CloudStatus, GameId } from '@/emu'
import {
  answerInUse,
  applyGame,
  applyProgress,
  applyStatus,
  askConflict,
  askInUse,
  clearToLaunch,
  cloud,
  dropConflict,
  failCloud,
  inUseToAsk,
  loadStatus,
  PUSHED_VISIBLE_MS,
} from './cloudState'

const bridge = vi.hoisted(() => ({
  status: vi.fn(),
  launchCheck: vi.fn(),
  games: vi.fn(),
  report: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    cloud: { status: bridge.status, launchCheck: bridge.launchCheck, games: bridge.games },
  },
}))
vi.mock('@/lib/errors', () => ({ reportError: bridge.report }))

const id = (hex: string) => `nds:${hex}` as GameId
const status = (over: Partial<CloudStatus> = {}): CloudStatus => ({
  configured: true,
  supported: true,
  signedIn: true,
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
const offline = { code: 'cloudOffline' } as const
const conflict = (hex: string, title = 'T'): CloudGameConflict => ({
  id: id(hex),
  title,
  files: [],
  local: { device: null, modified: 1, playTime: null, size: 1, thumb: null },
  remote: { device: 'Phone', modified: 2, playTime: null, size: 1, thumb: null },
})
const flush = () => new Promise(r => setTimeout(r, 0))

beforeEach(() => {
  vi.clearAllMocks()
  applyStatus(status())
  bridge.report.mockClear()
  cloud.conflicts = []
  cloud.progress = null
  cloud.pushed = null
  if (cloud.inUse) answerInUse(false)
})

describe('failures', () => {
  it('tells a status failure once while it lasts, and again after it cleared', () => {
    applyStatus(status({ error: offline }))
    applyStatus(status({ error: offline, syncing: true }))
    expect(bridge.report).toHaveBeenCalledTimes(1)
    applyStatus(status())
    applyStatus(status({ error: offline }))
    expect(bridge.report).toHaveBeenCalledTimes(2)
  })

  it('does not tell a command’s failure the status already told', () => {
    applyStatus(status({ error: offline }))
    failCloud(offline)
    expect(bridge.report).toHaveBeenCalledTimes(1)
    failCloud({ code: 'saveGameOpen' })
    expect(bridge.report).toHaveBeenCalledTimes(2)
  })
})

describe('the status', () => {
  it('drops a status answer older than an event', async () => {
    let answer: (s: CloudStatus) => void = () => {}
    bridge.status.mockReturnValue(new Promise(r => (answer = r)))
    const load = loadStatus()
    applyStatus(status({ syncing: true }))
    answer(status({ signedIn: false }))
    await load
    expect(cloud.status?.syncing).toBe(true)
    expect(cloud.status?.signedIn).toBe(true)
  })
})

describe('transfers', () => {
  it('keeps the transfer running and shows «Copiati» for a moment after a push', () => {
    vi.useFakeTimers()
    applyProgress({ phase: 'pushing', game: id('01'), done: 0, total: 10 })
    expect(cloud.progress?.phase).toBe('pushing')
    applyProgress({ phase: 'done', game: id('01'), done: 0, total: 0 })
    expect(cloud.progress).toBeNull()
    expect(cloud.pushed).toBe(id('01'))
    vi.advanceTimersByTime(PUSHED_VISIBLE_MS)
    expect(cloud.pushed).toBeNull()
    vi.useRealTimers()
  })

  it('shows nothing after a pull ends', () => {
    applyProgress({ phase: 'pulling', game: id('01'), done: 0, total: 0 })
    applyProgress({ phase: 'done', game: id('01'), done: 0, total: 0 })
    expect(cloud.pushed).toBeNull()
  })
})

describe('games', () => {
  it('replaces a row with the command’s answer, or adds it', () => {
    cloud.games = []
    const row: CloudGame = {
      id: id('01'),
      title: 'A',
      console: 'nds',
      size: 1,
      state: 'localOnly',
      here: true,
      onDrive: false,
      savesOnDrive: false,
      devices: [],
    }
    applyGame(row)
    applyGame({ ...row, state: 'both', onDrive: true })
    expect(cloud.games).toEqual([{ ...row, state: 'both', onDrive: true }])
  })
})

describe('G04', () => {
  it('queues one question per game, the newer replacing the older', () => {
    askConflict(conflict('01', 'old'))
    askConflict(conflict('02'))
    askConflict(conflict('01', 'new'))
    expect(cloud.conflicts.map(c => c.title)).toEqual(['new', 'T'])
    dropConflict(id('01'))
    expect(cloud.conflicts.map(c => c.id)).toEqual([id('02')])
  })
})

describe('G05', () => {
  const g = { id: id('01'), title: 'Brina' }

  it('opens at once when not signed in, free or here', async () => {
    applyStatus(status({ signedIn: false }))
    expect(await clearToLaunch(g)).toBe(true)
    expect(bridge.launchCheck).not.toHaveBeenCalled()
    applyStatus(status())
    bridge.launchCheck.mockResolvedValue({ kind: 'here' })
    expect(await clearToLaunch(g)).toBe(true)
  })

  it('never keeps a game from opening because the check failed', async () => {
    bridge.launchCheck.mockRejectedValue(offline)
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    expect(await clearToLaunch(g)).toBe(true)
    warn.mockRestore()
  })

  it('asks when another device has it open, and opens on «Gioca comunque»', async () => {
    bridge.launchCheck.mockResolvedValue({ kind: 'elsewhere', name: 'Phone', since: 5, beat: 9 })
    const asked = clearToLaunch(g)
    await flush()
    expect(cloud.inUse).toEqual({ ...g, device: 'Phone', since: 5, opened: false })
    answerInUse(true)
    expect(await asked).toBe(true)
    expect(cloud.inUse).toBeNull()
    // Its own `cloud://in-use` is not asked again.
    expect(inUseToAsk(g.id)).toBe(false)
  })

  it('answers an older question «Annulla» when a newer one comes', async () => {
    const first = askInUse({ ...g, device: 'A', since: 1, opened: false })
    const second = askInUse({ ...g, id: id('02'), device: 'B', since: 1, opened: true })
    expect(await first).toBe(false)
    answerInUse(false)
    expect(await second).toBe(false)
    expect(inUseToAsk(id('02'))).toBe(true)
  })
})
