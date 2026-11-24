// What the settings pages (T01–T19) need beyond the bridge: the schema loaded once, the i18n
// keys of an entry, which rows are shown and why a row cannot be changed at a level.
// Everything comes from the Rust schema (`emu.settings.schema`): nothing here lists settings.
import {
  emu,
  type ConsoleId,
  type SettingDevice,
  type SettingEntry,
  type SettingKey,
  type SettingLevel,
  type SettingPlatform,
  type SettingRow,
  type SettingsSchema,
  type SettingsScope,
  type SettingValue,
} from '@/emu'
import { nativeAvailable, videoMode } from './videoMode'
import { refreshBiosStatus } from './biosStatus'
import { UPDATE_ROWS, updateUnavailable } from './updates'
import type { Message } from '@/i18n/message'

let schemaPromise: Promise<SettingsSchema> | null = null

/** The schema, asked once per app run (it is static). A read that fails is asked again next time. */
export function loadSchema(): Promise<SettingsSchema> {
  schemaPromise ??= emu.settings.schema().catch((e: unknown) => {
    schemaPromise = null
    throw e
  })
  return schemaPromise
}

/** «Tutti i giochi»: the level every game shares. */
export const GLOBAL: SettingsScope = { level: 'global' }

/**
 * One setting as `scope` resolves it (its value and the level it comes from), read from its page
 * in the schema. Undefined when the schema has no such key. It can throw: the caller reports it.
 */
export async function readSetting(
  key: SettingKey,
  scope: SettingsScope = GLOBAL,
): Promise<SettingRow | undefined> {
  const page = (await loadSchema()).entries.find(e => e.key === key)?.page
  if (!page) return undefined
  return (await emu.settings.page(page, scope)).find(r => r.key === key)
}

/** i18n key of an option: option ids may hold `.` (speeds), a path separator for vue-i18n. */
export const optionKey = (id: string) => id.replace(/\./g, '_')

export const msg = {
  name: (key: string) => `settings.${key}.name`,
  desc: (key: string) => `settings.${key}.desc`,
  /** The part of a description that names one input's keys (#148), shown only for that input. */
  keys: (key: string, input: KeyInput) => `settings.${key}.keys.${input}`,
  button: (key: string) => `settings.${key}.button`,
  option: (key: string, id: string) => `settings.${key}.option.${optionKey(id)}`,
  optionDesc: (key: string, id: string) => `settings.${key}.optionDesc.${optionKey(id)}`,
  page: (page: string) => `settings.pages.${page}.name`,
  summary: (page: string) => `settings.pages.${page}.summary`,
  section: (page: string, id: string) => `settings.sections.${page}.${id}`,
  group: (id: string) => `settings.groups.${id}`,
}

/** An input whose keys a description can name (#148). */
export type KeyInput = 'keyboard' | 'controller'
export const KEY_INPUTS: readonly KeyInput[] = ['keyboard', 'controller']

/**
 * A setting's description with the key parts of the inputs shown (#148, the rule of #118): the
 * text without keys, then each shown input's part (`settings.<key>.keys.<input>`) its entry has.
 * `has` / `text` are vue-i18n's `te` / `t`.
 */
export function descWithKeys(
  key: string,
  base: string,
  shown: Readonly<Record<KeyInput, boolean>>,
  has: (k: string) => boolean,
  text: (k: string) => string,
): string {
  const parts = KEY_INPUTS.filter(i => shown[i] && has(msg.keys(key, i))).map(i =>
    text(msg.keys(key, i)),
  )
  return [base, ...parts].filter(p => p !== '').join(' ')
}

/** Every key part of a setting's description, for the search (a key's name finds its row). */
export function keyWords(
  key: string,
  has: (k: string) => boolean,
  text: (k: string) => string,
): string {
  return descWithKeys(key, '', { keyboard: true, controller: true }, has, text)
}

/** Shown at all: entries of a later roadmap phase stay hidden until then. */
export function isShown(entry: SettingEntry, schema: SettingsSchema): boolean {
  // `touch.tscreens` has no row: it is edited with the `touch.tedit` row's layouts.
  return entry.phase <= schema.currentPhase && entry.kind.type !== 'touchScreens'
}

/**
 * Whether a row is shown with the values of its page (`show_if`, mirror of `ShowIf::holds`):
 * `value(key)` is the other row's value resolved at the level being edited.
 */
/** A choice by its option id, a switch by `true` / `false` (as `ShowIf::holds`). */
const matches = (values: readonly string[], v: string | boolean) =>
  typeof v === 'boolean' ? values.includes(v ? 'true' : 'false') : values.includes(v)

export function showIfHolds(
  entry: SettingEntry,
  value: (key: SettingKey) => SettingValue | null | undefined,
): boolean {
  const cond = entry.showIf
  if (!cond) return true
  const v = value(cond.key)
  const listed = typeof v === 'string' || typeof v === 'boolean' ? matches(cond.values, v) : false
  return listed !== cond.not
}

/** Whether an entry of `platform` exists on `device` (mirror of `Platform::includes`). */
export function platformIncludes(platform: SettingPlatform, device: SettingDevice): boolean {
  switch (platform) {
    case 'all':
      return true
    // A keyboard row is not in the phone's schema at all (`Platform::listed_on`, #118).
    case 'desktop':
    case 'keyboard':
      return device === 'desktop'
    case 'phone':
    case 'touch':
      return device === 'phone'
  }
}

/** Levels the page's shown entries accept, among Global and Console. */
export function pageLevels(entries: SettingEntry[]): Set<SettingLevel> {
  const out = new Set<SettingLevel>()
  for (const e of entries)
    for (const l of e.levels) if (l === 'global' || l === 'console') out.add(l)
  return out
}

/** A row whose button opens a layout editor (the screens' C09, the touch controls' C06). */
export const opensLayoutEditor = (entry: SettingEntry) =>
  entry.kind.type === 'screenLayout' || entry.kind.type === 'touchLayout'

/**
 * Why a row cannot be changed here:i18n key of the reason, or null when it can. `console`: the
 * console of the scope being edited (null at Global, or when the caller has none).
 */
export function lockReason(
  entry: SettingEntry,
  schema: SettingsSchema,
  level: SettingLevel,
  console: ConsoleId | null = null,
): string | null {
  if (!platformIncludes(entry.platform, schema.device))
    return platformIncludes(entry.platform, 'desktop')
      ? 'settingsUi.lockedDesktop'
      : 'settingsUi.lockedPhone'
  // A build without the update key cannot update itself (Trello #44, `emudome_platform::updater`).
  if (UPDATE_ROWS.includes(entry.key) && updateUnavailable.value)
    return 'settingsUi.lockedNoUpdates'
  // The console's emulator has no equivalent (`CoreProfile::unsupported_settings`).
  if (console && entry.unsupportedConsoles.includes(console)) return 'settingsUi.lockedConsole'
  // X05: the screen effects exist only on the Accelerated method.
  if (entry.needsNativeVideo && videoMode.value !== 'native')
    return nativeAvailable.value ? 'settingsUi.lockedNeedsNative' : 'qm.noNative'
  if (entry.kind.type === 'action' || entry.kind.type === 'apply') return null
  // A layout editor's row (C09, C06; Trello #111) at «Tutti i giochi» asks for the console first
  // (settings/layoutEditors.ts), then saves at that console's level.
  if (level === 'global' && opensLayoutEditor(entry)) return null
  // Session («in prova», quick menu): open to what can be saved per console or per game.
  if (level === 'session')
    return entry.levels.includes('console') || entry.levels.includes('game')
      ? null
      : 'settingsUi.lockedGlobal'
  if (!entry.levels.includes(level))
    return entry.levels.includes('global') ? 'settingsUi.lockedGlobal' : 'settingsUi.lockedScoped'
  return null
}

/** The `bios.*` rows with a backend (`emudome_core::bios` tables): their picker checks the file. */
const BIOS_ROWS: SettingKey[] = [
  'bios.bgb',
  'bios.bgbc',
  'bios.bgba',
  'bios.bds7',
  'bios.bds9',
  'bios.bdsfw',
  'bios.bdsi7',
  'bios.bdsi9',
  'bios.bdsifw',
  'bios.bdsi',
]

/**
 * File rows that work: the picker the row's button opens, which stores the chosen file itself.
 * Resolves what it chose (a path, a status), or null when cancelled.
 */
export const FILE_PICKERS: Partial<Record<string, () => Promise<string | null>>> = {
  'fx.fxfile': () => emu.effects.chooseFile(),
  ...Object.fromEntries(
    BIOS_ROWS.map(key => [
      key,
      async () => {
        const status = await emu.bios.choose(key)
        // The same file picked again keeps the value: the status is read here, not on change.
        await refreshBiosStatus()
        return status
      },
    ]),
  ),
}

/**
 * Action rows that work (`about.*`, Trello #44): what the row's button does. Resolves a message
 * to show once done, or null. A link row the Settings section opens itself (`about.lic`, its
 * licences page) is not here.
 */
export const ROW_ACTIONS: Partial<Record<string, () => Promise<Message | null>>> = {
  // The status that follows arrives on `updates://changed` (composables/useUpdates.ts).
  'about.upd': async () => {
    await emu.updates.check()
    return null
  },
  'about.log': async () => {
    const name = await emu.about.exportLog()
    return name ? { key: 'settings.about.log.saved', params: { name } } : null
  },
}

/**
 * Rows whose control has no backend yet: shown, disabled, with «Arriva con un prossimo
 * aggiornamento». Which entries nothing reads is the catalogue's `wired` (Rust, its list held by a
 * test); the rest is what only the interface knows: the pickers it has and the video path.
 */
export function notYetWired(entry: SettingEntry): boolean {
  if (!entry.wired) return true
  // The colour-blind filter is drawn on the webview's canvas; the native video has no filter
  // pass yet (it belongs in video.rs).
  if (entry.key === 'a11y.cblind') return videoMode.value === 'native'
  switch (entry.kind.type) {
    // File rows with a backend: their own picker (FILE_PICKERS), or the background image,
    // whose picker the page opens at its scope (`settings_choose_background`).
    case 'file':
      return !FILE_PICKERS[entry.key] && entry.key !== 'video.surroundImg'
    case 'action':
    case 'apply':
    case 'toggle':
    case 'choice':
    case 'multi':
    case 'slider':
    case 'folders':
    case 'colour':
    case 'text':
    case 'dateTime':
    case 'bindings':
    case 'screenLayout':
    case 'touchLayout':
    case 'touchScreens':
      // A screen layout: edited in the C09 screens editor (components/screen-editor, #28); the
      // touch controls in the C06 editor (components/touch, #29).
      return false
  }
}

/**
 * Options of a choice that are shown but cannot be picked yet (X05: disabled, with «Arriva con un
 * prossimo aggiornamento»). «Gioco sfocato» needs the game's frames blurred behind it, which the
 * native video does not draw yet.
 */
export function optionNotYet(entry: SettingEntry, id: string): boolean {
  return entry.key === 'video.surround' && id === 'blur'
}

/** Plain text for search: lower case, accents removed. */
export function fold(s: string): string {
  return s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
}
