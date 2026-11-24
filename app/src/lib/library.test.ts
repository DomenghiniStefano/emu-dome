// The library store: loads, the newest of several reloads wins, and L11's per-console count of a
// scan comes from the ids that were not there when it started.
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type {
  GameFamily,
  GameId,
  LibraryChanged,
  LibraryFolder,
  LibraryGame,
  LibraryScanEvent,
} from '@/emu'
import {
  applyLibraryChange,
  chooseFolder,
  folderLabel,
  library,
  reloadLibrary,
  replaceFolder,
  startLibrary,
  unavailableError,
} from './library'

type Listener<T> = (e: T) => void
const bridge = vi.hoisted(() => ({
  get: vi.fn(),
  choose: vi.fn(),
  remove: vi.fn(),
  changed: [] as Array<(e: unknown) => void>,
  scan: [] as Array<(e: unknown) => void>,
}))
vi.mock('@/emu', () => ({
  emu: {
    library: {
      get: bridge.get,
      chooseFolder: bridge.choose,
      removeFolder: bridge.remove,
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
    bridge.get.mockRejectedValueOnce({ code: 'libraryReadFailed', params: { cause: 'other' } })
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
    scan(done(3))
    await flush()
    expect(library.scanning).toBe(false)
    expect(library.progress).toBeNull()
    expect(library.summary).toEqual({ added: 3, perConsole: { gba: 1, nds: 2 }, skipped: NONE })
  })

  it('a scan that added nothing shows no summary', async () => {
    bridge.get.mockResolvedValue(answer([game('a')]))
    await startLibrary()
    scan({ phase: 'started' } as LibraryScanEvent)
    scan(done(0))
    await flush()
    expect(library.summary).toBeNull()
  })

  it('counts every scan since the banner was last closed (#127: the first run)', async () => {
    bridge.get.mockResolvedValue(answer([]))
    await startLibrary()
    scan({ phase: 'started' } as LibraryScanEvent)
    bridge.get.mockResolvedValue(answer([game('a')]))
    scan(done(1))
    await flush()
    scan({ phase: 'started' } as LibraryScanEvent)
    bridge.get.mockResolvedValue(answer([game('a'), game('b', 'nds')]))
    scan(done(1))
    await flush()
    expect(library.summary).toEqual({ added: 2, perConsole: { gba: 1, nds: 1 }, skipped: NONE })
    // Closed, the next scan starts a new count.
    library.summary = null
    scan({ phase: 'started' } as LibraryScanEvent)
    bridge.get.mockResolvedValue(answer([game('a'), game('b', 'nds'), game('c', 'nds')]))
    scan(done(1))
    await flush()
    expect(library.summary).toEqual({ added: 1, perConsole: { nds: 1 }, skipped: NONE })
  })

  it('says the skipped files of a folder the user picked, not of its own scans (#127)', async () => {
    bridge.get.mockResolvedValue(answer([]))
    await startLibrary()
    // A scan EmuDome started by itself: the same skipped files every time, nothing said.
    scan({ phase: 'started' } as LibraryScanEvent)
    scan(done(0, 1, 2))
    await flush()
    expect(library.summary).toBeNull()
    // A folder picked whose only game is encrypted: no game added, the reason said.
    bridge.choose.mockResolvedValue('D:/3ds')
    expect(await chooseFolder()).toBeNull()
    scan({ phase: 'started' } as LibraryScanEvent)
    scan(done(0, 1, 0))
    await flush()
    expect(library.summary).toEqual({
      added: 0,
      perConsole: {},
      skipped: { encrypted: 1, unrecognised: 0 },
    })
    // A picker closed without a folder: the next scan says nothing more.
    library.summary = null
    bridge.choose.mockResolvedValue(null)
    await chooseFolder()
    scan({ phase: 'started' } as LibraryScanEvent)
    scan(done(0, 1, 0))
    await flush()
    expect(library.summary).toBeNull()
  })
})

const NONE = { encrypted: 0, unrecognised: 0 }
const done = (added: number, encrypted = 0, unrecognised = 0) =>
  ({
    phase: 'done',
    found: added,
    added,
    unreachable: [],
    skippedEncrypted: encrypted,
    skippedUnrecognised: unrecognised,
  }) as LibraryScanEvent

describe('folderLabel', () => {
  const folders: LibraryFolder[] = [
    { path: 'C:/Games', name: 'C:/Games', access: 'reachable' },
    { path: 'content://x/tree/primary%3AROMs', name: 'ROMs', access: 'reachable' },
    { path: 'content://x/tree/primary%3AGone', name: null, access: 'unreachable' },
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

describe('replaceFolder (E01)', () => {
  const TREE = 'content://x/tree/primary%3AROMs'

  it('keeps the folder when the same one is picked again (#138: its permission back)', async () => {
    bridge.choose.mockResolvedValue(TREE)
    expect(await replaceFolder(TREE)).toBeNull()
    expect(bridge.remove).not.toHaveBeenCalled()
  })

  it('removes the old folder when another one is picked', async () => {
    bridge.choose.mockResolvedValue('content://x/tree/primary%3ANew')
    expect(await replaceFolder(TREE)).toBeNull()
    expect(bridge.remove).toHaveBeenCalledWith(TREE)
  })

  it('keeps the old folder when the picker is closed', async () => {
    bridge.choose.mockResolvedValue(null)
    expect(await replaceFolder(TREE)).toBeNull()
    expect(bridge.remove).not.toHaveBeenCalled()
  })
})

describe('unavailableError', () => {
  const folders: LibraryFolder[] = [
    { path: 'D:/Old', name: 'D:/Old', access: 'unreachable' },
    { path: 'content://x/tree/primary%3AROMs', name: 'ROMs', access: 'permissionLost' },
    { path: 'content://x/tree/primary%3ANoName', name: null, access: 'permissionLost' },
  ]
  const at = (folder: string | null) => ({ id: 'gba:0a1b' as GameId, folder })

  it('says the permission was taken back, with the folder as shown (#138)', () => {
    expect(
      unavailableError(at('content://x/tree/primary%3AROMs'), folders, 'Chosen folder'),
    ).toEqual({ code: 'libraryFolderPermissionLost', params: { path: 'ROMs' } })
    expect(
      unavailableError(at('content://x/tree/primary%3ANoName'), folders, 'Chosen folder'),
    ).toEqual({ code: 'libraryFolderPermissionLost', params: { path: 'Chosen folder' } })
  })

  it('says the file is not found otherwise', () => {
    const notFound = { code: 'libraryGameUnavailable', params: { id: 'gba:0a1b' } }
    expect(unavailableError(at('D:/Old'), folders, 'Chosen folder')).toEqual(notFound)
    // A single file, outside any folder.
    expect(unavailableError(at(null), folders, 'Chosen folder')).toEqual(notFound)
  })
})
