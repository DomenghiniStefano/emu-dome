// The backend's events the shell follows for its whole life: settings, the controller (menu
// button, navigation, combos, connections), the library, the core, the screen effects and the
// one-time move of old save names (told once, if it failed or left files behind).
// Stopped on unmount.
import { useI18n } from 'vue-i18n'
import { emu, type SavesMigration } from '@/emu'
import { gamepadName } from '@/i18n/labels'
import { report } from '@/lib/errors'
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

  // The move ends once per start: its problem is told once, whether the event or the first
  // question answers first.
  let migrationTold = false
  function tellMigration(m: SavesMigration) {
    if (m.running || !m.problem || migrationTold) return
    migrationTold = true
    toasts.failWith(m.problem)
  }

  /** Whether the move of old save names ended badly (it may have ended before the listener). */
  async function askMigration() {
    try {
      tellMigration(await emu.saves.migration())
    } catch (e) {
      toasts.failWith(e)
    }
  }

  /**
   * Subscribes to every event at once: one that fails is thrown (App.vue reports it), the others
   * are kept, the controller's above all.
   */
  async function listen() {
    try {
      await subscribe()
    } finally {
      await askMigration()
    }
  }

  async function subscribe() {
    await listeners.add(
      onSettingsChanged('pad.remap', () => void loadKeymap(game.value !== null)),
      onSettingsChanged(
        e => session.settingsConcern(e.key),
        () => void session.loadSettings(),
      ),
      emu.onMenu(() => {
        usedController()
        showFocusRing()
        report(session.toggleMenu())
      }),
      emu.onNav(action => {
        usedController()
        dispatchNav(action)
      }),
      emu.onHotkey(session.onHotkey),
      // A screen effect could not be used: the game shows without it.
      emu.onEffectError(toasts.failWith),
      // A game opened without the system files a setting asked for: it runs without them.
      emu.onBiosFallback(toasts.failWith),
      emu.onGamepadConnected(p => {
        controller.value = gamepadName(p)
        toasts.notify(t('toast.gamepadConnected', { name: gamepadName(p) }))
      }),
      emu.onGamepadDisconnected(p => {
        if (controller.value === gamepadName(p)) controller.value = null
        toasts.notify(t('toast.gamepadDisconnected', { name: gamepadName(p) }))
      }),
      listenDragDrop(),
      startLibrary(),
      emu.onEvent(session.onEmuEvent),
      emu.saves.onMigration(tellMigration),
    )
  }

  return { listen }
}
