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
  /**
   * The level a page goes back to (the phone: the pages opened from «Altro» go back to it, and
   * «Altro» to the main screen); none: the main screen.
   */
  parent?: (view: QuickMenuView) => QuickMenuView
}

type Opener = { id: string | null; el: HTMLElement }

export function useQuickMenuNavigation(root: Ref<HTMLElement | null>, hooks: NavigationHooks) {
  const view = ref<QuickMenuView>(QuickMenuView.Home)
  /**
   * The control each level opened its page from, focused again on the way back: by its
   * `FocusMark.Opener` identity, since the level is unmounted while the page shows and the
   * element itself is gone by then (the element is kept only for a control without the mark that
   * is still there).
   */
  const openers = new Map<QuickMenuView, Opener | null>()
  let opener: Opener | null = null

  function openerOf(e: Event | null) {
    const el = e?.currentTarget instanceof HTMLElement ? e.currentTarget : null
    if (!el) return null
    const marked = el.closest<HTMLElement>(`[${FocusMark.Opener}]`)
    return { id: marked?.getAttribute(FocusMark.Opener) ?? null, el }
  }

  /** The opener back on the remounted level, else the chosen slot (the main screen) or the
   * level's first control. */
  function focusOpener(level: QuickMenuView) {
    const target = opener?.id
      ? find(`[${FocusMark.Opener}="${CSS.escape(opener.id)}"]`)
      : opener?.el.isConnected
        ? opener.el
        : null
    if (target) target.focus()
    else if (level === QuickMenuView.Home) void focusChosenSlot()
    else focusPage()
  }

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
    if (parentOf(next) === view.value) openers.set(view.value, openerOf(e))
    if (next === QuickMenuView.TouchEditor) touchEditorFrom = view.value
    view.value = next
    void nextTick(focusPage)
  }
  /** Where the C06 editor was opened: G10 «Controlli», or the phone's «Altro» (Trello #111). */
  let touchEditorFrom: QuickMenuView = QuickMenuView.Controls

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
    // The touch controls editor closes back onto G10, on the row that opened it, or onto the
    // phone's «Altro», on «Modifica la disposizione».
    if (view.value === QuickMenuView.TouchEditor && touchEditorFrom === QuickMenuView.More) {
      opener = openers.get(QuickMenuView.More) ?? null
      view.value = QuickMenuView.More
      void nextTick(() => focusOpener(QuickMenuView.More))
      return
    }
    if (view.value === QuickMenuView.TouchEditor) {
      view.value = QuickMenuView.Controls
      void nextTick(() => find(`[${FocusMark.TouchEdit}]`)?.focus())
      return
    }
    const level = parentOf(view.value)
    opener = openers.get(level) ?? null
    view.value = level
    if (level === QuickMenuView.Home) hooks.home()
    void nextTick(() => focusOpener(level))
  }

  const parentOf = (v: QuickMenuView) => hooks.parent?.(v) ?? QuickMenuView.Home

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
