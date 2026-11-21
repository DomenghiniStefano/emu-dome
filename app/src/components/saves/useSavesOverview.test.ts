import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { GameId, LibraryGame, SavesSummary } from '@/emu'
import { i18n } from '@/i18n'
import { sizeLabel } from '@/i18n/labels'
import { library } from '@/lib/library'
import { useSavesOverview } from './useSavesOverview'

const bridge = vi.hoisted(() => ({ summary: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { saves: { summary: bridge.summary } } }))

const flush = () => new Promise(r => setTimeout(r, 0))
const game = (id: string) => ({ id: id as GameId, title: id }) as LibraryGame
const summary = (id: string, over: Partial<SavesSummary> = {}): SavesSummary => ({
  game: id as GameId,
  inGame: true,
  quickCount: 0,
  bytes: 2048,
  lastChange: 1,
  broken: false,
  ...over,
})

function setup() {
  const fail = vi.fn()
  let overview!: ReturnType<typeof useSavesOverview>
  mount(
    defineComponent({
      setup() {
        overview = useSavesOverview(ref(4), fail)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { overview, fail }
}

beforeEach(() => {
  vi.clearAllMocks()
  library.games = [game('a'), game('b'), game('c')]
  library.loaded = true
})

describe('useSavesOverview', () => {
  it('asks for every library game at the slot count, lists those with saves, newest first', async () => {
    bridge.summary.mockResolvedValue([
      summary('a', { lastChange: 1 }),
      summary('b', { inGame: false, quickCount: 0, bytes: 0, lastChange: null }),
      summary('c', { lastChange: 9, quickCount: 2 }),
    ])
    const { overview } = setup()
    await overview.load()
    expect(bridge.summary).toHaveBeenCalledWith(['a', 'b', 'c'], 4)
    expect(overview.withSaves.value.map(g => g.id)).toEqual(['c', 'a'])
    expect(overview.loaded.value).toBe(true)
  })

  it('says the quick saves when there are some, else the in-game save; marks a broken one', async () => {
    bridge.summary.mockResolvedValue([
      summary('a', { quickCount: 3 }),
      summary('b', { broken: true }),
    ])
    const { overview } = setup()
    await overview.load()
    const t = i18n.global.t
    const size = sizeLabel(2048)
    expect(overview.detailOf(game('a'))).toBe(t('saves.metaQuick', { n: 3, size }, 3))
    expect(overview.detailOf(game('b'))).toBe(t('saves.metaInGame', { size }))
    expect(overview.detailOf(game('c'))).toBe('')
    expect(overview.isBroken(game('b'))).toBe(true)
    expect(overview.isBroken(game('a'))).toBe(false)
  })

  it('reports a failure, and drops an answer overtaken by a newer load', async () => {
    let first!: (v: SavesSummary[]) => void
    bridge.summary.mockImplementationOnce(() => new Promise(r => (first = r)))
    bridge.summary.mockResolvedValueOnce([summary('b')])
    const { overview, fail } = setup()
    const older = overview.load()
    await overview.load()
    first([summary('a')])
    await older
    expect(overview.withSaves.value.map(g => g.id)).toEqual(['b'])

    bridge.summary.mockRejectedValueOnce({ code: 'io' })
    await overview.load()
    expect(fail).toHaveBeenCalledWith({ code: 'io' })
    await flush()
  })
})
