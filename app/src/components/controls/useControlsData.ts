// What the Controls section reads while it is open: the settings schema, the `pad` page at
// «Tutti i giochi» (reloaded when a `pad.*` setting changes), the connected controllers (kept up
// to date by their events) and their live state (`emu://pad-state`, watched only meanwhile).
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { emu, type PadInfo, type PadState, type SettingRow, type SettingsSchema } from '../../emu'
import { errorText } from '../../i18n/labels'
import { loadSchema } from '../../settingsModel'

export function useControlsData(options: {
  onError: (text: string) => void
  /** A controller was disconnected. */
  onPadGone: (id: number) => void
}) {
  const schema = ref<SettingsSchema | null>(null)
  const globalRows = ref<Record<string, SettingRow>>({})
  const pads = ref<PadInfo[]>([])
  const states = ref<PadState[]>([])

  async function loadGlobal() {
    try {
      const rows = await emu.settings.page('pad', { level: 'global' })
      globalRows.value = Object.fromEntries(rows.map(r => [r.key, r]))
    } catch (e) {
      options.onError(errorText(e))
    }
  }

  const unlisten: Array<() => void> = []
  onMounted(async () => {
    schema.value = await loadSchema()
    await loadGlobal()
    try {
      pads.value = await emu.controls.pads()
    } catch (e) {
      options.onError(errorText(e))
    }
    unlisten.push(
      await emu.onGamepadConnected(p => {
        if (!pads.value.some(x => x.id === p.id)) pads.value = [...pads.value, p]
      }),
      await emu.onGamepadDisconnected(p => {
        pads.value = pads.value.filter(x => x.id !== p.id)
        options.onPadGone(p.id)
      }),
      await emu.onGamepadChanged(p => {
        pads.value = pads.value.map(x => (x.id === p.id ? p : x))
      }),
      await emu.controls.onPadState(s => (states.value = s)),
      await emu.settings.onChanged(e => {
        if (e.key.startsWith('pad.')) void loadGlobal()
      }),
    )
    await emu.controls.watch(true)
  })
  onBeforeUnmount(() => {
    unlisten.forEach(u => u())
    void emu.controls.watch(false)
  })

  return { schema, globalRows, pads, states }
}
