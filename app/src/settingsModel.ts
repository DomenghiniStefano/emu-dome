// What the settings pages (T01–T19) need beyond the bridge: the schema loaded once, the i18n
// keys of an entry, which rows are shown and why a row cannot be changed at a level.
// Everything comes from the Rust schema (`emu.settings.schema`): nothing here lists settings.
import { ref } from 'vue'
import {
  emu,
  type SettingDevice,
  type SettingEntry,
  type SettingLevel,
  type SettingPlatform,
  type SettingsSchema,
  type SettingValue,
  type VideoMode,
} from './emu'

/** The video path in use (App.vue keeps it current): some game effects exist on one path only. */
export const liveVideoMode = ref<VideoMode>('native')

let schemaPromise: Promise<SettingsSchema> | null = null

/** The schema, asked once per app run (it is static). */
export function loadSchema(): Promise<SettingsSchema> {
  schemaPromise ??= emu.settings.schema()
  return schemaPromise
}

/** i18n key of an option: option ids may hold `.` (speeds), a path separator for vue-i18n. */
export const optionKey = (id: string) => id.replace(/\./g, '_')

export const msg = {
  name: (key: string) => `settings.${key}.name`,
  desc: (key: string) => `settings.${key}.desc`,
  button: (key: string) => `settings.${key}.button`,
  option: (key: string, id: string) => `settings.${key}.option.${optionKey(id)}`,
  optionDesc: (key: string, id: string) => `settings.${key}.optionDesc.${optionKey(id)}`,
  page: (page: string) => `settings.pages.${page}.name`,
  summary: (page: string) => `settings.pages.${page}.summary`,
  section: (page: string, id: string) => `settings.sections.${page}.${id}`,
  group: (id: string) => `settings.groups.${id}`,
}

/** Shown at all: entries of a later roadmap phase stay hidden until then. */
export function isShown(entry: SettingEntry, schema: SettingsSchema): boolean {
  return entry.phase <= schema.currentPhase
}

/**
 * Whether a row is shown with the values of its page (`show_if`, mirror of `ShowIf::holds`):
 * `value(key)` is the other row's value resolved at the level being edited.
 */
export function showIfHolds(
  entry: SettingEntry,
  value: (key: string) => SettingValue | null | undefined,
): boolean {
  const cond = entry.showIf
  if (!cond) return true
  const v = value(cond.key)
  const listed = typeof v === 'string' && cond.values.includes(v)
  return listed !== cond.not
}

/** Whether an entry of `platform` exists on `device` (mirror of `Platform::includes`). */
export function platformIncludes(platform: SettingPlatform, device: SettingDevice): boolean {
  switch (platform) {
    case 'all':
      return true
    case 'desktop':
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

/** Why a row cannot be changed here: i18n key of the reason, or null when it can. */
export function lockReason(
  entry: SettingEntry,
  schema: SettingsSchema,
  level: SettingLevel,
): string | null {
  if (!platformIncludes(entry.platform, schema.device))
    return entry.platform === 'desktop' ? 'settingsUi.lockedDesktop' : 'settingsUi.lockedPhone'
  if (entry.kind.type === 'action' || entry.kind.type === 'apply') return null
  // Session («in prova», quick menu): open to what can be saved per console or per game.
  if (level === 'session')
    return entry.levels.includes('console') || entry.levels.includes('game')
      ? null
      : 'settingsUi.lockedGlobal'
  if (!entry.levels.includes(level))
    return entry.levels.includes('global') ? 'settingsUi.lockedGlobal' : 'settingsUi.lockedScoped'
  return null
}

/** Rows whose control has no backend yet: shown, disabled, with «Arriva con un prossimo aggiornamento». */
export function notYetWired(entry: SettingEntry): boolean {
  // The colour-blind filter is drawn on the webview's canvas; the native video has no filter
  // pass yet (it belongs in video.rs).
  if (entry.key === 'a11y.cblind') return liveVideoMode.value === 'native'
  switch (entry.kind.type) {
    // The one file row with a backend: the background image (`settings_choose_background`).
    case 'file':
      return entry.key !== 'video.surroundImg'
    // `apply` (presets): the bundles and their confirmation are not defined yet.
    case 'action':
    case 'apply':
      return true
    case 'toggle':
    case 'choice':
    case 'multi':
    case 'slider':
    case 'folders':
    case 'colour':
    case 'text':
    case 'bindings':
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
