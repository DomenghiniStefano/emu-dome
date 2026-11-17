// What the settings pages (T01–T20) need beyond the bridge: the schema loaded once, the i18n
// keys of an entry, which rows are shown and why a row cannot be changed at a level.
// Everything comes from the Rust schema (`emu.settings.schema`): nothing here lists settings.
import { emu, type SettingEntry, type SettingLevel, type SettingsSchema } from './emu'

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
  if (entry.platform !== 'all' && entry.platform !== schema.device)
    return entry.platform === 'phone' ? 'settingsUi.lockedPhone' : 'settingsUi.lockedDesktop'
  if (entry.kind.type === 'action') return null
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
  switch (entry.kind.type) {
    case 'action':
    case 'file':
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

/** Plain text for search: lower case, accents removed. */
export function fold(s: string): string {
  return s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
}
