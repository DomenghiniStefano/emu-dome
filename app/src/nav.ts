// Focus navigation with the controller and the keyboard, for every screen.
//
// A screen registers a scope with `useNavScope(root, { back })`; the steps (`dispatchNav`) come
// from `emu://nav` (the controller, routed in Rust) or from the arrow keys / Enter / Esc.
// Directions move the focus spatially to the nearest focusable element in that direction, by
// bounding rects, so a segmented control moves left/right within itself and up/down out of it
// without any per-screen code. `confirm` clicks the focused element, `back` calls the scope's
// `back`. Esc on the keyboard is the same `back` for a scope that asks for it (`escape`), and for
// a part of a screen that goes back on its own (`useEscapeBack`).
//
// Scopes form a tree that follows the component tree (provide / inject): a scope registered
// inside another one's component is its child. The steps go down the tree to the newest child
// with its root in the page, level by level, so a child wins over its parent whatever their
// mount order (Vue mounts a child before its parent). Two kinds:
// - a scope (`useNavScope`): a screen, a menu, a dialog; the focus moves inside its root;
// - a section (`useNavSection`): a part of a screen with its own `back` / `tab` (the shell's
//   sections); the focus still moves across the whole screen around it.
// A long grid can move the focus by row and column itself (`useNavGrid`): a step inside it asks
// the grid first, and moves spatially only when the grid lets it leave.
// `back` and `tab` bubble: from the deepest scope up, the first handler that does not return
// `false` takes the step; they stop at the first scope (a section's parent screen gets what its
// section leaves, a screen behind a dialog gets nothing).
//
// The focus ring shows only while moving without the mouse: `data-nav` on <html>
// (styles.css), set by a step or a key, cleared by the pointer. The focused element also
// carries `data-nav-focus`, because `:focus` does not match while the window is in the
// background, and the controller keeps working then (SDL reads pads in the background).
import { inject, onBeforeUnmount, onMounted, provide, type InjectionKey, type Ref } from 'vue'
import type { NavAction } from './emu'

/** A step's handler: `false` = not handled here, the step goes on to the parent. */
type Handler<A extends unknown[]> = (...args: A) => boolean | void

interface Scope {
  root: Ref<HTMLElement | null>
  back?: Handler<[]>
  /** L1 / R1 (`prev` / `next`): the screen's own tabs or levels. */
  tab?: Handler<[dir: -1 | 1]>
  /**
   * Every step, before nav.ts acts on it, for a screen that works by steps of its own (the C09
   * screens editor: the d-pad moves a screen, Select folds its panel, the right stick resizes).
   * Asked only when its scope is the one the focus moves in; `false`: nav.ts goes on as usual.
   * A section's (Settings in TV: ◀ ▶ change the focused row's value) is asked first, while the
   * focus is inside its root and no scope above it (a dialog) takes the steps.
   */
  own?: Handler<[action: NavAction]>
  /** Y (`search`, X01): the shell opens its search. Bubbles like `back`, so a dialog stops it. */
  search?: Handler<[]>
  /** A scope keeps the focus inside its root; a section (false) only adds handlers. */
  contain: boolean
  /** The scopes registered inside this one, oldest first. */
  children: Scope[]
}

/** The scopes with no scope above them, oldest first. */
const topScopes: Scope[] = []
const PARENT: InjectionKey<Scope> = Symbol('navScope')

/** The last scope of `list` that passes `test`. */
function lastOf(list: Scope[], test: (s: Scope) => unknown): Scope | undefined {
  for (let i = list.length - 1; i >= 0; i--) {
    const s = list[i]
    if (s && test(s)) return s
  }
  return undefined
}

/** The scopes that receive the steps, top down: the newest shown child at each level. */
function activePath(): Scope[] {
  const path: Scope[] = []
  let level = topScopes
  for (;;) {
    const next = lastOf(level, s => s.root.value)
    if (!next) return path
    path.push(next)
    level = next.children
  }
}

/** Where the focus moves: the deepest scope of the path (not a section). */
function movementScope(path: Scope[]): Scope | undefined {
  return lastOf(path, s => s.contain) ?? path.at(-1)
}

/** Runs `call` from the deepest scope up, until one handles it or a scope (not a section) ends it. */
function bubble(path: Scope[], call: (s: Scope) => boolean | void) {
  for (let i = path.length - 1; i >= 0; i--) {
    const s = path[i]
    if (!s) return
    if (call(s) !== false) return
    if (s.contain) return
  }
}

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

/**
 * The elements a step can land on inside `within`: focusable, not disabled (`disabled` or
 * `aria-disabled="true"`), not inside an `inert` subtree, laid out. `visibility: hidden` is
 * checked by `shown`, only on the element about to be chosen (it needs the computed style).
 */
function candidates(within: HTMLElement): HTMLElement[] {
  return [...within.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    el =>
      !el.matches(':disabled, [aria-disabled="true"]') &&
      !el.closest('[inert]') &&
      el.getClientRects().length > 0,
  )
}

const shown = (el: HTMLElement) => getComputedStyle(el).visibility !== 'hidden'

function focusables(root: HTMLElement): HTMLElement[] {
  return candidates(root).filter(shown)
}

const SCROLLS = /(auto|scroll)/

/**
 * The nearest ancestor of `el` inside `root` that scrolls (overflow auto / scroll, and content
 * larger than its box), or null. A step looks there first: on a long list or grid only its own
 * elements are measured, the rest of the screen only when nothing is there in that direction.
 */
function scrollBox(el: HTMLElement, root: HTMLElement): HTMLElement | null {
  for (let p = el.parentElement; p && p !== root && root.contains(p); p = p.parentElement) {
    const big = p.scrollHeight > p.clientHeight || p.scrollWidth > p.clientWidth
    if (!big) continue
    const style = getComputedStyle(p)
    if (SCROLLS.test(style.overflowY) || SCROLLS.test(style.overflowX)) return p
  }
  return null
}

function focus(el: HTMLElement | null | undefined) {
  if (!el) return
  el.focus({ preventScroll: true })
  el.scrollIntoView({ block: 'nearest', inline: 'nearest' })
}

/**
 * Whether the user has not moved the focus yet: it is nowhere, or still on the shell rail's
 * current item (`aria-current="page"`), where the shell's autofocus puts it while a section is
 * still loading.
 */
function focusUnmoved(): boolean {
  const active = document.activeElement
  if (!active || active === document.body) return true
  return active instanceof HTMLElement && active.getAttribute('aria-current') === 'page'
}

/**
 * Focuses `root`'s first element: the one marked `data-nav-initial`, else its first focusable.
 * `unlessMoved`: only while the user has not moved the focus yet (a section whose content
 * arrives after the shell's autofocus).
 */
export function focusInitial(root: HTMLElement, { unlessMoved = false } = {}) {
  if (unlessMoved && !focusUnmoved()) return
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

/** The nearest element of `within` in `dir` from `from`, or null. */
function nearest(from: HTMLElement, rect: DOMRect, dir: Direction, within: HTMLElement) {
  const scored: { el: HTMLElement; s: number }[] = []
  for (const el of candidates(within)) {
    if (el === from) continue
    const s = score(rect, el.getBoundingClientRect(), dir)
    if (s !== null) scored.push({ el, s })
  }
  scored.sort((a, b) => a.s - b.s)
  return scored.find(c => shown(c.el))?.el ?? null
}

/**
 * Entering a radio group from outside lands on its chosen radio, as Tab does: Reka chooses the
 * radio a keyboard arrow focuses, so landing on the nearest one would change the choice.
 */
function entering(to: HTMLElement | null, from: HTMLElement): HTMLElement | null {
  if (to?.getAttribute('role') !== 'radio') return to
  const group = to.closest('[role="radiogroup"]')
  if (!group || group.contains(from)) return to
  const chosen = group.querySelector<HTMLElement>('[role="radio"][aria-checked="true"]')
  return chosen && !chosen.matches(':disabled, [data-disabled]') ? chosen : to
}

function move(root: HTMLElement, dir: Direction) {
  const from = current(root)
  if (!from) return focusInitial(root)
  if (gridStep(from, dir)) return
  const rect = from.getBoundingClientRect()
  const box = scrollBox(from, root)
  const to = (box && nearest(from, rect, dir, box)) ?? nearest(from, rect, dir, root)
  focus(entering(to, from))
}

// ---- Grids that move by row and column
//
// A long grid (the library's covers, windowed: most tiles are not in the DOM) moves the focus
// itself instead of having every tile measured (`useNavGrid`): a step from an element inside it
// asks the grid first. The grid answers the element to focus, a promise of it (a tile it has to
// render first), or `false`: the step leaves the grid and moves spatially as everywhere else.

/** A grid's own step from `from` (an element inside it, or the grid itself) towards `dir`. */
export type GridStep = (
  from: HTMLElement,
  dir: Direction,
) => HTMLElement | Promise<HTMLElement | null> | false

const GRID = '[data-nav-grid]'
const grids = new WeakMap<Element, GridStep>()

/** Whether the grid around `from` took the step (the focus moves now or once rendered). */
function gridStep(from: HTMLElement, dir: Direction): boolean {
  const grid = from.closest(GRID)
  const step = grid && grids.get(grid)
  if (!grid || !step) return false
  const to = step(from, dir)
  if (to === false) return false
  if (to instanceof HTMLElement) {
    focus(to)
    return true
  }
  // Rendered later: focused only if the focus is still in the grid by then.
  void to.then(el => {
    if (el && grid.contains(document.activeElement)) focus(el)
  })
  return true
}

/** Registers `root` as a grid that moves the focus itself (`GridStep`) while it is mounted. */
export function useNavGrid(root: Ref<HTMLElement | null>, step: GridStep) {
  let el: HTMLElement | null = null
  onMounted(() => {
    el = root.value
    if (!el) return
    el.dataset.navGrid = ''
    grids.set(el, step)
  })
  onBeforeUnmount(() => {
    if (el) grids.delete(el)
  })
}

/** Root the focus moves in now: the deepest scope receiving the steps (not a section). */
export function activeScopeRoot(): HTMLElement | null {
  return movementScope(activePath())?.root.value ?? null
}

/**
 * Whether a section under the scope the focus moves in took `action` (`own`), the deepest first,
 * each only while the focus is inside its root.
 */
function sectionOwns(path: Scope[], scope: Scope, action: NavAction): boolean {
  const active = document.activeElement
  const below = path.slice(path.indexOf(scope) + 1).reverse()
  return below.some(
    s => !!s.own && !!active && !!s.root.value?.contains(active) && s.own(action) !== false,
  )
}

/** One step to the scopes that receive the steps (`activePath`). `ring: false` for a step that is
 * not a move without the mouse (Android's own back, #131): the focus ring stays as it was. */
export function dispatchNav(action: NavAction, opts: { ring?: boolean } = {}) {
  if (opts.ring !== false) showFocusRing()
  const path = activePath()
  const scope = movementScope(path)
  const root = scope?.root.value
  if (!root) return
  if (sectionOwns(path, scope, action)) return
  if (scope.own && scope.own(action) !== false) return
  // `select` and the right stick (`stick*`) mean something only to a scope that owns them.
  switch (action) {
    case 'back':
      bubble(path, s => (s.back ? s.back() : false))
      return
    case 'prev':
    case 'next': {
      const dir = action === 'prev' ? -1 : 1
      bubble(path, s => (s.tab ? s.tab(dir) : false))
      return
    }
    case 'search':
      bubble(path, s => (s.search ? s.search() : false))
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
// - Enter on a tab, an option, a select field, a switch or a menu item is the element's:
//   `confirm` sends it Enter instead of a click.
// An open DropdownMenu's arrows are nav.ts's: its scope holds the focus and the items are moved
// among spatially (Reka's own arrow handling did not move between our `as-child` items, measured
// headless 2026-11-21), so a menu item owns Enter only.
// A segment of a Reka date field (the DateTimeField primitive, always inside its popover) owns
// every arrow, like an open list: up and down its value, left and right along the field (Reka
// stops at its ends; nothing else is in the popover, and nav.ts would land on the field's hidden
// native input), and Enter (the primitive's «done»).
// docs/specs/2026-11-18-design-system-foundations.md, Decision 6.

/** An item of a Reka roving group or a slider thumb: owns the arrows of its `data-orientation`. */
const ROVING_ITEM = '[data-reka-collection-item][data-orientation]'
/** An open Reka listbox (a Select's options): owns every arrow. */
const OPEN_LIST = '[role="listbox"][data-dismissable-layer]'
/** A segment of a Reka date field (day, month, year, hour…; its literals are not focusable). */
const DATE_SEGMENT = '[role="spinbutton"][data-reka-date-field-segment]'
/** Reka elements that act on Enter themselves (`data-state` is Reka's; a menu item has none). */
const ENTER_OWNER = [
  ...['tab', 'option', 'combobox', 'switch'].map(role => `[role="${role}"][data-state]`),
  `[role="menuitem"]`,
  DATE_SEGMENT,
].join(', ')

const ARROW_KEY: Record<Direction, string> = {
  up: 'ArrowUp',
  down: 'ArrowDown',
  left: 'ArrowLeft',
  right: 'ArrowRight',
}

/** Whether `el` handles the arrow of `dir` itself, so nav.ts must not move the focus. */
export function ownsArrow(el: Element, dir: Direction): boolean {
  if (el.closest(OPEN_LIST)) return true
  if (el.matches(DATE_SEGMENT)) return true
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

/** Keyboard → step: arrows and Enter. Null for any other key (Esc: `useEscapeBack`). */
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
 * `autofocus` focuses its `data-nav-initial` element (or the first focusable) on mount;
 * `escape` makes Esc on the keyboard its `back` too (B = Esc). `own` sees every step first while
 * the focus moves in this scope (`Scope.own`).
 */
export function useNavScope(
  root: Ref<HTMLElement | null>,
  options: {
    back?: Handler<[]>
    tab?: Handler<[dir: -1 | 1]>
    own?: Handler<[action: NavAction]>
    search?: Handler<[]>
    autofocus?: boolean
    escape?: boolean
  } = {},
) {
  register({
    root,
    back: options.back,
    tab: options.tab,
    own: options.own,
    search: options.search,
    contain: true,
    children: [],
  })
  onMounted(() => {
    if (options.autofocus && root.value) focusInitial(root.value)
  })
  if (options.escape) useEscapeBack(root, () => void options.back?.())
}

/**
 * Registers a part of a screen with its own `back` / `tab` (a section of the shell) while it is
 * mounted. The focus keeps moving across the whole screen; a step the section does not take
 * (its handler returns `false`, or it has none) goes on to the screen. `own` sees every step
 * first while the focus is inside the section (`Scope.own`).
 */
export function useNavSection(
  root: Ref<HTMLElement | null>,
  options: { back?: Handler<[]>; tab?: Handler<[dir: -1 | 1]>; own?: Handler<[action: NavAction]> },
) {
  register({
    root,
    back: options.back,
    tab: options.tab,
    own: options.own,
    contain: false,
    children: [],
  })
}

/** Puts `scope` under the scope of the component around it (or at the top) while mounted. */
function register(scope: Scope) {
  const parent = inject(PARENT, null)
  provide(PARENT, scope)
  const siblings = parent ? parent.children : topScopes
  onMounted(() => siblings.push(scope))
  onBeforeUnmount(() => {
    const i = siblings.indexOf(scope)
    if (i >= 0) siblings.splice(i, 1)
  })
}

// ---- Esc
//
// One listener for the whole app. Esc goes to the latest registered part whose root is inside
// the scope that receives the steps: while a menu, a list or a dialog with a scope of its own is
// open above it, Esc is theirs (a Reka primitive closes on Esc itself). In game, Esc is the
// quick menu's key (useGameKeys.ts), not a step.

interface EscapeTarget {
  root: Ref<HTMLElement | null>
  back: () => void
}
const escapeTargets: EscapeTarget[] = []
let escapeListening = false

function onEscape(e: KeyboardEvent) {
  if (e.code !== 'Escape') return
  const active = activeScopeRoot()
  if (!active) return
  const target = [...escapeTargets]
    .reverse()
    .find(x => x.root.value !== null && active.contains(x.root.value))
  if (!target) return
  e.preventDefault()
  target.back()
}

/** Esc on the keyboard calls `back` while `root` is inside the scope that receives the steps. */
export function useEscapeBack(root: Ref<HTMLElement | null>, back: () => void) {
  const target: EscapeTarget = { root, back }
  onMounted(() => {
    escapeTargets.push(target)
    if (escapeListening) return
    escapeListening = true
    window.addEventListener('keydown', onEscape)
  })
  onBeforeUnmount(() => {
    const i = escapeTargets.indexOf(target)
    if (i >= 0) escapeTargets.splice(i, 1)
  })
}
