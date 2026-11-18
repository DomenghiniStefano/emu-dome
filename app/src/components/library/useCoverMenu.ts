// The «…» menu of a cover (CoverMenu): one open at a time, the focus back on its button when it
// closes, favourite and hide through `library.setFlags`.
import { nextTick, ref } from 'vue'
import { emu, type LibraryGame } from '../../emu'
import { errorText } from '../../i18n/labels'

export function useCoverMenu(onError: (text: string) => void) {
  /** The id of the game whose menu is open. */
  const menuFor = ref<string | null>(null)
  let menuButton: HTMLElement | null = null

  function toggleMenu(g: LibraryGame, e: Event) {
    menuButton = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
    menuFor.value = menuFor.value === g.id ? null : g.id
  }
  function closeMenu() {
    menuFor.value = null
    void nextTick(() => menuButton?.focus())
  }
  async function setFlags(g: LibraryGame, flags: { favourite?: boolean; hidden?: boolean }) {
    closeMenu()
    try {
      await emu.library.setFlags(g.id, flags)
    } catch (e) {
      onError(errorText(e))
    }
  }

  return { menuFor, toggleMenu, closeMenu, setFlags }
}
