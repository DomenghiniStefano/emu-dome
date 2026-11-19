import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { GameSaves, ImportCandidate, LibraryGame } from '@/emu'
import { useSaveImport } from './useSaveImport'

// The bridge's saves calls and the library's games, as the import reads them.
const bridge = vi.hoisted(() => ({
  importPick: vi.fn(),
  game: vi.fn(),
  import: vi.fn(),
  games: [] as LibraryGame[],
}))
vi.mock('@/emu', () => ({
  emu: {
    saves: { importPick: bridge.importPick, game: bridge.game, import: bridge.import },
  },
}))
vi.mock('@/lib/library', () => ({
  library: {
    get games() {
      return bridge.games
    },
  },
}))

const game = (title: string, path: string) => ({ title, path }) as LibraryGame
const zelda = game('The Minish Cap', 'C:/roms/minish.gba')
const mario = game('mario kart', 'C:/roms/mk.gba')
const file = (fileName: string): ImportCandidate => ({
  path: `C:/downloads/${fileName}`,
  fileName,
  format: 'raw' as ImportCandidate['format'],
  size: 8192,
})
const saves = { quick: [], inGame: null } as unknown as GameSaves

function setup() {
  const fail = vi.fn()
  const imported = vi.fn(async () => {})
  const importer = useSaveImport(ref(4), fail, imported)
  return { importer, fail, imported }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.games = [zelda, mario]
  bridge.game.mockResolvedValue(saves)
  bridge.import.mockResolvedValue(undefined)
})

describe('useSaveImport', () => {
  it('finds the game from the file name (the ROM’s, else the title) and reads its saves', async () => {
    const { importer } = setup()
    bridge.importPick.mockResolvedValueOnce(file('minish.sav'))
    await importer.pickFile()
    expect(importer.target.value).toEqual(zelda)
    expect(importer.targetFound.value).toBe(true)
    expect(bridge.game).toHaveBeenCalledWith(zelda.path, 4)
    expect(importer.targetSaves.value).toEqual(saves)

    bridge.importPick.mockResolvedValueOnce(file('Mario Kart.srm'))
    await importer.pickFile()
    expect(importer.target.value).toEqual(mario)
  })

  it('says when no game matches, and keeps no target', async () => {
    const { importer } = setup()
    bridge.importPick.mockResolvedValueOnce(file('unknown.sav'))
    await importer.pickFile()
    expect(importer.candidate.value?.fileName).toBe('unknown.sav')
    expect(importer.targetFound.value).toBe(false)
    expect(importer.target.value).toBeNull()
  })

  it('keeps the game chosen first (L05 «Importa…»): the file name does not change it', async () => {
    const { importer } = setup()
    await importer.resetFor(mario.path)
    expect(importer.target.value).toEqual(mario)
    bridge.importPick.mockResolvedValueOnce(file('minish.sav'))
    await importer.pickFile()
    expect(importer.target.value).toEqual(mario)
  })

  it('keeps a game chosen in «Scegli il gioco» over a later file', async () => {
    const { importer } = setup()
    await importer.chooseTarget(zelda)
    bridge.importPick.mockResolvedValueOnce(file('mk.sav'))
    await importer.pickFile()
    expect(importer.target.value).toEqual(zelda)
  })

  it('forgets the chosen game on reset: the file name decides again', async () => {
    const { importer } = setup()
    await importer.resetFor(mario.path)
    importer.reset()
    bridge.importPick.mockResolvedValueOnce(file('minish.sav'))
    await importer.pickFile()
    expect(importer.target.value).toEqual(zelda)
  })

  it('a picker cancelled changes nothing', async () => {
    const { importer } = setup()
    bridge.importPick.mockResolvedValueOnce(null)
    await importer.pickFile()
    expect(importer.candidate.value).toBeNull()
  })

  it('imports into the target, then shows that game; a failure goes to `fail`', async () => {
    const { importer, fail, imported } = setup()
    bridge.importPick.mockResolvedValueOnce(file('minish.sav'))
    await importer.pickFile()
    await importer.run()
    expect(bridge.import).toHaveBeenCalledWith('C:/downloads/minish.sav', zelda.path)
    expect(imported).toHaveBeenCalledWith(zelda)

    bridge.import.mockRejectedValueOnce({ code: 'io' })
    await importer.run()
    expect(fail).toHaveBeenCalledWith({ code: 'io' })
    expect(importer.importing.value).toBe(false)
  })

  it('does not import without a file and a game', async () => {
    const { importer } = setup()
    await importer.run()
    expect(bridge.import).not.toHaveBeenCalled()
  })
})
