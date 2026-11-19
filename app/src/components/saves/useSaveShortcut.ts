// The quick save shortcut as C01 shows it (generic pad names), for E03's sentence in Saves.
import { ref } from 'vue'
import { emu } from '../../emu'
import { comboText } from '../../controls'
import type { ErrorSink } from '../../i18n/labels'
import { GLOBAL } from '../../settingsModel'

export function useSaveShortcut(fail: ErrorSink) {
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
