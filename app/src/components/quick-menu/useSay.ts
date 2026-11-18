// Messages built by the pure functions (key and parameters), turned into text.
import { useI18n } from 'vue-i18n'
import type { Message } from './slots'

export function useSay() {
  const { t } = useI18n()
  return (m: Message) => t(m.key, m.params ?? {})
}
