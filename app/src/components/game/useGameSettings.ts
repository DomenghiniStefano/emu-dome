// The open game's settings the shell applies itself (fast forward, undo, rewind, colour
// correction, the webview canvas's smoothing), at session level so its «in prova» changes count.
import { ref, type Ref } from 'vue'
import { emu, type GameInfo } from '@/emu'
import { latest } from '@/lib/latest'
import { loadSchema } from '@/lib/settingsModel'
import { GAME_SETTING_PAGES, gameSettingsFrom } from './session'
import type { Toasts } from '@/composables/useToasts'

export function useGameSettings(game: Ref<GameInfo | null>, toasts: Toasts) {
  const settings = ref(gameSettingsFrom([]))
  /** Each setting's page, from the schema (a key's prefix is not always its page: «Avanzate»). */
  let pageOf = new Map<string, string>()
  // Rule 7: another game opened, or a newer change, while these loaded: the older answer is dropped.
  const loads = latest()

  async function load() {
    if (!game.value) return
    const fresh = loads.next()
    try {
      const schema = await loadSchema()
      pageOf = new Map(schema.entries.map(e => [e.key, e.page]))
      const pages = GAME_SETTING_PAGES.map(p => emu.settings.page(p, { level: 'session' }))
      const rows = (await Promise.all(pages)).flat()
      if (fresh()) settings.value = gameSettingsFrom(rows)
    } catch (e) {
      if (fresh()) toasts.failWith(e)
    }
  }
  /** A settings change that concerns them: a key on one of their pages. */
  const concerns = (key: string) =>
    (GAME_SETTING_PAGES as readonly string[]).includes(pageOf.get(key) ?? '')
  return { settings, load, concerns }
}
