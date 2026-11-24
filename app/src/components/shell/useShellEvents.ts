// The backend's events the shell follows for its whole life: settings, the controller (menu
// button, navigation, combos, connections), the library, the core, the screen effects and the
// one-time move of old save names (told once, if it failed or left files behind).
// Stopped on unmount.
import { useI18n } from 'vue-i18n'
import { emu, type ErrorCode, type SavesMigration } from '@/emu'
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
import { tvAsksFor } from '@/composables/useUiMode'
import { touchSpeaksFor } from '@/components/touch/useControllerLoss'

export function useShellEvents(
  session: GameSession,
  toasts: Toasts,
  listenDragDrop: () => Promise<() => void>,
) {
  const { t } = useI18n()
  const { game } = session
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

  // What is not working (`emu://health`: no audio output, native video fallen back, controllers,
  // a stopped thread): each code told once while it lasts; one that clears and comes back is
  // told again.
  let healthTold = new Set<string>()
  function tellHealth(issues: ErrorCode[]) {
    const now = new Set(issues.map(i => JSON.stringify(i)))
    for (const issue of issues) {
      if (!healthTold.has(JSON.stringify(issue))) toasts.failWith(issue)
    }
    healthTold = now
  }

  /** What was already not working before the listener (it may have been reported at start). */
  async function askHealth() {
    try {
      tellHealth(await emu.diagnostics())
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
      await askHealth()
    }
  }

  async function subscribe() {
    await listeners.add(
      onSettingsChanged('pad.remap', () => void loadKeymap(game.value !== null)),
      onSettingsChanged(
        e => session.settingsConcern(e.key),
        () => void session.loadSettings(),
      ),
      // Android's own back (`system`) is neither a controller nor a step without the mouse: no
      // focus ring, and the input in use stays what it was (#131).
      emu.onMenu(system => {
        if (!system) {
          usedController()
          showFocusRing()
        }
        report(session.toggleMenu())
      }),
      emu.onNav((action, system) => {
        if (!system) usedController()
        dispatchNav(action, { ring: !system })
      }),
      emu.onHotkey(session.onHotkey),
      // A screen effect could not be used: the game shows without it.
      emu.onEffectError(toasts.failWith),
      // A game opened without the system files a setting asked for: it runs without them.
      emu.onBiosFallback(toasts.failWith),
      emu.onHealth(tellHealth),
      // The toasts stay quiet when the TV question (TvModeDialog) or, on a phone, E14
      // (components/touch) says it instead: never both.
      emu.onGamepadConnected(p => {
        if (!tvAsksFor('connected', p))
          toasts.notify(t('toast.gamepadConnected', { name: gamepadName(p) }))
      }),
      emu.onGamepadDisconnected(p => {
        if (!tvAsksFor('disconnected', p) && !touchSpeaksFor(p))
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
