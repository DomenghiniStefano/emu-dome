// Moving between the quick menu's screens with the focus where the hand expects it: a page opens
// on its marked control (else its back button); B / Esc go back one level and focus the control
// the page was opened from (the chosen slot for a page the search opened); from the main screen
// they resume the game.
import { nextTick, ref, type Ref } from 'vue'
import { FocusMark, QuickMenuView } from './view'

const CONTROL = 'button, input, select, [tabindex]'

export interface NavigationHooks {
  /** B / Esc on the main screen. */
  resume: () => void
  /** Back on the main screen from a page. */
  home: () => void
}

export function useQuickMenuNavigation(root: Ref<HTMLElement | null>, hooks: NavigationHooks) {
  const view = ref<QuickMenuView>(QuickMenuView.Home)
  /** The control a page was opened from, focused again on the way back. */
  let opener: HTMLElement | null = null

  const find = (selector: string) => root.value?.querySelector<HTMLElement>(selector) ?? null

  /** The page's marked control (or the first one inside a marked group), else its back button. */
  function focusPage() {
    const mark = find(`[${FocusMark.SubInitial}]:not(:disabled)`)
    const first = mark?.matches(CONTROL)
      ? mark
      : mark?.querySelector<HTMLElement>(`:is(${CONTROL}):not(:disabled)`)
    ;(first ?? find(`[${FocusMark.Back}]`))?.focus()
  }

  /** Opens a page; `e` is the click that opened it (null: the search). */
  function open(next: QuickMenuView, e: Event | null) {
    if (view.value === QuickMenuView.Home)
      opener = e?.currentTarget instanceof HTMLElement ? e.currentTarget : null
    view.value = next
    void nextTick(focusPage)
  }

  function back() {
    if (view.value === QuickMenuView.Home) return hooks.resume()
    if (view.value === QuickMenuView.Shortcuts) {
      view.value = QuickMenuView.Controls
      void nextTick(() => find(`[${FocusMark.Shortcuts}]`)?.focus())
      return
    }
    // The screens editor closes back onto G09, on the button that opened it.
    if (view.value === QuickMenuView.ScreenEditor) {
      view.value = QuickMenuView.Screens
      void nextTick(() => find(`[${FocusMark.Customise}]`)?.focus())
      return
    }
    // The touch controls editor closes back onto G10, on the row that opened it.
    if (view.value === QuickMenuView.TouchEditor) {
      view.value = QuickMenuView.Controls
      void nextTick(() => find(`[${FocusMark.TouchEdit}]`)?.focus())
      return
    }
    view.value = QuickMenuView.Home
    hooks.home()
    void nextTick(() => (opener ? opener.focus() : void focusChosenSlot()))
  }

  /** Back to the main screen without moving the focus (Riavvia confirmed). */
  function home() {
    view.value = QuickMenuView.Home
  }

  /** Opening the menu focuses the chosen slot (or «Altri slot» when it is not on screen). */
  async function focusChosenSlot() {
    await nextTick()
    const target = find(`[${FocusMark.ChosenSlot}]`) ?? find(`[${FocusMark.MoreSlots}]`)
    target?.focus({ preventScroll: true })
  }

  return { view, open, back, home, focusChosenSlot }
}
