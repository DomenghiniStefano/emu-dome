// The state of one setting row (SettingRow), as pure functions: what the row shows at the level
// being edited, and the small decisions of its controls. No i18n here: useSettingRow turns these
// into text.
import type {
  SettingEntry,
  SettingLevel,
  SettingRow,
  SettingsSchema,
  SettingValue,
} from '../../emu'
import { lockReason, notYetWired } from '../../settingsModel'
import { BUILTIN_BACKGROUNDS, SURROUND_CARDS } from '../../surround'

export interface RowState {
  /** i18n key of why the row cannot be changed at this level, or null. */
  reason: string | null
  /** The control has no backend yet («Arriva con un prossimo aggiornamento»). */
  notYet: boolean
  /** Shown faded, its controls disabled: locked here, or not wired yet. */
  disabled: boolean
  /** Changed at this level (not merely different from the default): dot, «Modificato», reset. */
  changedHere: boolean
  /** The level the value comes from when it is inherited and the page has a scope bar. */
  inheritedFrom: SettingLevel | null
  /** The level a reset falls back to, when it is not the default: «Usa il valore di …». */
  fallback: SettingLevel | null
}

export function rowState(
  entry: SettingEntry,
  row: SettingRow | undefined,
  schema: SettingsSchema,
  level: SettingLevel,
  scoped: boolean,
): RowState {
  const reason = lockReason(entry, schema, level)
  const notYet = notYetWired(entry)
  const source = row?.source
  return {
    reason,
    notYet,
    disabled: reason !== null || notYet,
    changedHere: reason === null && source === level,
    inheritedFrom: scoped && source && source !== 'default' && source !== level ? source : null,
    fallback: level !== 'global' && entry.levels.includes('global') ? 'global' : null,
  }
}

/** The value a row shows: the stored one, else the schema's default. */
export function rowValue(entry: SettingEntry, row: SettingRow | undefined): SettingValue | null {
  return row?.value ?? entry.default
}

/** The name of a chosen file (its path's last part), or '' when the value is not a path. */
export function chosenFileName(value: SettingValue | null): string {
  return typeof value === 'string' ? (value.split(/[\\/]/).pop() ?? '') : ''
}

/** The chosen ids of a multiple choice, in order (anything else counts as none chosen). */
export function chosenList(value: SettingValue | null): string[] {
  return Array.isArray(value) ? value : []
}

/**
 * A chip of a multiple choice turned on or off. Turning one on puts it where it is among the
 * options, so the list keeps the options' order.
 */
export function toggleChosen(chosen: string[], options: string[], id: string): string[] {
  return chosen.includes(id)
    ? chosen.filter(x => x !== id)
    : options.filter(x => x === id || chosen.includes(x))
}

// The previews of choice cards: the background ones come from surround.ts, the others take a
// colour from this round of hues (the board's), by position.
const CARD_PREVIEWS: Record<string, string> = {
  ...Object.fromEntries(
    Object.entries(SURROUND_CARDS).map(([id, bg]) => [`video.surround.${id}`, bg]),
  ),
  ...Object.fromEntries(
    Object.entries(BUILTIN_BACKGROUNDS).map(([id, bg]) => [`video.surroundSet.${id}`, bg]),
  ),
}
const PREVIEW_HUES = ['#2f5d8a', '#4f6b2e', '#3d6b3a', '#8a3b2f', '#5b3f8c', '#245f7a']

/** The preview of option `id`, the `index`-th card of setting `key`. */
export function cardPreview(key: string, id: string, index: number): string {
  return CARD_PREVIEWS[`${key}.${id}`] ?? PREVIEW_HUES[index % PREVIEW_HUES.length] ?? ''
}

/** i18n key of a swatch's name (`settings.general.accent.swatch.7152f7`). */
export function swatchKey(key: string, hex: string): string {
  return `settings.${key}.swatch.${hex.replace('#', '')}`
}
