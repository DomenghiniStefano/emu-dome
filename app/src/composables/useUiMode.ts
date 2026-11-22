// TV mode (Trello #30): `data-layout="tv"` on <html> while the interface is laid out for the sofa,
// so the screens and the primitives style their TV look with Tailwind's `tv:` variant
// (src/assets/main.css), and the few screens whose structure changes (the shell's tabs, the
// library's shelves) read `uiMode.tv`. TV is a layout of the same screens and logic, like the
// touch density (useDensity.ts), not a second app.
//
// The decision is lib/uiMode.ts's (pure, tested): `general.ui`, the window full screen, the
// controllers connected. App.vue calls `useUiMode` once; it reads the setting and the controllers,
// follows their changes and the window's (a resize asks Rust whether it is full screen: F11, the
// quick menu's tile, the system's own way out), and sets the attribute. Every other screen reads
// `uiMode` (the layout, the controller to name and draw the glyphs of, the confirm side).
import { computed, onMounted, readonly, ref, watchEffect, type Ref } from 'vue'
import { emu, type PadInfo, type SettingDevice } from '@/emu'
import { ignore, reportError } from '@/lib/errors'
import { confirmSide, type ConfirmSide } from '@/lib/padGlyphs'
import { readSetting } from '@/lib/settingsModel'
import { activePad, decideUiLayout, padsAfter, uiPreference, type UiPreference } from '@/lib/uiMode'
import { useListeners } from './useListeners'
import { onSettingsChanged } from './useSettingsChanged'

const pads = ref<PadInfo[]>([])
const preference = ref<UiPreference>('auto')
const confirm = ref<ConfirmSide>('right')
const tv = ref(false)

/** What the screens read: the layout, the controller in use, the side that confirms. */
export const uiMode = readonly({
  tv,
  pad: computed(() => activePad(pads.value)),
  confirm,
})

/** How long the window's size stays still before it is asked whether it is full screen. */
const RESIZE_SETTLE_MS = 150

/**
 * Starts TV mode's decision. `fullscreen`: the window's state as the game session keeps it (kept
 * current here on every resize); `device`: what the settings schema says this is.
 */
export function useUiMode(fullscreen: Ref<boolean>, device: Readonly<Ref<SettingDevice | null>>) {
  watchEffect(() => {
    tv.value =
      decideUiLayout({
        preference: preference.value,
        device: device.value,
        fullscreen: fullscreen.value,
        controllers: pads.value.length,
      }) === 'tv'
    const root = document.documentElement
    if (tv.value) root.dataset.layout = 'tv'
    else delete root.dataset.layout
  })

  async function readPreferences() {
    preference.value = uiPreference((await readSetting('general.ui'))?.value)
    confirm.value = confirmSide((await readSetting('pad.confirm'))?.value)
  }
  const refresh = () => readPreferences().catch(reportError)

  let settle: ReturnType<typeof setTimeout> | undefined
  function onResize() {
    clearTimeout(settle)
    settle = setTimeout(() => {
      ignore(
        emu.isFullscreen().then(on => (fullscreen.value = on)),
        'TV mode: the window did not say whether it is full screen',
      )
    }, RESIZE_SETTLE_MS)
  }

  const listeners = useListeners()
  // The controllers' events first, then the list: a controller connected in between is in one
  // or the other.
  onMounted(async () => {
    void refresh()
    window.addEventListener('resize', onResize)
    try {
      await listeners.add(
        onSettingsChanged(['general.ui', 'pad.confirm'], refresh),
        emu.onGamepadConnected(p => (pads.value = padsAfter(pads.value, p, 'connected'))),
        emu.onGamepadChanged(p => (pads.value = padsAfter(pads.value, p, 'changed'))),
        emu.onGamepadDisconnected(p => (pads.value = padsAfter(pads.value, p, 'disconnected'))),
        () => {
          window.removeEventListener('resize', onResize)
          clearTimeout(settle)
        },
      )
      const known = await emu.controls.pads()
      // Events that arrived meanwhile are newer: keep their controllers after the listed ones.
      pads.value = [...known.filter(k => !pads.value.some(p => p.id === k.id)), ...pads.value]
    } catch (e) {
      reportError(e)
    }
  })
}
