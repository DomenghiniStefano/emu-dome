// The Saves section's settings at «Tutti i giochi»: how many quick save slots there are
// (`saves.slots`) and how often the auto quick save runs (`saves.autoq`, E03's sentence); and the
// quick save shortcut as C01 shows it (generic pad names), for E03 too.
import { ref } from 'vue'
import { emu, type SettingsScope } from '../../emu'
import { comboText } from '../../controls'

const GLOBAL: SettingsScope = { level: 'global' }
const DEFAULT_SLOTS = 10

export function useSavesSettings(fail: (e: unknown) => void) {
  const slotCount = ref(DEFAULT_SLOTS)
  const autoq = ref('5min')

  async function load() {
    try {
      const rows = await emu.settings.page('saves', GLOBAL)
      const value = (key: string) => rows.find(r => r.key === key)?.value
      slotCount.value = Number(value('saves.slots') ?? DEFAULT_SLOTS) || DEFAULT_SLOTS
      const q = value('saves.autoq')
      if (typeof q === 'string') autoq.value = q
    } catch (e) {
      fail(e)
    }
  }

  return { slotCount, autoq, load }
}

export function useSaveShortcut(fail: (e: unknown) => void) {
  const text = ref('')

  async function load() {
    try {
      const b = await emu.controls.bindings('generic', GLOBAL)
      const row = b.combos.find(c => c.hotkey === 'save')
      text.value = comboText('generic', b.modifier, row?.pos ?? row?.default ?? null)
    } catch (e) {
      fail(e)
    }
  }

  return { text, load }
}
