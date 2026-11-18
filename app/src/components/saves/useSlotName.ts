// A quick save slot's name: «Automatico» for the auto quick save (slot 0), else «Slot N».
import { useI18n } from 'vue-i18n'
import { AUTO_SLOT } from '../../emu'

export function useSlotName(): (slot: number) => string {
  const { t } = useI18n()
  return slot => (slot === AUTO_SLOT ? t('qm.auto') : t('qm.slot', { slot }))
}
