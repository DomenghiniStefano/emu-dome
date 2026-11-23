import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { GameInfo, LibraryGame } from '@/emu'
import { i18n } from '@/i18n'
import { causeText, errorText } from '@/i18n/labels'
import { flow } from './gameFlow'
import { launchFailure, startProblem } from './launch'
import { useGameLaunch } from './useGameLaunch'
import type { GameSession } from './useGameSession'
import type { Toasts } from '@/composables/useToasts'

const bridge = vi.hoisted(() => ({
  openGame: vi.fn(),
  libraryOpen: vi.fn(),
  startupGame: vi.fn(),
  startupLastGame: vi.fn(),
  setPaused: vi.fn(),
  loadState: vi.fn(),
  abandonGame: vi.fn(),
  restoreLatest: vi.fn(),
  pick: vi.fn(),
  title: vi.fn(),
  clearToLaunch: vi.fn(),
}))
// G05 (#31): the cloud's question before a library game opens.
vi.mock('@/components/cloud/cloudState', () => ({ clearToLaunch: bridge.clearToLaunch }))
vi.mock('@/emu', () => ({
  AUTO_SLOT: 0,
  emu: {
    openGame: bridge.openGame,
    library: { open: bridge.libraryOpen },
    startupGame: bridge.startupGame,
    startupLastGame: bridge.startupLastGame,
    pickGameFile: bridge.pick,
    gameFileTitle: bridge.title,
    setPaused: bridge.setPaused,
    loadState: bridge.loadState,
    abandonGame: bridge.abandonGame,
    saves: { restoreLatest: bridge.restoreLatest },
  },
}))

const info = (title: string) => ({ title }) as GameInfo
const minish = {
  id: 'gba:01',
  title: 'The Minish Cap',
  path: 'C:/roms/minish.gba',
  colour: '#2a6',
} as LibraryGame
const unreadable = {
  code: 'saveUnreadable',
  params: { game: 'gba:01', aside: 'broken-1.sav', backupAt: 1700 },
}
// A file opened by its path: Rust identified it and names its key in the error.
const kirbyUnreadable = {
  code: 'saveUnreadable',
  params: { game: 'gba:0c', aside: 'broken-2.sav', backupAt: 1800 },
}

/** G02 for a failure that names no file. */
const noFile = { file: null, path: null }

function setup() {
  const session = {
    begin: vi.fn(async () => {}),
    close: vi.fn(async () => {}),
    end: vi.fn(),
    game: ref<GameInfo | null>(null),
  }
  const toasts = { notify: vi.fn(), fail: vi.fn(), failWith: vi.fn() }
  let launch!: ReturnType<typeof useGameLaunch>
  mount(
    defineComponent({
      setup() {
        launch = useGameLaunch(session as unknown as GameSession, toasts as unknown as Toasts)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { launch, session, toasts }
}

beforeEach(() => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
  Object.assign(flow, {
    launching: null,
    startError: null,
    saveBroken: null,
    resumeAsk: false,
    crashed: null,
  })
  bridge.openGame.mockResolvedValue(info('MINISH'))
  bridge.libraryOpen.mockResolvedValue(info('MINISH'))
  // Rust's title of a file outside the library: its name without folder and extension.
  bridge.title.mockImplementation(async (path: string) =>
    (path.split('/').pop() ?? path).replace(/\.[^.]+$/, ''),
  )
  bridge.clearToLaunch.mockResolvedValue(true)
  for (const f of [bridge.setPaused, bridge.loadState, bridge.abandonGame, bridge.restoreLatest])
    f.mockResolvedValue(undefined)
})

describe('launch', () => {
  it('tells an unreadable in-game save (E04) from any other failure (G02)', () => {
    expect(launchFailure(unreadable)).toEqual({
      kind: 'saveBroken',
      game: 'gba:01',
      backupAt: 1700,
    })
    expect(launchFailure({ code: 'io' })).toEqual({
      kind: 'startError',
      problem: { reason: errorText({ code: 'io' }), ...noFile },
    })
    expect(launchFailure(new Error('boom'))).toEqual({
      kind: 'startError',
      problem: { reason: errorText(new Error('boom')), ...noFile },
    })
  })

  it('shows the file and its place as Rust sends them, never parsed from the text (#128)', () => {
    const uri = 'content://com.android.providers.downloads.documents/document/msf%3A1000'
    const rom = {
      code: 'romUnreadable',
      params: { file: 'Kirby (USA).gba', path: uri, cause: 'permissionDenied' },
    } as const
    expect(startProblem(rom)).toEqual({
      reason: causeText('romUnreadable', 'permissionDenied'),
      file: 'Kirby (USA).gba',
      path: uri,
    })
    const cloud = { code: 'documentNotLocal', params: { file: 'Kirby.gba' } } as const
    expect(startProblem(cloud)).toEqual({ reason: errorText(cloud), file: 'Kirby.gba', path: null })
    const folder = {
      code: 'saveFolderUnavailable',
      params: { folder: 'D:/EmuDome/saves/gba/1a', cause: 'notFound' },
    } as const
    expect(startProblem(folder)).toEqual({
      reason: errorText(folder),
      file: null,
      path: 'D:/EmuDome/saves/gba/1a',
    })
  })
})

describe('useGameLaunch', () => {
  it('opens the file given under its file name, G01 showing only while it loads', async () => {
    const { launch, session } = setup()
    let shownWhileLoading: unknown = null
    bridge.openGame.mockImplementationOnce(async () => {
      shownWhileLoading = { ...flow.launching }
      return info('MINISH')
    })
    await launch.openFile('C:/roms/Minish Cap.gba')
    expect(bridge.pick).not.toHaveBeenCalled()
    expect(shownWhileLoading).toEqual({ title: 'Minish Cap', colour: null })
    expect(flow.launching).toBeNull()
    expect(session.begin).toHaveBeenCalledWith(info('MINISH'), null, false)
  })

  it('opens another library game only once the open one has closed (#32, the quick menu search)', async () => {
    const { launch, session } = setup()
    const order: string[] = []
    session.close.mockImplementationOnce(async () => void order.push('close'))
    bridge.libraryOpen.mockImplementationOnce(async () => {
      order.push('open')
      return info('MINISH')
    })
    await launch.switchToLibraryGame(minish)
    expect(order).toEqual(['close', 'open'])
    expect(bridge.libraryOpen).toHaveBeenCalledWith('gba:01')
    expect(session.begin).toHaveBeenCalledWith(info('MINISH'), minish, false)
  })

  it('asks for a file when none is given; a cancelled picker opens nothing', async () => {
    const { launch, session } = setup()
    bridge.pick.mockResolvedValueOnce(null)
    await launch.openFile()
    expect(bridge.pick).toHaveBeenCalledOnce()
    expect(bridge.openGame).not.toHaveBeenCalled()
    expect(session.begin).not.toHaveBeenCalled()

    bridge.pick.mockResolvedValueOnce('C:/roms/kirby.gba')
    await launch.openFile()
    expect(bridge.openGame).toHaveBeenCalledWith('C:/roms/kirby.gba')
  })

  it('lets Rust pick (it checks the extensions), naming them in the dialog', async () => {
    const { launch } = setup()
    bridge.pick.mockResolvedValueOnce(null)
    await launch.openFile()
    expect(bridge.pick).toHaveBeenCalledWith(i18n.global.t('library.dialogFilter'))
  })

  it('tells the user when the picker fails or refuses the file, and opens nothing', async () => {
    const { launch, toasts } = setup()
    bridge.pick.mockRejectedValueOnce({ code: 'unsupportedFormat', params: { ext: 'zip' } })
    await launch.openFile()
    expect(bridge.openGame).not.toHaveBeenCalled()
    expect(toasts.failWith).toHaveBeenCalledOnce()
  })

  it('titles a document by the name Rust reads, never by its URI (Android)', async () => {
    const { launch } = setup()
    const uri = 'content://downloads/document/raw%3A%2Fstorage%2FDownload%2FKirby.gba'
    bridge.title.mockResolvedValueOnce('Kirby')
    bridge.openGame.mockRejectedValueOnce({ code: 'io' })
    await launch.openFile(uri)
    expect(bridge.title).toHaveBeenCalledWith(uri)
    expect(flow.startError).toEqual({
      title: 'Kirby',
      reason: errorText({ code: 'io' }),
      ...noFile,
    })

    // A name the provider does not tell: a generic title, still not the URI.
    bridge.title.mockResolvedValueOnce(null)
    bridge.openGame.mockRejectedValueOnce({ code: 'io' })
    await launch.openFile(uri)
    expect(flow.startError?.title).toBe(i18n.global.t('flow.untitledGame'))
  })

  it('opens the game given at startup, and nothing without one', async () => {
    const { launch } = setup()
    bridge.startupGame.mockResolvedValueOnce(null)
    await launch.openStartupGame()
    expect(bridge.openGame).not.toHaveBeenCalled()
    bridge.startupGame.mockResolvedValueOnce('C:/roms/kirby.gba')
    await launch.openStartupGame()
    expect(bridge.openGame).toHaveBeenCalledWith('C:/roms/kirby.gba')
    expect(bridge.pick).not.toHaveBeenCalled()
  })

  it('opens the last game when Rust names one and no game was given', async () => {
    const { launch, session } = setup()
    bridge.startupGame.mockResolvedValueOnce(null)
    bridge.startupLastGame.mockResolvedValueOnce(minish)
    await launch.openStartupGame()
    expect(bridge.libraryOpen).toHaveBeenCalledWith(minish.id)
    expect(session.begin).toHaveBeenCalledWith(info('MINISH'), minish, false)
    // A game given at startup wins: the last game is not even asked for.
    bridge.startupLastGame.mockClear()
    bridge.startupGame.mockResolvedValueOnce('C:/roms/kirby.gba')
    await launch.openStartupGame()
    expect(bridge.startupLastGame).not.toHaveBeenCalled()
  })

  it('opens a library game by its id, over its cover colour, and counts it as that game', async () => {
    const { launch, session } = setup()
    await launch.openLibraryGame(minish)
    expect(bridge.libraryOpen).toHaveBeenCalledWith(minish.id)
    expect(session.begin).toHaveBeenCalledWith(info('MINISH'), minish, false)
    expect(bridge.loadState).not.toHaveBeenCalled()
  })

  it('«Apri il gioco e carica»: the quick save asked for wins over the resume question', async () => {
    const { launch } = setup()
    flow.resumeAsk = true
    await launch.openLibraryGame(minish, 3)
    expect(flow.resumeAsk).toBe(false)
    expect(bridge.setPaused).toHaveBeenCalledWith(false)
    expect(bridge.loadState).toHaveBeenCalledWith(3)
  })

  it('a game that cannot start shows G02 with the reason, and loads no quick save', async () => {
    const { launch, session } = setup()
    bridge.libraryOpen.mockRejectedValueOnce({ code: 'io' })
    await launch.openLibraryGame(minish, 3)
    expect(flow.startError).toEqual({
      title: minish.title,
      reason: errorText({ code: 'io' }),
      ...noFile,
    })
    expect(flow.launching).toBeNull()
    expect(session.begin).not.toHaveBeenCalled()
    expect(bridge.loadState).not.toHaveBeenCalled()
  })

  it('a failure after the game opened is told over it, not as G02 «could not start»', async () => {
    const { launch, session, toasts } = setup()
    session.begin.mockRejectedValueOnce({ code: 'stateInvalid', params: { slot: 0 } })
    await launch.openLibraryGame(minish)
    expect(flow.startError).toBeNull()
    expect(flow.launching).toBeNull()
    expect(toasts.failWith).toHaveBeenCalledWith({ code: 'stateInvalid', params: { slot: 0 } })
  })

  it('G05: a game open on another device opens only on «Gioca comunque»', async () => {
    const { launch, session } = setup()
    bridge.clearToLaunch.mockResolvedValueOnce(false)
    await launch.openLibraryGame(minish, 3)
    expect(bridge.clearToLaunch).toHaveBeenCalledWith(minish)
    expect(bridge.libraryOpen).not.toHaveBeenCalled()
    expect(session.begin).not.toHaveBeenCalled()
    expect(flow.launching).toBeNull()
    // The answer let go of the launch: the next one goes through.
    await launch.openLibraryGame(minish)
    expect(bridge.libraryOpen).toHaveBeenCalledWith(minish.id)
  })

  it('a second launch while one is opening is dropped', async () => {
    const { launch, session } = setup()
    const first = launch.openLibraryGame(minish)
    const second = launch.openFile('C:/roms/kirby.gba')
    await Promise.all([first, second])
    expect(bridge.libraryOpen).toHaveBeenCalledOnce()
    expect(bridge.openGame).not.toHaveBeenCalled()
    expect(session.begin).toHaveBeenCalledOnce()
    // Once it ended, the next one opens.
    await launch.openFile('C:/roms/kirby.gba')
    expect(bridge.openGame).toHaveBeenCalledWith('C:/roms/kirby.gba')
  })

  it('answers the resume question: from the auto quick save, or from where the game starts', async () => {
    const { launch } = setup()
    flow.resumeAsk = true
    await launch.answerResume(true)
    expect(flow.resumeAsk).toBe(false)
    expect(bridge.loadState).toHaveBeenCalledWith(0)
    expect(bridge.setPaused).toHaveBeenCalledWith(false)

    vi.clearAllMocks()
    flow.resumeAsk = true
    await launch.answerResume(false)
    expect(bridge.loadState).not.toHaveBeenCalled()
    expect(bridge.setPaused).toHaveBeenCalledWith(false)
  })

  it('a resume whose auto quick save cannot load says so and plays on', async () => {
    const { launch, toasts } = setup()
    bridge.loadState.mockRejectedValueOnce({ code: 'stateInvalid', params: { slot: 0 } })
    await launch.answerResume(true)
    expect(toasts.failWith).toHaveBeenCalledWith({ code: 'stateInvalid', params: { slot: 0 } })
    expect(bridge.setPaused).toHaveBeenCalledWith(false)
  })

  it('E11: leaving the crash dialog ends the game even when Rust cannot let go of it', async () => {
    const { launch, session, toasts } = setup()
    flow.crashed = { console: 'gba', at: null, detail: '' }
    bridge.abandonGame.mockRejectedValueOnce(new Error('ipc'))
    await launch.crashToLibrary()
    expect(flow.crashed).toBeNull()
    expect(session.end).toHaveBeenCalledOnce()
    expect(toasts.failWith).toHaveBeenCalledWith(new Error('ipc'))
  })

  describe('E04: the in-game save cannot be read', () => {
    it('shows E04 with the newest backup instead of G02', async () => {
      const { launch } = setup()
      bridge.libraryOpen.mockRejectedValueOnce(unreadable)
      await launch.openLibraryGame(minish)
      expect(flow.saveBroken).toEqual({ title: minish.title, backupAt: 1700 })
      expect(flow.startError).toBeNull()
    })

    it('«Usa il backup» restores the newest backup of the game Rust named, then opens it again', async () => {
      const { launch, session } = setup()
      bridge.libraryOpen.mockRejectedValueOnce(unreadable)
      await launch.openLibraryGame(minish)
      await launch.useLatestBackup()
      expect(flow.saveBroken).toBeNull()
      expect(bridge.restoreLatest).toHaveBeenCalledWith(minish.id)
      expect(session.begin).toHaveBeenCalledWith(info('MINISH'), minish, false)
    })

    it('a backup that cannot be restored shows G02 and opens nothing', async () => {
      const { launch, session } = setup()
      bridge.openGame.mockRejectedValueOnce(kirbyUnreadable)
      await launch.openFile('C:/roms/kirby.gba')
      bridge.restoreLatest.mockRejectedValueOnce({ code: 'io' })
      await launch.useLatestBackup()
      expect(bridge.restoreLatest).toHaveBeenCalledWith('gba:0c')
      expect(flow.startError).toEqual({
        title: 'kirby',
        reason: errorText({ code: 'io' }),
        ...noFile,
      })
      expect(bridge.openGame).toHaveBeenCalledOnce()
      expect(session.begin).not.toHaveBeenCalled()
    })

    it('«Gioca senza salvataggio» opens the same game again, without restoring', async () => {
      const { launch, session } = setup()
      bridge.openGame.mockRejectedValueOnce(unreadable)
      await launch.openFile('C:/roms/kirby.gba')
      await launch.playWithoutSave()
      expect(bridge.restoreLatest).not.toHaveBeenCalled()
      expect(bridge.openGame).toHaveBeenCalledTimes(2)
      expect(session.begin).toHaveBeenCalledOnce()
    })

    it('closing E04 forgets the game: its buttons then do nothing', async () => {
      const { launch } = setup()
      bridge.openGame.mockRejectedValueOnce(unreadable)
      await launch.openFile('C:/roms/kirby.gba')
      launch.closeBroken()
      expect(flow.saveBroken).toBeNull()
      await launch.useLatestBackup()
      await launch.playWithoutSave()
      expect(bridge.restoreLatest).not.toHaveBeenCalled()
      expect(bridge.openGame).toHaveBeenCalledOnce()
    })
  })

  describe('E11: the emulator stopped', () => {
    it('«Torna alla libreria» abandons the game and ends the session', async () => {
      const { launch, session } = setup()
      flow.crashed = { console: 'gba', at: null, detail: '' }
      await launch.crashToLibrary()
      expect(flow.crashed).toBeNull()
      expect(bridge.abandonGame).toHaveBeenCalledOnce()
      expect(session.end).toHaveBeenCalledOnce()
    })

    it('«Riprendi» opens the last game again from its auto quick save, under the running title', async () => {
      const { launch, session } = setup()
      await launch.openLibraryGame(minish)
      session.game.value = info('MINISH')
      flow.crashed = { console: 'gba', at: 1, detail: '' }
      let titleWhileLoading: string | undefined
      bridge.libraryOpen.mockImplementationOnce(async () => {
        titleWhileLoading = flow.launching?.title
        return info('MINISH')
      })
      await launch.crashResume()
      expect(flow.crashed).toBeNull()
      expect(bridge.abandonGame).toHaveBeenCalledOnce()
      expect(session.end).toHaveBeenCalledOnce()
      expect(titleWhileLoading).toBe('MINISH')
      expect(session.begin).toHaveBeenLastCalledWith(info('MINISH'), minish, true)
    })

    it('«Riprendi» with no game opened yet only abandons', async () => {
      const { launch, session } = setup()
      await launch.crashResume()
      expect(session.end).toHaveBeenCalledOnce()
      expect(session.begin).not.toHaveBeenCalled()
    })

    it('«Copia i dettagli» copies the crash detail and says so; a refused clipboard says why', async () => {
      const { launch, toasts } = setup()
      const writeText = vi.fn<(text: string) => Promise<void>>(async () => {})
      Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
      flow.crashed = { console: 'gba', at: null, detail: 'MINISH · mgba' }
      await launch.crashCopy()
      expect(writeText).toHaveBeenCalledWith('MINISH · mgba')
      expect(toasts.notify).toHaveBeenCalledWith(i18n.global.t('flow.detailsCopied'))

      writeText.mockRejectedValueOnce(new Error('denied'))
      await launch.crashCopy()
      expect(toasts.failWith).toHaveBeenCalledWith(new Error('denied'))
    })
  })
})
