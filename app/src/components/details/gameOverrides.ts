// L06: the settings a game has a value of its own for, as the tab lists them. Pure: the
// translations come in as a `Translator`.
import type { GameOverride, SettingEntry, SettingKey, SettingsSchema, SettingValue } from '@/emu'
import { isShown, msg } from '@/lib/settingsModel'
import type { Translator } from '@/i18n/message'
import { settingDateTimeLabel } from '@/i18n/labels'

/** One setting of the game, named and with its values in words. */
export interface OverrideRow {
  key: SettingKey
  name: string
  /** The settings page it lives on. */
  where: string
  /** The game's own value. */
  value: string
  /** What every game of the console gets instead; null when the console has nothing set. */
  base: string | null
}

/** A value as the settings pages name it. */
export function settingValueText(e: SettingEntry, v: SettingValue, { t, te }: Translator): string {
  const option = (id: string) => (te(msg.option(e.key, id)) ? t(msg.option(e.key, id)) : id)
  switch (e.kind.type) {
    case 'toggle':
      return t(v === true ? 'details.settings.on' : 'details.settings.off')
    case 'choice':
      return typeof v === 'string' ? option(v) : String(v)
    case 'multi':
      return Array.isArray(v) ? v.map(option).join(', ') : String(v)
    case 'slider':
      return t(`settingsUi.unit.${e.kind.unit}`, { v: Number(v) })
    case 'dateTime':
      return typeof v === 'string' ? settingDateTimeLabel(v) : String(v)
    case 'bindings':
    case 'screenLayout': {
      const k = `settings.${e.key}.value.custom`
      return te(k) ? t(k) : ''
    }
    case 'folders':
      return Array.isArray(v) ? v.join(', ') : String(v)
    case 'file':
    case 'colour':
    case 'text':
    case 'action':
    case 'apply':
      return Array.isArray(v) ? v.join(', ') : String(v)
  }
}

/** The overrides the schema knows and shows now, in their order. */
export function overrideRows(
  overrides: GameOverride[],
  schema: SettingsSchema | null,
  i18n: Translator,
): OverrideRow[] {
  if (!schema) return []
  const byKey = new Map(schema.entries.map(e => [e.key, e]))
  return overrides.flatMap(o => {
    const e = byKey.get(o.key)
    if (!e || !isShown(e, schema)) return []
    return [
      {
        key: o.key,
        name: i18n.t(msg.name(o.key)),
        where: i18n.t(msg.page(o.page)),
        value: settingValueText(e, o.value, i18n),
        base: o.base === null ? null : settingValueText(e, o.base, i18n),
      },
    ]
  })
}
