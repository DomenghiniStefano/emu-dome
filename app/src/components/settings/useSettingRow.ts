// The words and options of one setting row (SettingRow): its name, its description at this level
// (why it is locked, the live data some rows show in a neutral form), its button's text and the
// options this device offers. The state itself is settingRowModel.ts's.
import { computed, onMounted, ref, watch, type ComputedRef } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type ConsoleId,
  type SettingEntry,
  type SettingLevel,
  type SettingRow,
  type SettingsSchema,
} from '@/emu'
import { LOCALE } from '@/i18n'
import { biosStatus, refreshBiosStatus } from '@/lib/biosStatus'
import { FILE_PICKERS, msg, optionNotYet } from '@/lib/settingsModel'
import {
  chosenFileName,
  descNotes,
  offeredOptions,
  rowState,
  rowValue,
  type RowState,
} from './settingRowModel'

/** One option of a choice, as the controls show it. */
export interface RowOption {
  id: string
  text: string
  desc: string
  /** Shown, not pickable yet (X05). */
  notYet: boolean
}

export interface SettingRowInput {
  entry: SettingEntry
  row: SettingRow | undefined
  schema: SettingsSchema
  level: SettingLevel
  scoped: boolean
  /** The console of the scope being edited: a row its emulator cannot honour is locked. */
  console?: ConsoleId | null
  /** The page says once why its «Tutti i giochi» rows are grey: the row keeps its description. */
  lockNote?: boolean
}

export function useSettingRow(props: SettingRowInput) {
  const { t, te } = useI18n()
  const key = computed(() => props.entry.key)
  const value = computed(() => rowValue(props.entry, props.row))
  const state: ComputedRef<RowState> = computed(() =>
    rowState(
      props.entry,
      props.row,
      props.schema,
      props.level,
      props.scoped,
      props.console ?? null,
    ),
  )

  // A `bios.*` row with a picker: its status follows its value (the row reset, another file).
  // Unknown (no backend, as in tests): the row keeps saying «built-in».
  if (key.value.startsWith('bios.') && FILE_PICKERS[key.value])
    watch(value, () => void refreshBiosStatus().catch(() => {}), { immediate: true })

  const version = ref('')
  onMounted(async () => {
    if (key.value === 'about.ver') version.value = await emu.appVersion()
  })

  const name = computed(() => {
    if (key.value === 'pad.dev1') return t('settings.pad.dev1.name', { controllerName: '' })
    return t(msg.name(key.value))
  })

  // Rows whose description shows live data the app does not have yet get their neutral form.
  const baseDesc = computed(() => {
    const k = key.value
    const status = `settings.${k}.status.${biosStatus(k)}`
    if (k.startsWith('bios.') && te(status)) return t(status)
    // A number from Rust (`count`: the options a core declares), in the text's plural form.
    const count = props.entry.count
    if (count !== undefined) return t(msg.desc(k), { n: count }, count)
    switch (k) {
      case 'cloud.acct':
        return t('settings.cloud.acct.value.signedOut')
      case 'cloud.space':
      case 'about.upd':
        return t('settingsUi.notYet')
      case 'about.ver':
        return t('settings.about.ver.desc', { version: version.value })
      default:
        return t(msg.desc(k))
    }
  })
  /** The description at this level: why it is locked first, «Arriva…» after when not wired. */
  const desc = computed(() => {
    const { reason, notYet } = descNotes(state.value, !!props.lockNote)
    const own = reason ? `${t(reason)} ${baseDesc.value}`.trim() : baseDesc.value
    return notYet ? `${own} ${t('settingsUi.notYet')}` : own
  })

  /** A file row with no file chosen says so beside its button («Nessuna immagine scelta»). */
  const noFile = computed(() =>
    props.entry.kind.type === 'file' && !value.value && te(`settings.${key.value}.value.none`)
      ? t(`settings.${key.value}.value.none`)
      : '',
  )

  /** A file row with its own picker and a file chosen shows the file's name («crt.slangp»). */
  const fileName = computed(() =>
    props.entry.kind.type === 'file' && FILE_PICKERS[key.value] ? chosenFileName(value.value) : '',
  )

  const buttonText = computed(() => {
    const k = key.value
    if (k === 'cloud.acct') return t('settings.cloud.acct.value.signIn')
    const button = `settings.${k}.buttons.${biosStatus(k) === 'builtin' ? 'use' : 'replace'}`
    if (k.startsWith('bios.') && te(button)) return t(button)
    return te(msg.button(k)) ? t(msg.button(k)) : ''
  })

  /** The options this device offers (a choice may have some for another device only). */
  const options = computed<RowOption[]>(() =>
    offeredOptions(props.entry).map(id => ({
      id,
      // `{language}`: «Come il sistema (Italiano)» of `general.lang`; other options ignore it.
      text: t(msg.option(key.value, id), {
        language: t(`settings.general.lang.option.${LOCALE}`),
      }),
      desc: te(msg.optionDesc(key.value, id)) ? t(msg.optionDesc(key.value, id)) : '',
      notYet: optionNotYet(props.entry, id),
    })),
  )

  return { key, value, state, name, desc, noFile, fileName, buttonText, options }
}
