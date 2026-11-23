// TV mode (Trello #30): `data-layout="tv"` on <html> while the interface is laid out for the sofa,
// so the screens and the primitives style their TV look with Tailwind's `tv:` variant
// (src/design-system/index.css), and the few screens whose structure changes (the shell's tabs, the
// library's shelves, Settings) read `uiMode.tv`. TV is a layout of the same screens and logic,
// like the touch density (useDensity.ts), not a second app.
//
// The decision is lib/uiMode.ts's (pure, tested): `general.ui` and, at Automatica, the answer to
// the TV question (`uiMode.ask`, shown by components/tv-mode/TvModeDialog.vue): a controller
// connecting outside TV asks to switch, the last one leaving TV asks to go back. App.vue calls
// `useUiMode` once; it reads the settings and the controllers, follows their changes and the
// window's size (the TV sizes grow on a larger screen; a resize also asks Rust whether the window
// is full screen, kept for the quick menu's tile: it no longer decides TV), and sets the
// attribute. Every other screen reads `uiMode` (the layout, the question, the controller to name
// and draw the glyphs of, the confirm side).
import { computed, onMounted, readonly, ref, watch, watchEffect, type Ref } from 'vue'
import { emu, type PadInfo, type SettingDevice } from '@/emu'
import { ignore, report, reportError } from '@/lib/errors'
import { confirmSide, type ConfirmSide } from '@/lib/padGlyphs'
import { GLOBAL, readSetting } from '@/lib/settingsModel'
import {
  activePad,
  askAfter,
  asksOnConnect,
  asksOnDisconnect,
  decideUiLayout,
  padsAfter,
  rememberedPreference,
  tvScale,
  uiPreference,
  type AskInputs,
  type TvAsk,
  type UiLayout,
  type UiPreference,
} from '@/lib/uiMode'
import { useListeners } from './useListeners'
import { onSettingsChanged } from './useSettingsChanged'

const pads = ref<PadInfo[]>([])
const preference = ref<UiPreference>('auto')
const confirm = ref<ConfirmSide>('right')
const tv = ref(false)
/** Automatica: what the user answered for this run. */
const answered = ref<UiLayout | null>(null)
/** The TV question on screen, if any. */
const ask = ref<TvAsk | null>(null)
/** What the settings schema says this device is (set by `useUiMode`). */
const device = ref<SettingDevice | null>(null)

/** What the screens read: the layout, the question, the controller in use, the side that confirms. */
export const uiMode = readonly({
  tv,
  ask,
  pad: computed(() => activePad(pads.value)),
  confirm,
})

const inputsWith = (controllers: number): AskInputs => ({
  preference: preference.value,
  device: device.value,
  tv: tv.value,
  controllers,
})

/**
 * Whether a controller's change asks the TV question: the shell's «controller connected /
 * disconnected» toast stays quiet then, so the two never show together. Safe to call before or
 * after this module saw the same event (`padsAfter` gives the same list either way).
 */
export function tvAsksFor(change: 'connected' | 'disconnected', pad: PadInfo): boolean {
  const left = padsAfter(pads.value, pad, change).length
  return change === 'connected'
    ? asksOnConnect(inputsWith(left))
    : asksOnDisconnect(inputsWith(left))
}

/**
 * The answer to the TV question: `layout` for this run; with `remember` («Non chiederlo più»)
 * saved as `general.ui` (Modalità TV / desktop), so it is never asked again.
 */
export function answerTvAsk(layout: UiLayout, remember: boolean) {
  answered.value = layout
  ask.value = null
  if (remember) report(emu.settings.set('general.ui', GLOBAL, rememberedPreference(layout)))
}

/** How long the window's size stays still before it is asked whether it is full screen. */
const RESIZE_SETTLE_MS = 150

/**
 * Starts TV mode's decision. `fullscreen`: the window's state as the game session keeps it (kept
 * current here on every resize); `deviceOf`: what the settings schema says this is.
 */
export function useUiMode(fullscreen: Ref<boolean>, deviceOf: Readonly<Ref<SettingDevice | null>>) {
  // The window's size in CSS pixels: the TV sizes grow on a screen larger than the mockup's.
  const size = ref({ width: window.innerWidth, height: window.innerHeight })
  watchEffect(() => {
    device.value = deviceOf.value
    tv.value =
      decideUiLayout({
        preference: preference.value,
        device: device.value,
        answered: answered.value,
      }) === 'tv'
    const root = document.documentElement
    if (tv.value) {
      root.dataset.layout = 'tv'
      root.style.setProperty('--tv-scale', String(tvScale(size.value.width, size.value.height)))
    } else {
      delete root.dataset.layout
      root.style.removeProperty('--tv-scale')
    }
  })
  // A fixed layout chosen in Settings never asks: a question on screen goes away.
  watch(preference, p => {
    if (p !== 'auto') ask.value = null
  })

  /** The controllers changed: the question follows (lib/uiMode.ts). */
  function padsChanged(change: 'connected' | 'disconnected', pad: PadInfo) {
    const asks = tvAsksFor(change, pad)
    pads.value = padsAfter(pads.value, pad, change)
    const now = inputsWith(pads.value.length)
    if (asks) ask.value = change === 'connected' ? 'connect' : 'disconnect'
    else ask.value = askAfter(ask.value, now)
  }

  let prefsRead = false
  async function readPreferences() {
    preference.value = uiPreference((await readSetting('general.ui'))?.value)
    confirm.value = confirmSide((await readSetting('pad.confirm'))?.value)
    prefsRead = true
  }
  const refresh = () => readPreferences().catch(reportError)

  let settle: ReturnType<typeof setTimeout> | undefined
  function onResize() {
    size.value = { width: window.innerWidth, height: window.innerHeight }
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
  // or the other. A controller already connected at startup asks once everything is known.
  onMounted(async () => {
    const read = refresh()
    window.addEventListener('resize', onResize)
    try {
      await listeners.add(
        onSettingsChanged(['general.ui', 'pad.confirm'], refresh),
        emu.onGamepadConnected(p => padsChanged('connected', p)),
        emu.onGamepadChanged(p => (pads.value = padsAfter(pads.value, p, 'changed'))),
        emu.onGamepadDisconnected(p => padsChanged('disconnected', p)),
        () => {
          window.removeEventListener('resize', onResize)
          clearTimeout(settle)
        },
      )
      const known = await emu.controls.pads()
      // Events that arrived meanwhile are newer: keep their controllers after the listed ones.
      pads.value = [...known.filter(k => !pads.value.some(p => p.id === k.id)), ...pads.value]
      await read
      if (!prefsRead) return
      const askAtStart = () => {
        if (ask.value === null && asksOnConnect(inputsWith(pads.value.length)))
          ask.value = 'connect'
      }
      if (device.value !== null) return askAtStart()
      const stop = watch(device, d => {
        if (d === null) return
        stop()
        askAtStart()
      })
    } catch (e) {
      reportError(e)
    }
  })
}
