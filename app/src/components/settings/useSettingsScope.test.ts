import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, ref } from 'vue'
import type { ConsoleId, LibraryGame, SettingEntry } from '@/emu'
import { i18n } from '@/i18n'
import type { GameFocus } from '@/lib/gameDetails'
import { library } from '@/lib/library'
import { useSettingsScope } from './useSettingsScope'

const bridge = vi.hoisted(() => ({ gameScope: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { settings: { gameScope: bridge.gameScope } } }))
// The library as a plain reactive list the tests fill.
vi.mock('@/lib/library', async () => {
  const { reactive } = await import('vue')
  return { library: reactive({ games: [] as LibraryGame[] }) }
})

const entry = (key: string, levels: SettingEntry['levels']) => ({ key, levels }) as SettingEntry
const perConsole = entry('video.fps', ['default', 'global', 'console', 'game'])
const globalOnly = entry('general.theme', ['default', 'global'])
const game = (console: ConsoleId, lastPlayed: number | null = null) =>
  ({ console, lastPlayed }) as LibraryGame
const focus: GameFocus = {
  scope: { level: 'game', console: 'nds', game: 'nds:01' },
  title: 'Kirby',
}

function setup(entries: SettingEntry[] = [perConsole], gameFocus: GameFocus | null = null) {
  const page = ref(entries)
  let scope!: ReturnType<typeof useSettingsScope>
  mount(
    defineComponent({
      setup() {
        scope = useSettingsScope(page, gameFocus)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { scope, page }
}

beforeEach(() => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
  library.games = [game('gb', 5), game('gba', 9), game('nds')]
  bridge.gameScope.mockResolvedValue(null)
})

describe('useSettingsScope', () => {
  it('opens at «Tutti i giochi», or at «Solo questo gioco» when it opened for a game', () => {
    expect(setup().scope.level.value).toBe('global')
    const forGame = setup([perConsole], focus).scope
    expect(forGame.level.value).toBe('game')
    expect(forGame.scope.value).toEqual(focus.scope)
  })

  it('starts on the console of the game it opened for, before the open game’s', async () => {
    bridge.gameScope.mockResolvedValue({ level: 'game', console: 'gb', game: 'gb:01' })
    const { scope } = setup([perConsole], focus)
    await scope.findContextConsole()
    expect(scope.contextConsole.value).toBe('nds')
  })

  it('starts on the open game’s console, offered even without games of it in the library', async () => {
    bridge.gameScope.mockResolvedValue({ level: 'game', console: '3ds', game: '3ds:01' })
    const { scope } = setup()
    await scope.findContextConsole()
    expect(scope.contextConsole.value).toBe('3ds')
    expect(scope.consoleOptions.value.map(o => o.id)).toEqual(['gb', 'gba', 'nds', '3ds'])
  })

  it('without an open game, starts on the console played last, else the first one', async () => {
    const played = setup().scope
    await played.findContextConsole()
    expect(played.contextConsole.value).toBe('gba')

    library.games = [game('nds'), game('gb')]
    const never = setup().scope
    await never.findContextConsole()
    expect(never.contextConsole.value).toBe('gb')
  })

  it('a session scope or a failed question counts as no open game', async () => {
    bridge.gameScope.mockResolvedValueOnce({ level: 'session' })
    const session = setup().scope
    await session.findContextConsole()
    expect(session.contextConsole.value).toBe('gba')

    bridge.gameScope.mockRejectedValueOnce({ code: 'io' })
    const failed = setup().scope
    await failed.findContextConsole()
    expect(failed.contextConsole.value).toBe('gba')
  })

  it('with an empty library there is no console level at all', async () => {
    library.games = []
    const { scope } = setup()
    await scope.findContextConsole()
    expect(scope.contextConsole.value).toBeNull()
    expect(scope.levels.value).toEqual(['global'])
    expect(scope.scoped.value).toBe(true)
  })

  it('follows the library: a late answer picks a console, losing the chosen one moves off it', async () => {
    library.games = []
    const { scope } = setup()
    await scope.findContextConsole()
    library.games = [game('gb', 1), game('nds', 2)]
    await nextTick()
    expect(scope.contextConsole.value).toBe('nds')

    scope.pickConsole('gb')
    library.games = [game('gb', 1), game('nds', 2), game('gba')]
    await nextTick()
    expect(scope.contextConsole.value).toBe('gb')
    library.games = [game('gba'), game('nds')]
    await nextTick()
    expect(scope.contextConsole.value).toBe('gba')
  })

  it('picking a console in the menu selects its level; an unknown one changes nothing', async () => {
    const { scope } = setup()
    await scope.findContextConsole()
    scope.pickConsole('nds')
    expect(scope.level.value).toBe('console')
    expect(scope.scope.value).toEqual({ level: 'console', console: 'nds' })
    scope.pickConsole('3ds')
    expect(scope.contextConsole.value).toBe('nds')
    scope.level.value = 'global'
    scope.pickConsole('psx')
    expect(scope.level.value).toBe('global')
  })

  it('a page of settings for all games only has no scope bar, and edits «Tutti i giochi»', async () => {
    const { scope } = setup([globalOnly], focus)
    await scope.findContextConsole()
    expect(scope.scoped.value).toBe(false)
    expect(scope.levels.value).toEqual(['global'])
    expect(scope.effective.value).toBe('global')
    expect(scope.scope.value).toEqual({ level: 'global' })
  })

  it('moving to a page without the level chosen edits «Tutti i giochi», and back restores it', async () => {
    const { scope, page } = setup()
    await scope.findContextConsole()
    scope.pickConsole('gb')
    page.value = [globalOnly]
    expect(scope.effective.value).toBe('global')
    expect(scope.scope.value).toEqual({ level: 'global' })
    page.value = [perConsole]
    expect(scope.effective.value).toBe('console')
  })

  it('L1 / R1 step round the levels of the page; with one level they do nothing', async () => {
    const { scope, page } = setup([perConsole], focus)
    await scope.findContextConsole()
    expect(scope.levels.value).toEqual(['global', 'console', 'game'])
    scope.tab(1)
    expect(scope.level.value).toBe('global')
    scope.tab(1)
    expect(scope.level.value).toBe('console')
    scope.tab(-1)
    scope.tab(-1)
    expect(scope.level.value).toBe('game')

    page.value = [globalOnly]
    scope.level.value = 'global'
    scope.tab(1)
    expect(scope.level.value).toBe('global')
  })

  it('names the levels: the chosen console’s, the game’s, all games’', async () => {
    const { scope } = setup()
    const { t } = i18n.global
    await scope.findContextConsole()
    expect(scope.levelName('console')).toBe(
      t('settingsUi.scope.console', { console: t('console.gba') }),
    )
    expect(scope.levelName('game')).toBe(t('controls.levelGame'))
    expect(scope.levelName('global')).toBe(t('settingsUi.scope.global'))
  })

  it('a recent change opens at its console when offered, else at «Tutti i giochi»', async () => {
    const { scope } = setup()
    await scope.findContextConsole()
    scope.goTo('console', 'nds')
    expect(scope.scope.value).toEqual({ level: 'console', console: 'nds' })
    scope.goTo('console', '3ds')
    expect(scope.level.value).toBe('global')
    expect(scope.contextConsole.value).toBe('nds')
    scope.level.value = 'console'
    scope.goTo('game', null)
    expect(scope.level.value).toBe('global')
  })
})
