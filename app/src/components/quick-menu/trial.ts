// The changes «in prova» (Session level): live in the game, listed in the menu, saved for this
// game or its console, or undone.
import type { SettingEntry, SettingKey, SettingRow } from '@/emu'
import { msg } from '@/lib/settingsModel'
import type { Message } from '@/i18n/message'

/** Each change «in prova» by its value (a choice: «Affiancati») or else by the setting's name. */
export function trialNames(
  rows: Iterable<SettingRow>,
  entry: (key: SettingKey) => SettingEntry | undefined,
): string[] {
  return [...rows]
    .filter(r => r.source === 'session')
    .map(r => {
      // G09's layout card keeps the layout a game opens with as the first of `screens.cycle`.
      if (r.key === 'screens.cycle' && Array.isArray(r.value) && r.value[0])
        return msg.option(r.key, r.value[0])
      return entry(r.key)?.kind.type === 'choice' && typeof r.value === 'string'
        ? msg.option(r.key, r.value)
        : msg.name(r.key)
    })
}

/** The title over the changes: «1 modifica in prova: …» or «N modifiche in prova: …». */
export function trialTitle(names: string[]): Message {
  const list = names.join(', ')
  return names.length === 1
    ? { key: 'qm.trialOne', params: { names: list } }
    : { key: 'qm.trial', params: { n: names.length, names: list } }
}
