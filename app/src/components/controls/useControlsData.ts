// What the Controls section reads while it is open: the settings schema, the `pad` page at
// «Tutti i giochi» (reloaded when a `pad.*` setting changes), the connected controllers (kept up
// to date by their events) and their live state (`emu://pad-state`, watched only meanwhile).
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { emu, type PadInfo, type PadState, type SettingRow, type SettingsSchema } from '@/emu'
import { report } from '@/lib/errors'
import { useListeners } from '@/composables/useListeners'
import { onSettingsChanged, settingsUnder } from '@/composables/useSettingsChanged'
import type { ErrorSink } from '@/i18n/labels'
import { GLOBAL, loadSchema } from '@/lib/settingsModel'
import { padChanged, padConnected, padGone } from './devices'

export function useControlsData(options: {
  onError: ErrorSink
  /** A controller was disconnected. */
  onPadGone: (id: number) => void
}) {
  const schema = ref<SettingsSchema | null>(null)
  const globalRows = ref<Record<string, SettingRow>>({})
  const pads = ref<PadInfo[]>([])
  const states = ref<PadState[]>([])

  async function loadGlobal() {
    try {
      const rows = await emu.settings.page('pad', GLOBAL)
      globalRows.value = Object.fromEntries(rows.map(r => [r.key, r]))
    } catch (e) {
      options.onError(e)
    }
  }

  const listeners = useListeners()
  // Closed while still loading: the live state must not be switched on after it was switched off.
  let closed = false
  onMounted(async () => {
    try {
      schema.value = await loadSchema()
    } catch (e) {
      options.onError(e)
    }
    await loadGlobal()
    try {
      pads.value = await emu.controls.pads()
    } catch (e) {
      options.onError(e)
    }
    await listeners.add(
      emu.onGamepadConnected(p => (pads.value = padConnected(pads.value, p))),
      emu.onGamepadDisconnected(p => {
        pads.value = padGone(pads.value, p.id)
        options.onPadGone(p.id)
      }),
      emu.onGamepadChanged(p => (pads.value = padChanged(pads.value, p))),
      emu.controls.onPadState(s => (states.value = s)),
      onSettingsChanged(settingsUnder('pad.'), () => void loadGlobal()),
    )
    if (!closed) await emu.controls.watch(true)
  })
  onBeforeUnmount(() => {
    closed = true
    report(emu.controls.watch(false))
  })

  return { schema, globalRows, pads, states }
}
