// The library store: loads, the newest of several reloads wins, and L11's per-console count of a
// scan comes from the ids that were not there when it started.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameFamily, GameId, LibraryChanged, LibraryGame, LibraryScanEvent } from '@/emu'
import { applyLibraryChange, folderLabel, library, reloadLibrary, startLibrary } from './library'

type Listener<T> = (e: T) => void
const bridge = vi.hoisted(() => ({
  get: vi.fn(),
  changed: [] as Array<(e: unknown) => void>,
  scan: [] as Array<(e: unknown) => void>,
}))
vi.mock('@/emu', () => ({
  emu: {
    library: {
      get: bridge.get,
      onChanged: (cb: Listener<unknown>) => {
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
const answer = (games: LibraryGame[], families: GameFamily[] = []) => ({
  games,
  folders: [],
  scanning: false,
  families,
})
const scan = (e: LibraryScanEvent) => bridge.scan.forEach(cb => cb(e))
const change = (e: LibraryChanged) => bridge.changed.forEach(cb => cb(e))

beforeEach(() => {
  vi.clearAllMocks()
  bridge.changed.length = 0
  bridge.scan.length = 0
  Object.assign(library, {
    loaded: false,
    games: [],
    families: [],
    folders: [],
    scanning: false,
    progress: null,
    summary: null,
    error: null,
  })
})

describe('the library store', () => {
  it('loads on start, then follows each change without asking again', async () => {
    bridge.get.mockResolvedValue(answer([game('a'), game('b')]))
    await startLibrary()
    expect(library.loaded).toBe(true)
    expect(library.games.map(g => g.id)).toEqual(['gba:a', 'gba:b'])
    const b = library.games[1]
    const before = library.games

    // One row changed: it replaces its own, in a new list; the others are the same objects.
    const renamed = { ...game('a'), title: 'Renamed' }
    change({ kind: 'games', games: [renamed, game('unknown')] })
    expect(library.games).not.toBe(before)
    expect(library.games.map(g => g.id)).toEqual(['gba:a', 'gba:b'])
    expect(library.games[0]?.title).toBe('Renamed')
    expect(library.games[1]).toBe(b)

    // Everything changed (a scan): the library sent replaces ours.
    change({ kind: 'all', library: answer([game('c', 'nds')]) })
    expect(library.games.map(g => g.id)).toEqual(['nds:c'])
    expect(bridge.get).toHaveBeenCalledOnce()
  })

  it('takes the families Rust grouped, with the library and on their own (#102)', async () => {
    const family = (shown: string, ...rest: string[]): GameFamily => ({
      id: 'gba:BPE',
      shown: `gba:${shown}` as GameId,
      versions: [shown, ...rest].map(id => `gba:${id}` as GameId),
      pinned: false,
    })
    bridge.get.mockResolvedValue(answer([game('a'), game('b')], [family('a', 'b')]))
    await startLibrary()
    expect(library.families.map(f => f.shown)).toEqual(['gba:a'])
    const games = library.games
    // A pin or `library.versions`: only the families change, the rows stay the same list.
    change({ kind: 'families', families: [{ ...family('b', 'a'), pinned: true }] })
    expect(library.families.map(f => [f.shown, f.pinned])).toEqual([['gba:b', true]])
    expect(library.games).toBe(games)
    change({ kind: 'all', library: answer([game('c')]) })
    expect(library.families).toEqual([])
  })

  it('a change is newer than a load still answering', async () => {
    let pending!: (v: unknown) => void
    bridge.get.mockImplementationOnce(() => new Promise(r => (pending = r)))
    const older = reloadLibrary()
    applyLibraryChange({ kind: 'all', library: answer([game('new')]) })
    pending(answer([game('old')]))
    await older
    expect(library.games.map(g => g.id)).toEqual(['gba:new'])
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

  it('counts every scan since the banner was last closed (#127: the first run)', async () => {
    bridge.get.mockResolvedValue(answer([]))
    await startLibrary()
    scan({ phase: 'started' } as LibraryScanEvent)
    bridge.get.mockResolvedValue(answer([game('a')]))
    scan({ phase: 'done', added: 1 } as LibraryScanEvent)
    await flush()
    scan({ phase: 'started' } as LibraryScanEvent)
    bridge.get.mockResolvedValue(answer([game('a'), game('b', 'nds')]))
    scan({ phase: 'done', added: 1 } as LibraryScanEvent)
    await flush()
    expect(library.summary).toEqual({ added: 2, perConsole: { gba: 1, nds: 1 } })
    // Closed, the next scan starts a new count.
    library.summary = null
    scan({ phase: 'started' } as LibraryScanEvent)
    bridge.get.mockResolvedValue(answer([game('a'), game('b', 'nds'), game('c', 'nds')]))
    scan({ phase: 'done', added: 1 } as LibraryScanEvent)
    await flush()
    expect(library.summary).toEqual({ added: 1, perConsole: { nds: 1 } })
  })
})

describe('folderLabel', () => {
  const folders = [
    { path: 'C:/Games', name: 'C:/Games', reachable: true },
    { path: 'content://x/tree/primary%3AROMs', name: 'ROMs', reachable: true },
    { path: 'content://x/tree/primary%3AGone', name: null, reachable: false },
  ]
  it('shows the name Rust sends, never a stored URI', () => {
    expect(folderLabel(folders, 'C:/Games', 'Chosen folder')).toBe('C:/Games')
    expect(folderLabel(folders, 'content://x/tree/primary%3AROMs', 'Chosen folder')).toBe('ROMs')
    expect(folderLabel(folders, 'content://x/tree/primary%3AGone', 'Chosen folder')).toBe(
      'Chosen folder',
    )
    expect(folderLabel(folders, 'content://x/tree/new', 'Chosen folder')).toBe('Chosen folder')
  })
})
