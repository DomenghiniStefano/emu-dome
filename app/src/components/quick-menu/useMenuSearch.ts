// The search everywhere (X01, #32) from the quick menu: while the menu is open it is the search's
// host (search/useSearch.ts), opened with Y (its nav scope's `search` step) or Ctrl + K. Owner,
// 2026-11-22: «sì, includila nel menu rapido».
//
// What choosing does here, through what the menu already does, never typed again:
// - a shortcut runs as the menu's own controls do: Salva / Carica and the slot steps on the slot
//   chosen in the menu (Carica resumes, as its button does), fast forward and slow motion as the
//   shortcuts' toggle, the screenshot, the FPS, the screens and the lid, fullscreen, and «menu»
//   resumes the game; the palette closes and the menu stays as the action leaves it. The held
//   shortcuts and the pause carry a reason instead (searchContext.ts);
// - a setting opens the menu's page that shows it «in prova» (G09 Schermi e immagine, G10
//   Controlli, G16 Imposta le velocità); the others say to open them from the library;
// - another game closes this one («Chiudere senza salvare?» first with progress not in a quick
//   save, then G14, the close flow #86 with its «Wait» / «Close anyway») and opens; the game being
//   played carries a reason.
import type { Ref } from 'vue'
import type { LibraryGame, SettingKey, Speed } from '@/emu'
import type { KeyAction } from '@/lib/keyboard'
import type { GameContext } from '@/components/search/searchContext'
import { useSearchHost } from '@/components/search/useSearch'
import { steppedSlot } from './slots'
import { toggledSpeed } from './speedChoices'
import { sessionViewOf, type QuickMenuView } from './view'

export interface MenuSearchOptions {
  /** The menu's nav scope. */
  root: Ref<HTMLElement | null>
  /** The open game as the search needs it (read reactively). */
  context: () => GameContext
  /** The menu's quick saves (useQuickSlots). */
  slots: {
    numbers: () => number[]
    chosen: () => number
    pick: (slot: number) => void
    save: () => void
    loadChosen: () => void
  }
  /** The speed now, and the two modes' speeds (G16). */
  speed: () => Speed
  modes: () => { ff: Speed; slow: Speed }
  /** What the menu's controls do (its emits). */
  run: {
    resume: () => void
    speed: (s: Speed) => void
    screenshot: () => void
    fps: () => void
    swapScreens: () => void
    nextLayout: () => void
    toggleLid: () => void
    fullscreen: () => void
    openGame: (game: LibraryGame) => void
  }
  /** Opens one of the menu's pages. */
  openPage: (view: QuickMenuView) => void
}

export function useMenuSearch(o: MenuSearchOptions) {
  function action(id: KeyAction) {
    switch (id) {
      case 'menu':
        return o.run.resume()
      case 'save':
        return o.slots.save()
      case 'load':
        return o.slots.loadChosen()
      case 'next_slot':
      case 'prev_slot':
        return o.slots.pick(steppedSlot(o.slots.numbers(), o.slots.chosen(), id === 'next_slot'))
      case 'ff_toggle':
        return o.run.speed(toggledSpeed(o.speed(), o.modes().ff))
      case 'slow':
        return o.run.speed(toggledSpeed(o.speed(), o.modes().slow))
      case 'screenshot':
        return o.run.screenshot()
      case 'fps':
        return o.run.fps()
      case 'swap':
        return o.run.swapScreens()
      case 'layout':
        return o.run.nextLayout()
      case 'lid':
        return o.run.toggleLid()
      case 'fullscreen':
        return o.run.fullscreen()
      // Never chosen: they carry a reason here (searchContext.ts).
      case 'ff_hold':
      case 'rewind':
      case 'pause':
        return
    }
  }

  function setting(key: SettingKey) {
    const view = sessionViewOf(key)
    if (view) o.openPage(view)
  }

  return useSearchHost({
    root: o.root,
    context: () => ({ game: o.context() }),
    action,
    setting,
    game: g => o.run.openGame(g),
  })
}
