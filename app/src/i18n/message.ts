// Text built by pure functions, one convention for the whole interface: a function returns a
// `Message` (an i18n key and its parameters), testable without vue-i18n, and the component that
// shows it says it with `useSay()`. A pure function that must look keys up while it builds its
// text (whether a key exists) takes a `Translator` instead: `t` and `te` of `useI18n()`.
// Backend data (error codes, speeds, dates) is turned into text by labels.ts.
import { useI18n } from 'vue-i18n'

/** A message key and its parameters. */
export interface Message {
  key: string
  params?: Record<string, unknown>
}

/** What a pure function needs from vue-i18n when a Message is not enough. */
export interface Translator {
  t: (key: string, params?: Record<string, unknown>) => string
  te: (key: string) => boolean
}

/** Turns Messages into text in the calling component's language. */
export function useSay() {
  const { t } = useI18n()
  return (m: Message) => t(m.key, m.params ?? {})
}
