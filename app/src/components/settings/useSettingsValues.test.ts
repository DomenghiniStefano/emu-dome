import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import type { SettingEntry, SettingRow, SettingsSchema, SettingsScope } from '@/emu'
import { useSettingsValues } from './useSettingsValues'

const bridge = vi.hoisted(() => ({
  page: vi.fn(),
  recent: vi.fn(),
  set: vi.fn(),
  reset: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    settings: { page: bridge.page, recent: bridge.recent, set: bridge.set, reset: bridge.reset },
  },
}))

const row = (key: string, value: number): SettingRow =>
  ({ key, value, source: 'global', changed: false }) as SettingRow

/** A page load that answers when the test says so. */
function pending() {
  let answer!: (rows: SettingRow[]) => void
  let refuse!: (e: unknown) => void
  const p = new Promise<SettingRow[]>((ok, ko) => ((answer = ok), (refuse = ko)))
  return { p, answer, refuse }
}

const entry = (key: string, page: string, phase = 1) =>
  ({ key, page, phase }) as unknown as SettingEntry

function setup() {
  const schema = ref({ pages: ['video'], currentPhase: 1 } as unknown as SettingsSchema)
  const byKey = ref(
    new Map<string, SettingEntry>([
      ['video.scale', entry('video.scale', 'video')],
      // A key whose prefix is not its page («Avanzate» keeps `perf.`).
      ['perf.audio', entry('perf.audio', 'advanced')],
      ['later.thing', entry('later.thing', 'video', 9)],
    ]),
  )
  const scope = ref<SettingsScope>({ level: 'global' })
  const onError = vi.fn()
  const v = useSettingsValues(schema, byKey, scope, onError)
  return { ...v, scope, onError }
}

beforeEach(() => vi.clearAllMocks())

describe('useSettingsValues', () => {
  it('an older scope’s values answering last are dropped (quick L1 / R1)', async () => {
    const global = pending()
    const game = pending()
    bridge.page.mockReturnValueOnce(global.p).mockReturnValueOnce(game.p)
    const { values, loadAll, scope } = setup()
    void loadAll()
    scope.value = { level: 'game', game: 'gba:01' } as SettingsScope
    await nextTick()
    game.answer([row('video.scale', 3)])
    global.answer([row('video.scale', 1)])
    await new Promise(r => setTimeout(r, 0))
    expect(values.video?.['video.scale']?.value).toBe(3)
  })

  it('a stale load’s error is not shown; a current one’s is', async () => {
    const old = pending()
    bridge.page.mockReturnValueOnce(old.p).mockRejectedValueOnce({ code: 'settingsReadFailed' })
    const { loadAll, onError } = setup()
    void loadAll()
    await loadAll()
    old.refuse({ code: 'noGame' })
    await new Promise(r => setTimeout(r, 0))
    expect(onError.mock.calls).toEqual([[{ code: 'settingsReadFailed' }]])
  })

  it('writes at the level being edited and reports a refused write', async () => {
    bridge.set.mockRejectedValueOnce({ code: 'settingLevelRefused' })
    bridge.reset.mockResolvedValue(undefined)
    // The level changing reloads the pages too.
    bridge.page.mockResolvedValue([])
    const { set, reset, scope, onError } = setup()
    scope.value = { level: 'console', console: 'nds' } as SettingsScope
    await set('video.scale' as never, 2)
    await reset('video.scale' as never)
    expect(bridge.set).toHaveBeenCalledWith('video.scale', scope.value, 2)
    expect(bridge.reset).toHaveBeenCalledWith('video.scale', scope.value)
    expect(onError.mock.calls).toEqual([[{ code: 'settingLevelRefused' }]])
  })

  it('a change anywhere reloads the page the schema puts it on, and the recent list', async () => {
    bridge.page.mockResolvedValue([row('perf.audio', 1)])
    bridge.recent.mockResolvedValue([
      { key: 'perf.audio', level: 'global', at: 1 },
      { key: 'later.thing', level: 'global', at: 2 },
      { key: 'gone.key', level: 'global', at: 3 },
    ])
    const { onChanged, values, recent } = setup()
    onChanged({ key: 'perf.audio', level: 'global' } as never)
    await new Promise(r => setTimeout(r, 0))
    expect(bridge.page).toHaveBeenCalledWith('advanced', { level: 'global' })
    expect(values.advanced?.['perf.audio']?.value).toBe(1)
    // Not shown in this phase, or not in the schema at all: not listed.
    expect(recent.value.map(r => r.key)).toEqual(['perf.audio'])
  })
})
