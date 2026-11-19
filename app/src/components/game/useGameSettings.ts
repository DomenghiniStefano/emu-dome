// The open game's settings the shell applies itself (fast forward, undo, rewind, colour
// correction), at session level so its «in prova» changes count.
import { ref, type Ref } from 'vue'
import { emu, type GameInfo } from '@/emu'
import { GAME_SETTING_PAGES, gameSettingsFrom } from './session'
import type { Toasts } from '@/composables/useToasts'

export function useGameSettings(game: Ref<GameInfo | null>, toasts: Toasts) {
  const settings = ref(gameSettingsFrom([]))
  async function load() {
    if (!game.value) return
    try {
      const pages = GAME_SETTING_PAGES.map(p => emu.settings.page(p, { level: 'session' }))
      settings.value = gameSettingsFrom((await Promise.all(pages)).flat())
    } catch (e) {
      toasts.failWith(e)
    }
  }
  /** A settings change that concerns them. */
  const concerns = (key: string) => GAME_SETTING_PAGES.some(p => key.startsWith(`${p}.`))
  return { settings, load, concerns }
}
