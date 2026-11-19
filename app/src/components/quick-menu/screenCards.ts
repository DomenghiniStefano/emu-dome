// G09 «Schermi e immagine»: one card per setting, ‹ value ›, then the video method.
import type { SettingEntry, SettingRow, SettingValue, VideoMode } from '@/emu'
import { msg } from '@/settingsModel'
import type { Message } from './slots'

/** The line under a card's value: where the value comes from. */
export interface SourceLine {
  key: string
  /** The level named inside the line («Come in …»), translated first. */
  level?: Message
}

export interface ScreenCard {
  key: string
  name: Message
  /** Null when the setting has no value to show. */
  value: Message | null
  sub: SourceLine
  /** Changed «in prova»: the card is marked. */
  trial: boolean
  /** ‹ and ›: null at the ends, or when the setting cannot change here. */
  prev: (() => void) | null
  next: (() => void) | null
}

/** ‹ › step through `ids` from `current`; null at the ends. */
export function stepper(
  ids: string[],
  current: string,
  set: (id: string) => void,
): Pick<ScreenCard, 'prev' | 'next'> {
  const i = ids.indexOf(current)
  return {
    prev: i > 0 ? () => set(ids[i - 1]!) : null,
    next: i >= 0 && i < ids.length - 1 ? () => set(ids[i + 1]!) : null,
  }
}

/** Where a Session row's value comes from, as the card says it. */
export function sourceLine(row: SettingRow | undefined, consoleName: string): SourceLine {
  switch (row?.source) {
    case 'session':
      return { key: 'qm.source.session' }
    case 'game':
      return { key: 'qm.source.game' }
    case 'console':
      return {
        key: 'settingsUi.sameAs',
        level: { key: 'settingsUi.scope.console', params: { console: consoleName } },
      }
    case 'global':
    case 'default':
    case undefined:
      return { key: 'settingsUi.sameAs', level: { key: 'settingsUi.scope.global' } }
  }
}

export interface SettingCardInput {
  entry: SettingEntry
  row: SettingRow | undefined
  value: SettingValue | null
  /**
   * Why the setting cannot change at the Session level (i18n key, `lockReason`), or null: a
   * locked card has no ‹ › and says why instead of where its value comes from (X05: the screen
   * effects need the Accelerated method).
   */
  reason: string | null
  /** Options not offered right now (`unavailableOptions`). */
  unavailable?: string[]
  consoleName: string
  set: (id: string) => void
}

/**
 * Options that need something chosen elsewhere first: «Effetto da file» only once a file was
 * chosen in Settings (`fx.fxfile`).
 */
export function unavailableOptions(
  key: string,
  value: (key: string) => SettingValue | null,
): string[] {
  return key === 'fx.fxpick' && !value('fx.fxfile') ? ['file'] : []
}

/** The card of a choice setting. */
export function settingCard(c: SettingCardInput): ScreenCard {
  const kind = c.entry.kind
  const skip = [...c.entry.hiddenOptions, ...(c.unavailable ?? [])]
  const ids = kind.type === 'choice' ? kind.options.filter(id => !skip.includes(id)) : []
  const current = String(c.value ?? '')
  return {
    key: c.entry.key,
    name: { key: msg.name(c.entry.key) },
    value: current ? { key: msg.option(c.entry.key, current) } : null,
    sub: c.reason ? { key: c.reason } : sourceLine(c.row, c.consoleName),
    trial: c.row?.source === 'session',
    ...(c.reason ? { prev: null, next: null } : stepper(ids, current, c.set)),
  }
}

export interface VideoPathInput {
  mode: VideoMode
  nativeAvailable: boolean
  phone: boolean
  set: (mode: VideoMode) => void
}

/** The video method (Accelerato / Compatibile): this device only, not a Session setting. */
export function videoPathCard(c: VideoPathInput): ScreenCard {
  const modes: VideoMode[] = c.nativeAvailable ? ['native', 'webview'] : ['webview']
  return {
    key: 'video.path',
    name: { key: 'settings.video.path.name' },
    value: {
      key: `settings.video.path.option.${c.mode === 'native' ? 'accelerated' : 'compatible'}`,
    },
    sub: c.nativeAvailable
      ? { key: `settingsUi.onlyHere.${c.phone ? 'phone' : 'desktop'}` }
      : { key: 'qm.noNative' },
    trial: false,
    ...stepper(modes, c.mode, m => c.set(m as VideoMode)),
  }
}
