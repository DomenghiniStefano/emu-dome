// What the Controls section reads while it is open: the settings schema, the `pad` page at
// «Tutti i giochi» (reloaded when a `pad.*` setting changes), the connected controllers (kept up
// to date by their events) and their live state (`emu://pad-state`, watched only meanwhile).
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { emu, type PadInfo, type PadState, type SettingRow, type SettingsSchema } from '../../emu'
import { useListeners } from '../../composables/useListeners'
import { onSettingsChanged, settingsUnder } from '../../composables/useSettingsChanged'
import type { ErrorSink } from '../../i18n/labels'
import { GLOBAL, loadSchema } from '../../settingsModel'

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
  onMounted(async () => {
    schema.value = await loadSchema()
    await loadGlobal()
    try {
      pads.value = await emu.controls.pads()
    } catch (e) {
      options.onError(e)
    }
    await listeners.add(
      emu.onGamepadConnected(p => {
        if (!pads.value.some(x => x.id === p.id)) pads.value = [...pads.value, p]
      }),
      emu.onGamepadDisconnected(p => {
        pads.value = pads.value.filter(x => x.id !== p.id)
        options.onPadGone(p.id)
      }),
      emu.onGamepadChanged(p => {
        pads.value = pads.value.map(x => (x.id === p.id ? p : x))
      }),
      emu.controls.onPadState(s => (states.value = s)),
      onSettingsChanged(settingsUnder('pad.'), () => void loadGlobal()),
    )
    await emu.controls.watch(true)
  })
  onBeforeUnmount(() => void emu.controls.watch(false))

  return { schema, globalRows, pads, states }
}
