// What the touch controls draw for the open game: the console's controls and the layouts saved on
// this device (`emu.touch.layouts`), and the `touch.*` values at the session's scope (its changes
// «in prova» included). Both are read again when a `touch.*` setting changes (C06's «Salva»
// included); an older answer is dropped (rule 7).
import { onMounted, ref } from 'vue'
import { emu, type ConsoleId, type TouchSetup } from '@/emu'
import { settingsUnder, useSettingsChanged } from '@/composables/useSettingsChanged'
import { useErrorSink } from '@/composables/useToasts'
import { latest } from '@/lib/latest'
import { prefsFrom } from './touchControls'

const SESSION = { level: 'session' } as const

export function useTouchSetup(console: () => ConsoleId) {
  const failWith = useErrorSink()
  const setup = ref<TouchSetup | null>(null)
  /** Until read: the catalogue's defaults (`prefsFrom` of no rows). */
  const prefs = ref(prefsFrom([]))
  const loads = latest()

  async function load() {
    const fresh = loads.next()
    try {
      const [s, rows] = await Promise.all([
        emu.touch.layouts(console()),
        emu.settings.page('touch', SESSION),
      ])
      if (!fresh()) return
      setup.value = s
      prefs.value = prefsFrom(rows)
    } catch (e) {
      if (fresh()) failWith(e)
    }
  }

  useSettingsChanged(settingsUnder('touch.'), () => void load())
  onMounted(() => void load())
  return { setup, prefs, load }
}
