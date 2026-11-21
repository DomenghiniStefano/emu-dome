import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { SettingKey, SettingRow, SettingsChanged, SettingsSchema } from '@/emu'
import { i18n } from '@/i18n'
import { useTrialSession } from './useTrialSession'

const bridge = vi.hoisted(() => ({
  page: vi.fn(),
  set: vi.fn(),
  commit: vi.fn(),
  changed: [] as Array<(e: SettingsChanged) => void>,
}))
vi.mock('@/emu', () => ({
  emu: {
    settings: {
      schema: () =>
        Promise.resolve({
          currentPhase: 1,
          device: 'desktop',
          entries: [
            {
              key: 'video.filter',
              page: 'video',
              phase: 1,
              default: 'light',
              kind: { type: 'choice', options: ['none', 'light', 'soft'] },
            },
            { key: 'speed.ffval', page: 'speed', phase: 1, default: '4', kind: { type: 'slider' } },
          ],
        } as unknown as SettingsSchema),
      page: bridge.page,
      set: bridge.set,
      commitSession: bridge.commit,
      discardSession: vi.fn(),
      onChanged: (cb: (e: SettingsChanged) => void) => {
        bridge.changed.push(cb)
        return Promise.resolve(() => {})
      },
    },
  },
}))

const row = (key: string, value: SettingRow['value'], source: SettingRow['source']) =>
  ({ key, value, source, changed: source === 'session' }) as SettingRow
const flush = () => new Promise(r => setTimeout(r, 0))

function setup() {
  const onError = vi.fn()
  let trial!: ReturnType<typeof useTrialSession>
  mount(
    defineComponent({
      setup() {
        trial = useTrialSession(onError)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { trial, onError }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.changed.length = 0
  i18n.global.locale.value = 'en'
  bridge.page.mockImplementation(async (page: string) =>
    page === 'video' ? [row('video.filter', 'soft', 'session')] : [],
  )
})

describe('useTrialSession', () => {
  it('reads the Session rows; a value falls back to the default', async () => {
    const { trial } = setup()
    await trial.start()
    expect(bridge.page).toHaveBeenCalledWith('video', { level: 'session' })
    expect(trial.value('video.filter')).toBe('soft')
    expect(trial.value('speed.ffval')).toBe('4')
    expect(trial.value('nothing' as SettingKey)).toBeNull()
  })

  it('titles the changes «in prova», none without any', async () => {
    const { trial } = setup()
    expect(trial.title.value).toBe('')
    await trial.start()
    expect(trial.title.value).not.toBe('')
    expect(trial.names.value).toHaveLength(1)
  })

  it('reads the rows again on any settings change while the menu is open', async () => {
    const { trial } = setup()
    await trial.start()
    await flush()
    bridge.page.mockResolvedValue([])
    bridge.changed.forEach(cb => cb({ key: 'video.filter', level: 'session' }))
    await flush()
    expect(trial.title.value).toBe('')
  })

  it('reports a failure as it was thrown', async () => {
    const { trial, onError } = setup()
    bridge.commit.mockRejectedValueOnce({ code: 'io' })
    await trial.commit('game')
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
  })
})
