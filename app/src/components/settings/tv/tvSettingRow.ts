// Settings in TV mode (#30): how each row of the schema is changed from the sofa, as pure
// functions. Large rows; ◀ ▶ change a short choice, a switch, a slider or a colour
// in place; A opens the long choices in a full-screen sheet (TvChoiceSheet), toggles a switch, or
// runs a row's action as the desk's button does. Kinds the TV layout does not draw (a text, the
// game folders, a kind added later) keep the desk's row (SettingRow), scaled up.
// Everything comes from the schema's kinds: nothing here lists settings.
import type { SettingEntry, SettingValue } from '@/emu'
import { offeredOptions } from '@/components/settings/settingRowModel'
import type { EditLevel } from '@/components/settings/settingsPage'

/** A choice with more options than this opens the sheet instead of stepping in place. */
export const MAX_IN_PLACE = 5

/**
 * How a row is changed in TV:
 * - `toggle`: A and ◀ ▶ switch it;
 * - `step`: ◀ ▶ step its value in place (a short choice, a slider, a few colours); A opens the
 *   sheet for a choice or a colour, does nothing on a slider;
 * - `sheet`: A opens the sheet (a long choice, one whose options grow at run time, a multiple
 *   choice, presets);
 * - `run`: A runs the row's action as the desk's button does (a file, an action, a binding list,
 *   a screen layout);
 * - `desk`: the desk's row, scaled up (a text field, the game folders, any other kind).
 */
export type TvEdit = 'toggle' | 'step' | 'sheet' | 'run' | 'desk'

export function tvEdit(entry: SettingEntry): TvEdit {
  const kind = entry.kind
  if (kind.type === 'toggle') return 'toggle'
  if (kind.type === 'slider') return 'step'
  if (kind.type === 'choice')
    return kind.moreAtRuntime || offeredOptions(entry).length > MAX_IN_PLACE ? 'sheet' : 'step'
  if (kind.type === 'colour') return kind.swatches.length > MAX_IN_PLACE ? 'sheet' : 'step'
  if (kind.type === 'multi' || kind.type === 'apply') return 'sheet'
  if (
    kind.type === 'file' ||
    kind.type === 'action' ||
    kind.type === 'bindings' ||
    kind.type === 'screenLayout'
  )
    return 'run'
  return 'desk'
}

/** Whether A on the row opens the sheet (the long choices, and the short ones to read them). */
export function opensSheet(entry: SettingEntry): boolean {
  const edit = tvEdit(entry)
  if (edit === 'sheet') return true
  return edit === 'step' && entry.kind.type !== 'slider'
}

/** One option as the steps and the sheet see it. */
export interface StepOption {
  id: string
  /** Shown, not pickable yet (X05). */
  notYet: boolean
}

/**
 * The option ◀ (`-1`) or ▶ (`1`) picks from `current`, round the ends, skipping those not
 * pickable yet; null when no other option can be picked. From a value not among the options
 * (none chosen), ▶ picks the first pickable one and ◀ the last.
 */
export function stepOption(options: StepOption[], current: string, dir: -1 | 1): string | null {
  const n = options.length
  if (n === 0) return null
  const at = options.findIndex(o => o.id === current)
  const start = at < 0 ? (dir === 1 ? -1 : n) : at
  for (let i = 1; i <= n; i++) {
    const o = options[(((start + dir * i) % n) + n) % n]
    if (!o) continue
    if (o.id === current) return null
    if (!o.notYet) return o.id
  }
  return null
}

/**
 * A slider's value one step down (`-1`) or up (`1`), held within its range and on its steps
 * from `min`; null at the end it moves towards.
 */
export function stepSlider(
  value: number,
  range: { min: number; max: number; step: number },
  dir: -1 | 1,
): number | null {
  const { min, max, step } = range
  if (step <= 0 || max < min) return null
  const onGrid = Math.round((value - min) / step)
  const next = Math.min(max, Math.max(min, min + (onGrid + dir) * step))
  // Rounded to the step's decimals, so 0.1 steps do not drift (0.30000000000000004).
  const decimals = (String(step).split('.')[1] ?? '').length
  const rounded = Number(next.toFixed(decimals))
  return rounded === value ? null : rounded
}

/** The value a toggle switches to. */
export const toggled = (value: SettingValue | null): boolean => value !== true

/** The scope sheet's choices: «Tutti i giochi», one per console offered, «Solo questo gioco». */
export interface ScopeChoice {
  /** `global`, `game`, or a console's id (the console level for it). */
  id: string
  level: EditLevel
}

/** The scope sheet's options, from the levels the page offers and the consoles in the library. */
export function scopeChoices(levels: EditLevel[], consoles: string[]): ScopeChoice[] {
  const out: ScopeChoice[] = [{ id: 'global', level: 'global' }]
  if (levels.includes('console')) for (const c of consoles) out.push({ id: c, level: 'console' })
  if (levels.includes('game')) out.push({ id: 'game', level: 'game' })
  return out
}

/** The scope sheet's chosen option: the level in effect, a console's by its id. */
export function scopeChoiceAt(level: EditLevel, console: string | null): string {
  if (level === 'console' && console) return console
  return level === 'game' ? 'game' : 'global'
}
