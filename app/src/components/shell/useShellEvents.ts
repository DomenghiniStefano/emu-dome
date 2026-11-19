// The backend's events the shell follows for its whole life: settings, the controller (menu
// button, navigation, combos, connections), the library, the core and the screen effects.
// Stopped on unmount.
import { useI18n } from 'vue-i18n'
import { emu } from '@/emu'
import { gamepadName } from '@/i18n/labels'
import { usedController } from '@/lib/inputMode'
import { loadKeymap } from '@/lib/keymap'
import { startLibrary } from '@/lib/library'
import { dispatchNav, showFocusRing } from '@/nav'
import type { GameSession } from '@/components/game/useGameSession'
import { useListeners } from '@/composables/useListeners'
import { onSettingsChanged } from '@/composables/useSettingsChanged'
import type { Toasts } from '@/composables/useToasts'

export function useShellEvents(
  session: GameSession,
  toasts: Toasts,
  listenDragDrop: () => Promise<() => void>,
) {
  const { t } = useI18n()
  const { game, controller } = session
  const listeners = useListeners()

  async function listen() {
    await listeners.add(
      onSettingsChanged('pad.remap', () => void loadKeymap(game.value !== null)),
      onSettingsChanged(
        e => session.settingsConcern(e.key),
        () => void session.loadSettings(),
      ),
      emu.onMenu(() => {
        usedController()
        showFocusRing()
        void session.toggleMenu()
      }),
      emu.onNav(action => {
        usedController()
        dispatchNav(action)
      }),
      emu.onHotkey(session.onHotkey),
      // A screen effect could not be used: the game shows without it.
      emu.onEffectError(toasts.failWith),
      emu.onGamepadConnected(p => {
        controller.value = gamepadName(p)
        toasts.notify(t('toast.gamepadConnected', { name: gamepadName(p) }))
      }),
      emu.onGamepadDisconnected(p => {
        if (controller.value === gamepadName(p)) controller.value = null
        toasts.notify(t('toast.gamepadDisconnected', { name: gamepadName(p) }))
      }),
      listenDragDrop(),
      ...(await startLibrary()),
      emu.onEvent(session.onEmuEvent),
    )
  }

  return { listen }
}
