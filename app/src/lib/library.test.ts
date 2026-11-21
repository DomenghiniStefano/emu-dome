// The library store: loads, the newest of several reloads wins, and L11's per-console count of a
// scan comes from the ids that were not there when it started.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameId, LibraryGame, LibraryScanEvent } from '@/emu'
import { library, reloadLibrary, startLibrary } from './library'

type Listener<T> = (e: T) => void
const bridge = vi.hoisted(() => ({
  get: vi.fn(),
  changed: [] as Array<() => void>,
  scan: [] as Array<(e: unknown) => void>,
}))
vi.mock('@/emu', () => ({
  emu: {
    library: {
      get: bridge.get,
      onChanged: (cb: () => void) => {
        bridge.changed.push(cb)
        return Promise.resolve(() => {})
      },
      onScan: (cb: Listener<unknown>) => {
        bridge.scan.push(cb)
        return Promise.resolve(() => {})
      },
    },
  },
}))

const flush = () => new Promise(r => setTimeout(r, 0))
const game = (id: string, console: LibraryGame['console'] = 'gba') =>
  ({ id: `${console}:${id}` as GameId, console }) as LibraryGame
const answer = (games: LibraryGame[]) => ({ games, folders: [], scanning: false })
const scan = (e: LibraryScanEvent) => bridge.scan.forEach(cb => cb(e))

beforeEach(() => {
  vi.clearAllMocks()
  bridge.changed.length = 0
  bridge.scan.length = 0
  Object.assign(library, {
    loaded: false,
    games: [],
    folders: [],
    scanning: false,
    progress: null,
    summary: null,
    error: null,
  })
})

describe('the library store', () => {
  it('loads on start and on every change', async () => {
    bridge.get.mockResolvedValue(answer([game('a')]))
    await startLibrary()
    expect(library.loaded).toBe(true)
    expect(library.games.map(g => g.id)).toEqual(['gba:a'])
    bridge.get.mockResolvedValue(answer([game('a'), game('b')]))
    bridge.changed.forEach(cb => cb())
    await flush()
    expect(library.games).toHaveLength(2)
  })

  it('keeps only the newest of overlapping reloads', async () => {
    let first!: (v: unknown) => void
    bridge.get.mockImplementationOnce(() => new Promise(r => (first = r)))
    bridge.get.mockResolvedValueOnce(answer([game('new')]))
    const older = reloadLibrary()
    await reloadLibrary()
    first(answer([game('old')]))
    await older
    expect(library.games.map(g => g.id)).toEqual(['gba:new'])
  })

  it('a failed load is said, the games already there are kept', async () => {
    bridge.get.mockResolvedValueOnce(answer([game('a')]))
    await reloadLibrary()
    bridge.get.mockRejectedValueOnce({ code: 'libraryReadFailed' })
    await reloadLibrary()
    expect(library.error).not.toBeNull()
    expect(library.games).toHaveLength(1)
    expect(library.loaded).toBe(true)
  })
})

describe('a scan (L11)', () => {
  it('counts per console only the games that were not there when it started', async () => {
    bridge.get.mockResolvedValue(answer([game('a')]))
    await startLibrary()
    scan({ phase: 'started' } as LibraryScanEvent)
    scan({ phase: 'progress', checked: 3, found: 2 } as LibraryScanEvent)
    expect(library.scanning).toBe(true)
    expect(library.progress).toEqual({ checked: 3, found: 2 })
    bridge.get.mockResolvedValue(answer([game('a'), game('b'), game('c', 'nds'), game('d', 'nds')]))
    scan({ phase: 'done', added: 3 } as LibraryScanEvent)
    await flush()
    expect(library.scanning).toBe(false)
    expect(library.progress).toBeNull()
    expect(library.summary).toEqual({ added: 3, perConsole: { gba: 1, nds: 2 } })
  })

  it('a scan that added nothing shows no summary', async () => {
    bridge.get.mockResolvedValue(answer([game('a')]))
    await startLibrary()
    scan({ phase: 'started' } as LibraryScanEvent)
    scan({ phase: 'done', added: 0 } as LibraryScanEvent)
    await flush()
    expect(library.summary).toBeNull()
  })
})
