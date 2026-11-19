// The Saves section's settings at «Tutti i giochi»: how many quick save slots there are
// (`saves.slots`) and how often the auto quick save runs (`saves.autoq`, E03's sentence).
import { ref } from 'vue'
import type { ErrorSink } from '../../i18n/labels'
import { readSetting } from '../../settingsModel'
import { DEFAULT_SLOT_COUNT, loadSlotCount } from '../details/gameSaves'

export function useSavesSettings(fail: ErrorSink) {
  const slotCount = ref(DEFAULT_SLOT_COUNT)
  const autoq = ref('5min')

  async function load() {
    try {
      const [slots, q] = await Promise.all([loadSlotCount(), readSetting('saves.autoq')])
      slotCount.value = slots
      if (typeof q?.value === 'string') autoq.value = q.value
    } catch (e) {
      fail(e)
    }
  }

  return { slotCount, autoq, load }
}
