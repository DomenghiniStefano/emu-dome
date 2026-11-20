import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import type { SettingEntry, SettingRow, SettingsSchema, SettingsScope } from '@/emu'
import { useSettingsValues } from './useSettingsValues'

const bridge = vi.hoisted(() => ({ page: vi.fn(), recent: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { settings: { page: bridge.page, recent: bridge.recent } } }))

const row = (key: string, value: number): SettingRow =>
  ({ key, value, source: 'global', changed: false }) as SettingRow

/** A page load that answers when the test says so. */
function pending() {
  let answer!: (rows: SettingRow[]) => void
  let refuse!: (e: unknown) => void
  const p = new Promise<SettingRow[]>((ok, ko) => ((answer = ok), (refuse = ko)))
  return { p, answer, refuse }
}

function setup() {
  const schema = ref({ pages: ['video'] } as unknown as SettingsSchema)
  const byKey = ref(new Map<string, SettingEntry>())
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
})
