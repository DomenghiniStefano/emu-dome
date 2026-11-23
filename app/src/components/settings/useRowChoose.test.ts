import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { SettingKey } from '@/emu'
import { i18n } from '@/i18n'
import { createToasts, provideToasts, type Toasts } from '@/composables/useToasts'
import { useRowChoose } from './useRowChoose'

const bridge = vi.hoisted(() => ({
  check: vi.fn(),
  exportLog: vi.fn(),
  chooseFile: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    updates: { check: bridge.check },
    about: { exportLog: bridge.exportLog },
    effects: { chooseFile: bridge.chooseFile },
    bios: { choose: vi.fn(), status: vi.fn().mockResolvedValue([]) },
  },
}))

const flush = () => new Promise(r => setTimeout(r, 0))

function setup(key: SettingKey, disabled = false) {
  const page = { openCloud: vi.fn(), choose: vi.fn() }
  const toasts: Toasts = createToasts()
  let choose!: () => Promise<void>
  const Row = defineComponent({
    setup() {
      choose = useRowChoose(
        () => key,
        () => disabled,
        page,
      )
      return () => h('div')
    },
  })
  mount(
    defineComponent({
      setup() {
        provideToasts(toasts)
        return () => h(Row)
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { page, toasts, choose }
}

beforeEach(() => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
})

// The desk's row and the TV's run the same thing (Trello #112: the TV's sent every row to the
// page, which opened the background image's picker for «Check now» and «Licences»).
describe('useRowChoose', () => {
  it('runs an action with a backend here and says when it is done', async () => {
    bridge.exportLog.mockResolvedValue('emudome.log')
    const { page, toasts, choose } = setup('about.log')
    await choose()
    await flush()
    expect(bridge.exportLog).toHaveBeenCalledOnce()
    expect(toasts.list.value.at(-1)?.text).toBe(
      i18n.global.t('settings.about.log.saved', { name: 'emudome.log' }),
    )
    expect(page.choose).not.toHaveBeenCalled()

    const upd = setup('about.upd')
    await upd.choose()
    expect(bridge.check).toHaveBeenCalledOnce()
    expect(upd.page.choose).not.toHaveBeenCalled()
  })

  it('opens the Cloud section for the Google account', async () => {
    const { page, choose } = setup('cloud.acct')
    await choose()
    expect(page.openCloud).toHaveBeenCalledOnce()
    expect(page.choose).not.toHaveBeenCalled()
  })

  it('opens a file row’s own picker here, and leaves every other row to the page', async () => {
    const fx = setup('fx.fxfile')
    await fx.choose()
    expect(bridge.chooseFile).toHaveBeenCalledOnce()
    expect(fx.page.choose).not.toHaveBeenCalled()

    for (const key of ['about.lic', 'video.surroundImg'] as const) {
      const row = setup(key)
      await row.choose()
      expect(row.page.choose).toHaveBeenCalledOnce()
    }
  })

  it('does nothing on a disabled row', async () => {
    for (const key of ['about.log', 'cloud.acct', 'fx.fxfile', 'about.lic'] as const) {
      const { page, choose } = setup(key, true)
      await choose()
      expect(page.choose).not.toHaveBeenCalled()
      expect(page.openCloud).not.toHaveBeenCalled()
    }
    expect(bridge.exportLog).not.toHaveBeenCalled()
    expect(bridge.chooseFile).not.toHaveBeenCalled()
  })
})
