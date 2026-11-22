import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { SettingsChanged, SettingsSchema } from '@/emu'
import type { GamesView } from './libraryView'
import { useGamesView } from './useGamesView'

const bridge = vi.hoisted(() => ({
  page: vi.fn(),
  set: vi.fn(),
  changed: [] as Array<(e: SettingsChanged) => void>,
}))
vi.mock('@/emu', () => ({
  emu: {
    settings: {
      schema: () =>
        Promise.resolve({
          entries: [{ key: 'library.view', page: 'library' }],
        } as unknown as SettingsSchema),
      page: bridge.page,
      set: bridge.set,
      onChanged: (cb: (e: SettingsChanged) => void) => {
        bridge.changed.push(cb)
        return Promise.resolve(() => {})
      },
    },
  },
}))

const stored = (value: unknown) => [{ key: 'library.view', value, source: 'default' }]
const flush = () => new Promise(r => setTimeout(r, 0))

function setup(fallback: GamesView = 'grid') {
  const onError = vi.fn()
  let games!: ReturnType<typeof useGamesView>
  mount(
    defineComponent({
      setup() {
        games = useGamesView(() => fallback, onError)
        return () => h('div')
      },
    }),
  )
  return { games, onError }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.changed.length = 0
  bridge.page.mockResolvedValue(stored('list'))
  bridge.set.mockResolvedValue(undefined)
})

describe('useGamesView', () => {
  it('is null until read, then the setting; an unknown value is the fallback', async () => {
    const { games } = setup()
    expect(games.view.value).toBeNull()
    await flush()
    expect(bridge.page).toHaveBeenCalledWith('library', { level: 'global' })
    expect(games.view.value).toBe('list')

    bridge.page.mockResolvedValue(stored('rows'))
    const other = setup('list')
    await flush()
    expect(other.games.view.value).toBe('list')
  })

  it('an unreadable setting is reported and the fallback shown', async () => {
    bridge.page.mockRejectedValue({ code: 'settingsReadFailed' })
    const { games, onError } = setup('grid')
    await flush()
    expect(onError).toHaveBeenCalledWith({ code: 'settingsReadFailed' })
    expect(games.view.value).toBe('grid')
  })

  it('writes the view picked at once; a failure is reported and the view read back', async () => {
    const { games, onError } = setup()
    await flush()
    const picked = games.setView('grid')
    expect(games.view.value).toBe('grid')
    await picked
    expect(bridge.set).toHaveBeenCalledWith('library.view', { level: 'global' }, 'grid')

    bridge.set.mockRejectedValueOnce({ code: 'io' })
    await games.setView('grid')
    await flush()
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
    expect(games.view.value).toBe('list')
  })

  it('reads the view again when the setting changes elsewhere', async () => {
    const { games } = setup()
    await flush()
    bridge.page.mockResolvedValue(stored('grid'))
    bridge.changed.forEach(cb => cb({ key: 'library.view', level: 'global' }))
    await flush()
    expect(games.view.value).toBe('grid')
  })
})
