import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { GameInfo, SettingRow } from '@/emu'
import type { Toasts } from '@/composables/useToasts'
import { useGameSettings } from './useGameSettings'

const bridge = vi.hoisted(() => ({ schema: vi.fn(), page: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { settings: { schema: bridge.schema, page: bridge.page } } }))

const rows = (toggle: boolean) => [{ key: 'a11y.toggle', value: toggle }] as SettingRow[]

beforeEach(() => {
  vi.clearAllMocks()
  bridge.schema.mockResolvedValue({
    entries: [
      { key: 'a11y.toggle', page: 'a11y' },
      { key: 'speed.ffval', page: 'speed' },
      // A key whose prefix is not its page («Avanzate» keeps `perf.` keys).
      { key: 'speed.hidden', page: 'advanced' },
      { key: 'perf.thing', page: 'speed' },
    ],
  })
})

function setup() {
  const game = ref<GameInfo | null>({ title: 'MINISH' } as GameInfo)
  const toasts = { failWith: vi.fn() }
  return { game, toasts, ...useGameSettings(game, toasts as unknown as Toasts) }
}

describe('useGameSettings', () => {
  it('a change concerns them by the page the schema gives the key, not by its prefix', async () => {
    bridge.page.mockResolvedValue([])
    const { load, concerns } = setup()
    await load()
    expect(concerns('a11y.toggle')).toBe(true)
    expect(concerns('perf.thing')).toBe(true)
    expect(concerns('speed.hidden')).toBe(false)
    expect(concerns('video.scale')).toBe(false)
  })

  it('an older load answering last is dropped', async () => {
    let answerOld!: (r: SettingRow[]) => void
    // The first load reads three pages: the first answers late, the others at once.
    bridge.page
      .mockImplementationOnce(() => new Promise<SettingRow[]>(r => (answerOld = r)))
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
    const { load, settings } = setup()
    const old = load()
    await new Promise(r => setTimeout(r, 0))
    bridge.page.mockResolvedValue(rows(true))
    await load()
    answerOld(rows(false))
    await old
    expect(settings.value.rewindToggle).toBe(true)
  })
})
