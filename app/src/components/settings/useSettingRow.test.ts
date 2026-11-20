import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, reactive } from 'vue'
import type { SettingEntry, SettingRow, SettingsSchema } from '@/emu'
import { i18n } from '@/i18n'
import { refreshBiosStatus } from '@/lib/biosStatus'
import { useSettingRow, type SettingRowInput } from './useSettingRow'

const bridge = vi.hoisted(() => ({ appVersion: vi.fn(), biosStatus: vi.fn() }))
vi.mock('@/emu', () => ({
  emu: { appVersion: bridge.appVersion, bios: { status: bridge.biosStatus } },
}))

const t = (key: string, params: Record<string, unknown> = {}) => i18n.global.t(key, params)
const flush = () => new Promise(r => setTimeout(r, 0))

const entry = (key: string, over: Partial<SettingEntry> = {}): SettingEntry => ({
  key,
  page: key.split('.')[0] ?? '',
  kind: { type: 'toggle' },
  default: false,
  levels: ['default', 'global', 'console', 'game', 'session'],
  perDevice: false,
  platform: 'all',
  phase: 0,
  hiddenOptions: [],
  synonyms: [],
  mostUsed: false,
  needsNativeVideo: false,
  ...over,
})
const schema: SettingsSchema = {
  device: 'desktop',
  currentPhase: 1,
  pages: [],
  groups: [],
  sections: [],
  entries: [],
}
const fileEntry = (key: string) => entry(key, { kind: { type: 'file' }, default: '' })
const globalOnly = (key: string) => entry(key, { levels: ['default', 'global'] })

function setup(e: SettingEntry, over: Partial<SettingRowInput> = {}) {
  const props = reactive<SettingRowInput>({
    entry: e,
    row: undefined,
    schema,
    level: 'global',
    scoped: false,
    ...over,
  })
  let row!: ReturnType<typeof useSettingRow>
  mount(
    defineComponent({
      setup() {
        row = useSettingRow(props)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { row, props }
}

beforeEach(async () => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
  bridge.appVersion.mockResolvedValue('0.4.2')
  bridge.biosStatus.mockResolvedValue([])
  await refreshBiosStatus()
})

describe('useSettingRow', () => {
  it('names the row and describes it from its messages', () => {
    const { row } = setup(entry('general.theme'))
    expect(row.name.value).toBe(t('settings.general.theme.name'))
    expect(row.desc.value).toBe(t('settings.general.theme.desc'))
    expect(row.buttonText.value).toBe('')
  })

  it('says why a row is locked at this level before its own words', () => {
    const { row, props } = setup(globalOnly('general.theme'), { level: 'console' })
    const own = t('settings.general.theme.desc')
    expect(row.desc.value).toBe(`${t('settingsUi.lockedGlobal')} ${own}`)
    expect(row.state.value.disabled).toBe(true)
    props.level = 'global'
    expect(row.desc.value).toBe(own)
  })

  it('leaves out «only for all games» when the page says it once, not other reasons', () => {
    const { row } = setup(globalOnly('general.theme'), { level: 'console', lockNote: true })
    expect(row.desc.value).toBe(t('settings.general.theme.desc'))

    const phoneOnly = entry('general.theme', { platform: 'phone' })
    const other = setup(phoneOnly, { lockNote: true }).row
    expect(other.desc.value.startsWith(t('settingsUi.lockedPhone'))).toBe(true)
  })

  it('adds «Arriva…» to a row with no backend yet, not to a locked one', () => {
    const { row } = setup(fileEntry('online.wfc'))
    expect(row.desc.value).toBe(`${t('settings.online.wfc.desc')} ${t('settingsUi.notYet')}`)
    const locked = setup(
      entry('online.wfc', { kind: { type: 'file' }, default: '', levels: ['default', 'global'] }),
      { level: 'console' },
    ).row
    expect(locked.desc.value).not.toContain(t('settingsUi.notYet'))
  })

  it('gives rows with live data their neutral words: built-in BIOS, signed out, not yet', () => {
    expect(setup(fileEntry('bios.bgba')).row.desc.value).toContain(
      t('settings.bios.bgba.status.builtin'),
    )
    expect(setup(entry('cloud.acct')).row.desc.value).toBe(t('settings.cloud.acct.value.signedOut'))
    expect(setup(entry('cloud.space')).row.desc.value).toBe(t('settingsUi.notYet'))
  })

  it('shows the app’s version once the bridge answers; other rows never ask', async () => {
    const { row } = setup(entry('about.ver'))
    await flush()
    expect(bridge.appVersion).toHaveBeenCalledOnce()
    expect(row.desc.value).toBe(t('settings.about.ver.desc', { version: '0.4.2' }))
    setup(entry('about.lic'))
    await flush()
    expect(bridge.appVersion).toHaveBeenCalledOnce()
  })

  it('a file row says no file is chosen, then shows the chosen file’s name when it has a picker', () => {
    const { row, props } = setup(fileEntry('fx.fxfile'))
    expect(row.noFile.value).toBe(t('settings.fx.fxfile.value.none'))
    expect(row.fileName.value).toBe('')
    props.row = { key: 'fx.fxfile', value: 'C:\\fx\\crt.slangp', source: 'global', changed: true }
    expect(row.noFile.value).toBe('')
    expect(row.fileName.value).toBe('crt.slangp')
  })

  it('a file row without its own picker never shows a file name', () => {
    const row: SettingRow = {
      key: 'video.surroundImg',
      value: 'C:/bg.png',
      source: 'global',
      changed: true,
    }
    expect(setup(fileEntry('video.surroundImg'), { row }).row.fileName.value).toBe('')
  })

  it('words the button: sign in, use your BIOS file, the row’s own, or none', () => {
    expect(setup(entry('cloud.acct')).row.buttonText.value).toBe(
      t('settings.cloud.acct.value.signIn'),
    )
    expect(setup(fileEntry('bios.bgba')).row.buttonText.value).toBe(
      t('settings.bios.bgba.buttons.use'),
    )
    expect(setup(entry('saves.import')).row.buttonText.value).toBe(
      t('settings.saves.import.button'),
    )
  })

  it('a BIOS row follows the backend’s status: verified, not recognised, back to built-in', async () => {
    bridge.biosStatus.mockResolvedValue([{ key: 'bios.bgba', status: 'verified' }])
    const { row, props } = setup(fileEntry('bios.bgba'))
    await flush()
    expect(bridge.biosStatus).toHaveBeenCalled()
    expect(row.desc.value).toContain(t('settings.bios.bgba.status.verified'))
    expect(row.buttonText.value).toBe(t('settings.bios.bgba.buttons.replace'))

    // The row's value changes (another file): the status is read again.
    bridge.biosStatus.mockResolvedValue([{ key: 'bios.bgba', status: 'unrecognised' }])
    props.row = { key: 'bios.bgba', value: 'C:/mine.bin', source: 'global', changed: true }
    await flush()
    expect(row.desc.value).toContain(t('settings.bios.bgba.status.unrecognised'))
    expect(row.buttonText.value).toBe(t('settings.bios.bgba.buttons.replace'))

    // Reset: no file, the built-in replacement again.
    bridge.biosStatus.mockResolvedValue([{ key: 'bios.bgba', status: 'builtin' }])
    props.row = { key: 'bios.bgba', value: '', source: 'default', changed: false }
    await flush()
    expect(row.desc.value).toContain(t('settings.bios.bgba.status.builtin'))
    expect(row.buttonText.value).toBe(t('settings.bios.bgba.buttons.use'))
  })

  it('lists the options this device offers, with their words and the ones not ready yet', () => {
    const surround = entry('video.surround', {
      kind: {
        type: 'choice',
        style: 'cards',
        options: ['black', 'cover', 'blur'],
        moreAtRuntime: false,
      },
      default: 'black',
      hiddenOptions: ['cover'],
    })
    const { row } = setup(surround)
    expect(row.options.value).toEqual([
      {
        id: 'black',
        text: t('settings.video.surround.option.black'),
        desc: t('settings.video.surround.optionDesc.black'),
        notYet: false,
      },
      {
        id: 'blur',
        text: t('settings.video.surround.option.blur'),
        desc: t('settings.video.surround.optionDesc.blur'),
        notYet: true,
      },
    ])
  })

  it('an option without a description of its own gets none', () => {
    const theme = entry('general.theme', {
      kind: { type: 'choice', style: 'segments', options: ['dark'], moreAtRuntime: false },
    })
    expect(setup(theme).row.options.value[0]?.desc).toBe('')
  })
})
