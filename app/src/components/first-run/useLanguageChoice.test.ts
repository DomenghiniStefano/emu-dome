import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import { i18n, LOCALE } from '@/i18n'
import { useLanguageChoice } from './useLanguageChoice'

// `readSetting` finds the key's page in the schema, then reads that page at «Tutti i giochi».
const bridge = vi.hoisted(() => ({ schema: vi.fn(), page: vi.fn(), set: vi.fn() }))
vi.mock('@/emu', () => ({
  emu: { settings: { schema: bridge.schema, page: bridge.page, set: bridge.set } },
}))

function setup() {
  const onError = vi.fn()
  let choice!: ReturnType<typeof useLanguageChoice>
  mount(
    defineComponent({
      setup() {
        choice = useLanguageChoice(onError)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { choice, onError }
}

beforeEach(() => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
  bridge.schema.mockResolvedValue({ entries: [{ key: 'general.lang', page: 'general' }] })
  bridge.page.mockResolvedValue([{ key: 'general.lang', value: 'it', source: 'global' }])
  bridge.set.mockResolvedValue(undefined)
})

describe('useLanguageChoice', () => {
  it('offers the system’s language, named in the language it resolves to, then Italian and English', () => {
    const { choice } = setup()
    const { t } = i18n.global
    const system = t('settings.general.lang.option.system', {
      language: t(`settings.general.lang.option.${LOCALE}`),
    })
    expect(choice.options.value.map(o => o.id)).toEqual(['system', 'it', 'en'])
    expect(choice.options.value[0]?.text).toBe(system)
    expect(system).toContain(t(`settings.general.lang.option.${LOCALE}`))
  })

  it('starts on the system’s language, then shows the one saved', async () => {
    const { choice } = setup()
    expect(choice.lang.value).toBe('system')
    await choice.load()
    expect(bridge.page).toHaveBeenCalledWith('general', { level: 'global' })
    expect(choice.lang.value).toBe('it')
  })

  it('keeps the system’s language when nothing usable is saved', async () => {
    const { choice } = setup()
    bridge.page.mockResolvedValueOnce([{ key: 'general.lang', value: null, source: 'default' }])
    await choice.load()
    expect(choice.lang.value).toBe('system')
  })

  it('lets a failed read reach the caller, who reports it', async () => {
    const { choice } = setup()
    bridge.page.mockRejectedValueOnce({ code: 'io' })
    await expect(choice.load()).rejects.toEqual({ code: 'io' })
  })

  it('saves the choice at «Tutti i giochi»; a failed save is reported and the choice stays', async () => {
    const { choice, onError } = setup()
    await choice.choose('en')
    expect(bridge.set).toHaveBeenCalledWith('general.lang', { level: 'global' }, 'en')
    expect(onError).not.toHaveBeenCalled()

    bridge.set.mockRejectedValueOnce({ code: 'io' })
    await choice.choose('it')
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
    expect(choice.lang.value).toBe('it')
  })
})
