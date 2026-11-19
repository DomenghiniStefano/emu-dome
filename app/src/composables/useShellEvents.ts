// The backend's events the shell follows for its whole life: settings, the controller (menu
// button, navigation, combos, connections), the library, the core and the screen effects.
// Stopped on unmount.
import { onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { emu } from '../emu'
import { gamepadName } from '../i18n/labels'
import { usedController } from '../inputMode'
import { loadKeymap } from '../keymap'
import { startLibrary } from '../library'
import { dispatchNav, showFocusRing } from '../nav'
import type { GameSession } from './useGameSession'
import type { Toasts } from './useToasts'

export function useShellEvents(
  session: GameSession,
  toasts: Toasts,
  listenDragDrop: () => Promise<() => void>,
) {
  const { t } = useI18n()
  const { game, controller } = session
  const unlisten: Array<() => void> = []

  async function listen() {
    unlisten.push(
      await emu.settings.onChanged(e => {
        if (e.key === 'pad.remap') void loadKeymap(game.value !== null)
        if (session.settingsConcern(e.key)) void session.loadSettings()
      }),
      await emu.onMenu(() => {
        usedController()
        showFocusRing()
        void session.toggleMenu()
      }),
      await emu.onNav(action => {
        usedController()
        dispatchNav(action)
      }),
      await emu.onHotkey(session.onHotkey),
      // A screen effect could not be used: the game shows without it.
      await emu.onEffectError(toasts.failWith),
      await emu.onGamepadConnected(p => {
        controller.value = gamepadName(p)
        toasts.notify(t('toast.gamepadConnected', { name: gamepadName(p) }))
      }),
      await emu.onGamepadDisconnected(p => {
        if (controller.value === gamepadName(p)) controller.value = null
        toasts.notify(t('toast.gamepadDisconnected', { name: gamepadName(p) }))
      }),
      await listenDragDrop(),
      ...(await startLibrary()),
      await emu.onEvent(session.onEmuEvent),
    )
  }
  onUnmounted(() => unlisten.forEach(u => u()))

  return { listen }
}
