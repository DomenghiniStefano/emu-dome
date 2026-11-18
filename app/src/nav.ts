// Focus navigation with the controller and the keyboard, for every screen.
//
// A screen registers a scope with `useNavScope(root, { back })`: the latest mounted scope
// receives the steps (`dispatchNav`), coming from `emu://nav` (the controller, routed in
// Rust) or from the arrow keys / Enter / Esc. Directions move the focus spatially to the
// nearest focusable element in that direction, by bounding rects, so a segmented control
// moves left/right within itself and up/down out of it without any per-screen code.
// `confirm` clicks the focused element, `back` calls the scope's `back`.
//
// The focus ring shows only while moving without the mouse: `data-nav` on <html>
// (styles.css), set by a step or a key, cleared by the pointer. The focused element also
// carries `data-nav-focus`, because `:focus` does not match while the window is in the
// background, and the controller keeps working then (SDL reads pads in the background).
import { onBeforeUnmount, onMounted, type Ref } from 'vue'
import type { NavAction } from './emu'

interface Scope {
  root: Ref<HTMLElement | null>
  back?: () => void
  /** L1 / R1 (`prev` / `next`): the screen's own tabs or levels. */
  tab?: (dir: -1 | 1) => void
}

const scopes: Scope[] = []

const FOCUSABLE = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'

/** Mark the element a scope focuses first with this attribute (`data-nav-initial`). */
const INITIAL = '[data-nav-initial]'

let watching = false

/** Shows the focus ring: the user is moving with the controller or the keyboard. */
export function showFocusRing() {
  document.documentElement.dataset.nav = ''
  if (watching) return
  watching = true
  window.addEventListener('pointerdown', () => delete document.documentElement.dataset.nav, {
    capture: true,
  })
  let marked: HTMLElement | null = null
  const mark = (el: EventTarget | null) => {
    if (marked) delete marked.dataset.navFocus
    marked = el instanceof HTMLElement ? el : null
    if (marked) marked.dataset.navFocus = ''
  }
  // Only a focus moving to nothing (`relatedTarget` null) clears the mark: a window going to
  // the background blurs the element too, and the mark must survive that.
  document.addEventListener('focusin', e => mark(e.target))
  document.addEventListener('focusout', e => {
    if (!e.relatedTarget && document.activeElement !== e.target) mark(null)
  })
  if (document.activeElement !== document.body) mark(document.activeElement)
}

function focusables(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    el => !el.matches(':disabled') && el.getClientRects().length > 0,
  )
}

function focus(el: HTMLElement | null | undefined) {
  if (!el) return
  el.focus({ preventScroll: true })
  el.scrollIntoView({ block: 'nearest', inline: 'nearest' })
}

function focusInitial(root: HTMLElement) {
  const marked = root.querySelector<HTMLElement>(INITIAL)
  focus(marked && !marked.matches(':disabled') ? marked : focusables(root)[0])
}

/** The focused element, when it is inside `root`. */
function current(root: HTMLElement): HTMLElement | null {
  const active = document.activeElement
  return active instanceof HTMLElement && root.contains(active) && active !== root ? active : null
}

export type Direction = 'up' | 'down' | 'left' | 'right'

/** Gap between two ranges on one axis, 0 when they overlap. */
const gap = (a0: number, a1: number, b0: number, b1: number) => Math.max(0, b0 - a1, a0 - b1)

/**
 * How far `to` is from `from` going in `dir`, or null when it is not in that direction.
 * Distance along the direction, plus twice the sideways gap (an element in line wins over
 * a nearer one off to the side), plus a little of the sideways offset to break ties.
 */
function score(from: DOMRect, to: DOMRect, dir: Direction): number | null {
  const slack = 2
  const horizontal = dir === 'left' || dir === 'right'
  const along =
    dir === 'right'
      ? to.left - from.right
      : dir === 'left'
        ? from.left - to.right
        : dir === 'down'
          ? to.top - from.bottom
          : from.top - to.bottom
  if (along < -slack) return null
  const side = horizontal
    ? gap(from.top, from.bottom, to.top, to.bottom)
    : gap(from.left, from.right, to.left, to.right)
  const offset = horizontal
    ? Math.abs(to.top + to.height / 2 - (from.top + from.height / 2))
    : Math.abs(to.left + to.width / 2 - (from.left + from.width / 2))
  return Math.max(0, along) + side * 2 + offset * 0.01
}

function move(root: HTMLElement, dir: Direction) {
  const from = current(root)
  if (!from) return focusInitial(root)
  const rect = from.getBoundingClientRect()
  let best: HTMLElement | null = null
  let bestScore = Infinity
  for (const el of focusables(root)) {
    if (el === from) continue
    const s = score(rect, el.getBoundingClientRect(), dir)
    if (s !== null && s < bestScore) {
      best = el
      bestScore = s
    }
  }
  focus(best)
}

/** Root of the scope that receives the steps now (the latest mounted one with a root). */
export function activeScopeRoot(): HTMLElement | null {
  return [...scopes].reverse().find(s => s.root.value)?.root.value ?? null
}

/** One step to the active scope (the latest mounted one with its root in the page). */
export function dispatchNav(action: NavAction) {
  showFocusRing()
  const scope = [...scopes].reverse().find(s => s.root.value)
  const root = scope?.root.value
  if (!scope || !root) return
  switch (action) {
    case 'back':
      scope.back?.()
      return
    case 'prev':
    case 'next':
      scope.tab?.(action === 'prev' ? -1 : 1)
      return
    // X: the focused item's other actions, as a right click (a cover opens its game details).
    case 'more':
      current(root)?.dispatchEvent(
        new MouseEvent('contextmenu', { bubbles: true, cancelable: true }),
      )
      return
    case 'confirm': {
      const el = current(root)
      if (!el) return focusInitial(root)
      if (ownsEnter(el)) sendKey(el, 'Enter')
      else el.click()
      return
    }
    case 'up':
    case 'down':
    case 'left':
    case 'right': {
      const el = current(root)
      if (el && ownsArrow(el, action)) return sendKey(el, ARROW_KEY[action])
      if (stepSlider(el, action)) return
      move(root, action)
    }
  }
}

// ---- Elements that handle keys themselves (the design system's Reka UI primitives)
//
// Some Reka components move the focus or act on a key of their own: a tab list or a segmented
// group moves along itself on the arrows (roving focus), a slider thumb steps on them, an open
// select list moves among its options, and a tab, an option, a select field or a switch acts on
// Enter. If nav.ts moved too, one key would act twice. So, by the attributes Reka renders (never
// by component name; none of them is on the screens written before the design system):
// - along its axis, the key belongs to the element: `keyToNav` lets the keyboard's key through
//   untouched, and a controller step is sent to the element as that same key (`sendKey`), which
//   App.vue's listener then ignores by this same rule;
// - at the ends of a roving group (the primitives turn Reka's looping off) and on the other axis,
//   nav.ts moves spatially as everywhere else; a slider and an open list have no ends;
// - Enter on a tab, an option, a select field or a switch is the element's: `confirm` sends it
//   Enter instead of a click.
// docs/specs/2026-11-18-design-system-foundations.md, Decision 6.

/** An item of a Reka roving group or a slider thumb: owns the arrows of its `data-orientation`. */
const ROVING_ITEM = '[data-reka-collection-item][data-orientation]'
/** An open Reka listbox (a Select's options): owns every arrow. */
const OPEN_LIST = '[role="listbox"][data-dismissable-layer]'
/** Reka elements that act on Enter themselves (`data-state` is Reka's). */
const ENTER_OWNER = ['tab', 'option', 'combobox', 'switch']
  .map(role => `[role="${role}"][data-state]`)
  .join(', ')

const ARROW_KEY: Record<Direction, string> = {
  up: 'ArrowUp',
  down: 'ArrowDown',
  left: 'ArrowLeft',
  right: 'ArrowRight',
}

/** Whether `el` handles the arrow of `dir` itself, so nav.ts must not move the focus. */
export function ownsArrow(el: Element, dir: Direction): boolean {
  if (el.closest(OPEN_LIST)) return true
  if (!(el instanceof HTMLElement) || !el.matches(ROVING_ITEM)) return false
  const horizontal = dir === 'left' || dir === 'right'
  if ((el.dataset.orientation === 'horizontal') !== horizontal) return false
  if (el.getAttribute('role') === 'slider') return true
  // A roving group: the item's nearest ancestor with an orientation that is not an item itself.
  const group = el.parentElement?.closest('[data-orientation]:not([data-reka-collection-item])')
  if (!group) return true
  const items = [...group.querySelectorAll<HTMLElement>('[data-reka-collection-item]')].filter(
    item =>
      !item.matches(':disabled, [data-disabled]') &&
      item.closest('[data-orientation]:not([data-reka-collection-item])') === group,
  )
  const at = items.indexOf(el)
  const forward = dir === 'right' || dir === 'down'
  return forward ? at < items.length - 1 : at > 0
}

/** Whether `el` acts on Enter itself. */
export function ownsEnter(el: Element): boolean {
  return el.matches(ENTER_OWNER)
}

function sendKey(el: HTMLElement, key: string) {
  el.dispatchEvent(
    new KeyboardEvent('keydown', { key, code: key, bubbles: true, cancelable: true }),
  )
}

/** A focused slider takes left / right as its own steps (the focus stays on it). */
function stepSlider(el: HTMLElement | null, dir: Direction): boolean {
  if (!(el instanceof HTMLInputElement) || el.type !== 'range') return false
  if (dir !== 'left' && dir !== 'right') return false
  if (dir === 'left') el.stepDown()
  else el.stepUp()
  el.dispatchEvent(new Event('input', { bubbles: true }))
  el.dispatchEvent(new Event('change', { bubbles: true }))
  return true
}

const KEY_DIRECTION: Record<string, Direction | undefined> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
}

/** Keyboard → step: arrows and Enter. Null for any other key (Esc stays with each screen). */
export function keyToNav(e: KeyboardEvent): NavAction | null {
  // A text field keeps left / right for its caret.
  const t = e.target
  const typing =
    t instanceof HTMLTextAreaElement ||
    (t instanceof HTMLInputElement && !['range', 'checkbox', 'radio', 'button'].includes(t.type))
  if (typing && (e.code === 'ArrowLeft' || e.code === 'ArrowRight')) return null
  // A key the focused element handles itself (the rule above).
  if (t instanceof Element) {
    const dir = KEY_DIRECTION[e.code]
    if (dir && ownsArrow(t, dir)) return null
    if ((e.code === 'Enter' || e.code === 'NumpadEnter') && ownsEnter(t)) return null
  }
  switch (e.code) {
    case 'ArrowUp':
      return 'up'
    case 'ArrowDown':
      return 'down'
    case 'ArrowLeft':
      return 'left'
    case 'ArrowRight':
      return 'right'
    case 'Enter':
    case 'NumpadEnter':
      return 'confirm'
    default:
      return null
  }
}

/**
 * Registers a screen for controller/keyboard navigation while it is mounted.
 * `autofocus` focuses its `data-nav-initial` element (or the first focusable) on mount.
 */
export function useNavScope(
  root: Ref<HTMLElement | null>,
  options: { back?: () => void; tab?: (dir: -1 | 1) => void; autofocus?: boolean } = {},
) {
  const scope: Scope = { root, back: options.back, tab: options.tab }
  onMounted(() => {
    scopes.push(scope)
    if (options.autofocus && root.value) focusInitial(root.value)
  })
  onBeforeUnmount(() => {
    const i = scopes.indexOf(scope)
    if (i >= 0) scopes.splice(i, 1)
  })
}
