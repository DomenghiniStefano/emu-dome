// E14 «Controller scollegato» carried out (the rule is controllerLoss.ts's), and the controllers
// the touch controls follow (`touch.tpad`). The touch layer (TouchLayer.vue) calls
// `useControllerLoss` while a game is open on a touch screen; it keeps the connected controllers
// and what the game is doing in this module, so the shell's «controller disconnected» toast can
// ask `touchSpeaksFor` and stay quiet when E14 speaks (never both), as it does for the TV question.
import { reactive, readonly, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu, type PadInfo, type SettingDevice } from '@/emu'
import { useListeners } from '@/composables/useListeners'
import { useToasts } from '@/composables/useToasts'
import { report, reportError } from '@/lib/errors'
import { padsAfter } from '@/lib/uiMode'
import { onControllerLost, type LostChoice } from './controllerLoss'

const pads = ref<PadInfo[]>([])
/** What E14 needs to know of the game, kept current by the touch layer. */
const context = reactive({
  device: null as SettingDevice | null,
  inGame: false,
  menuOpen: false,
  busy: false,
  choice: 'pause' as LostChoice,
})

/** The controllers connected now, as the touch controls see them. */
export const touchPads = readonly(pads)

/**
 * Whether E14 speaks for `pad` going away (the question or its own toast): the shell's
 * «controller disconnected» toast stays quiet then. Safe before or after this module saw the
 * same event (`padsAfter` gives the same list either way).
 */
export function touchSpeaksFor(pad: PadInfo): boolean {
  const left = padsAfter(pads.value, pad, 'disconnected').length
  return onControllerLost({ ...context, left }, context.choice) !== null
}

export interface LossOptions {
  device: () => SettingDevice | null
  menuOpen: () => boolean
  busy: () => boolean
  choice: () => LostChoice
  /** The game paused or running again because of E14 (the session's `paused`, for the HUD and
   * the hang watchdog). */
  paused: (on: boolean) => void
}

export function useControllerLoss(o: LossOptions) {
  const { t } = useI18n()
  const toasts = useToasts()
  /** The question on screen. */
  const asking = ref(false)
  /** «Aspetta il controller»: paused until one connects. */
  const waiting = ref(false)

  watch(
    () => [o.device(), o.menuOpen(), o.busy(), o.choice()] as const,
    ([device, menuOpen, busy, choice]) => {
      Object.assign(context, { device, inGame: true, menuOpen, busy, choice })
      // The quick menu takes over: it pauses and resumes the game itself.
      if (menuOpen) waiting.value = false
    },
    { immediate: true },
  )

  function hold(on: boolean) {
    report(emu.setPaused(on))
    o.paused(on)
  }

  function lost(pad: PadInfo) {
    const left = padsAfter(pads.value, pad, 'disconnected')
    const what = onControllerLost({ ...context, left: left.length }, context.choice)
    pads.value = left
    if (what === 'ask') {
      hold(true)
      asking.value = true
    } else if (what === 'toast') {
      toasts.notify(t('touch.lost.toast'))
    }
  }

  function connected(pad: PadInfo) {
    pads.value = padsAfter(pads.value, pad, 'connected')
    // The question closes by itself, and the game goes on, when a controller comes back.
    if (asking.value || waiting.value) {
      asking.value = false
      waiting.value = false
      hold(false)
    }
  }

  /** «Continua con il tocco»: the game goes on, the touch controls are back. */
  function continueWithTouch() {
    asking.value = false
    hold(false)
  }

  /** «Aspetta il controller»: the question goes, the game stays paused until one connects. */
  function wait() {
    asking.value = false
    waiting.value = true
  }

  const listeners = useListeners()
  // The events first, then the list: a controller connected in between is in one or the other.
  async function listen() {
    try {
      await listeners.add(
        emu.onGamepadConnected(connected),
        emu.onGamepadChanged(p => (pads.value = padsAfter(pads.value, p, 'changed'))),
        emu.onGamepadDisconnected(lost),
        () => (context.inGame = false),
      )
      const known = await emu.controls.pads()
      pads.value = [...known.filter(k => !pads.value.some(p => p.id === k.id)), ...pads.value]
    } catch (e) {
      reportError(e)
    }
  }

  return { asking, waiting, pads: touchPads, listen, continueWithTouch, wait }
}
