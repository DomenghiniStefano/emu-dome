// The C06 touch controls editor's state and what it does (Trello #29, option A of the owner's
// mockup): the layout of this console for the way the phone is held now, edited over the game.
// Tap a control to pick it, drag it to move it (its centre on the 8 px grid), pinch to resize the
// picked one; the toolbar's tools act on it; every change is a step of undo / redo, a gesture one
// step. «Salva» stores it on this device for the console and makes it the one shown
// (`emu.touch.setLayout`); leaving without saving changes nothing. The geometry is touchLayout.ts's.
// Turning the phone switches to that orientation's layout (unsaved changes are dropped).
// The game screens are elements too (Trello #134, editorScreens.ts): picked, dragged (snapped to
// the grid), resized by their corner or the size slider, keeping their proportions while `keep`;
// the layout holds every screen or none, and starts from Rust's placement (`useStartScreens`)
// until the user changes a screen.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import {
  emu,
  type ConsoleId,
  type CustomScreen,
  type NavAction,
  type ScreenId,
  type SettingsScope,
  type TouchBehaviour,
  type TouchControl,
  type TouchControlId,
  type TouchLayout,
  type TouchSetup,
} from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { NO_INSET } from '@/components/game/inset'
import { SCREEN_ATTR } from '@/components/screen-editor/screenEditor'
import { latest } from '@/lib/latest'
import { canRedo, canUndo, commit, historyOf, redo, undo, type History } from './editorHistory'
import {
  elementIds,
  isScreen,
  moveScreen,
  nextElement,
  nudgeScreen,
  ratioOf,
  resizeScreen,
  screensShown,
  withScreen,
  type ElementId,
} from './editorScreens'
import { prefsFrom } from './touchControls'
import {
  activeLayout,
  add,
  defaultLayout,
  GRID_PX,
  isDualScreen,
  minSize,
  missing,
  moveTo,
  nudge,
  orientationOf,
  pinchSize,
  remove,
  replace,
  reservedInset,
  resizeTo,
  SIZE_STEP,
  withOpacity,
  type Box,
} from './touchLayout'
import { useStartScreens } from './useStartScreens'
import { useArea, useControlBoxes } from './useTouchView'

/** The tool panel under the toolbar. */
export type ToolPanel = 'add' | 'size' | 'opacity' | 'behaviour'

export interface TouchEditorOptions {
  console: ConsoleId
  /** A game is open (the quick menu): the touch values at its session; else the console's. */
  live: boolean
  root: Ref<HTMLElement | null>
  onError: ErrorSink
  /** The editor is done (saved or left). */
  close: () => void
}

/** How far a finger travels before a press on a control becomes a drag (CSS px). */
const DRAG_START = 4
/** An opacity step from the controller (per cent). */
const OPACITY_STEP = 10

export function useTouchEditor(o: TouchEditorOptions) {
  const area = useArea()
  const orientation = computed(() => orientationOf(area.value))
  const setup = ref<TouchSetup | null>(null)
  const prefs = ref(prefsFrom([]))
  const history = ref<History<TouchLayout>>(historyOf({ controls: [], screens: [] }))
  /** The layout shown: the history's present, or a gesture's on its way. */
  const draft = ref<TouchLayout>({ controls: [], screens: [] })
  /** The element picked: a control or a game screen. */
  const selected = ref<ElementId | null>(null)
  const collapsed = ref(false)
  const panel = ref<ToolPanel | null>(null)
  /** A control is being dragged or pinched: the toolbar fades out. */
  const gesturing = ref(false)
  const busy = ref(false)
  const loaded = ref(false)
  const { boxOf } = useControlBoxes(o.root, () => draft.value)
  const scope = (): SettingsScope =>
    o.live ? { level: 'session' } : { level: 'console', console: o.console }
  /** The edges the controls keep clear of the game, as in game (TouchLayer): with no game open
   * the editor draws where the screens would be in the rest (Trello #111). Of the history's
   * present, so a drag on its way asks for nothing. */
  const reserved = computed(() =>
    reservedInset(history.value.present, area.value, boxOf, isDualScreen(o.console)),
  )
  /** Where the screens start while the layout has no placement of its own: where Rust puts them
   * now, inside the edges the controls keep clear. */
  const startScreens = useStartScreens(o.console, scope(), area, reserved, o.onError)
  /** The screens drawn: the layout's placement, else where they start. */
  const shownScreens = computed(() => screensShown(draft.value, startScreens.value))
  /** «Mantieni le proporzioni»: a screen's height follows its width. */
  const keep = ref(true)
  /** The open game shows its screens swapped: a frame holds the other screen's picture. */
  const swapped = ref(false)

  /** The picked control (none while a screen is picked). */
  const current = computed(() => draft.value.controls.find(c => c.id === selected.value))
  /** The picked screen (none while a control is picked). */
  const currentScreen = computed(() => shownScreens.value.find(s => s.id === selected.value))
  const ratio = (id: ScreenId) => ratioOf(setup.value?.screenShapes ?? [], id, swapped.value)
  const isButton = (id: TouchControlId) => setup.value?.buttons.includes(id) ?? false
  const limits = computed(() => ({
    sizeMin: setup.value?.sizeMin ?? 50,
    sizeMax: setup.value?.sizeMax ?? 200,
    opacityMin: setup.value?.opacityMin ?? 20,
    opacityMax: setup.value?.opacityMax ?? 100,
  }))
  const addable = computed(() => missing(draft.value, setup.value?.available ?? []))
  /** The edges the layout keeps clear of the game (`reserved`), of the history's present: once a
   * screen change is committed the layout places the screens itself (fractions of the whole
   * window) and nothing is kept clear. A drag on its way changes nothing here, the first one
   * included. */
  const inset = computed(() =>
    history.value.present.screens.length > 0 ? NO_INSET : reserved.value,
  )
  /** Where the layout puts the screens, live while one is dragged (empty: none placed). */
  const placement = computed(() => draft.value.screens)

  // ---- Loading (rule 7: the newest answer only) ----
  const loads = latest()

  function start() {
    const s = setup.value
    if (!s) return
    const layout = activeLayout(
      o.console,
      s,
      orientation.value,
      prefs.value[orientation.value],
      area.value,
    )
    history.value = historyOf(layout)
    draft.value = layout
    selected.value = layout.controls[0]?.id ?? null
  }

  async function load() {
    const fresh = loads.next()
    try {
      const [s, rows] = await Promise.all([
        emu.touch.layouts(o.console),
        emu.settings.page('touch', scope()),
      ])
      if (!fresh()) return
      setup.value = s
      prefs.value = prefsFrom(rows)
      start()
      loaded.value = true
      focusSelected()
    } catch (e) {
      if (fresh()) o.onError(e)
    }
  }
  watch(orientation, start)

  /** Over a game: whether it shows its screens swapped, read once (the frames' proportions). */
  const swaps = latest()
  async function loadSwapped() {
    if (!o.live) return
    const fresh = swaps.next()
    try {
      const status = await emu.screens.status()
      if (fresh()) swapped.value = status.swapped
    } catch (e) {
      if (fresh()) o.onError(e)
    }
  }

  // ---- Changes: each one a step of the history ----
  function apply(next: TouchLayout) {
    history.value = commit(history.value, next)
    draft.value = history.value.present
  }
  function change(edit: (c: TouchControl, box: Box) => TouchControl) {
    const c = current.value
    if (c) apply(replace(draft.value, edit(c, boxOf(c.id))))
  }
  function changeScreen(edit: (s: CustomScreen) => CustomScreen) {
    const s = currentScreen.value
    if (s) apply(withScreen(draft.value, startScreens.value, edit(s)))
  }
  const exists = (id: ElementId) =>
    isScreen(id)
      ? shownScreens.value.some(s => s.id === id)
      : draft.value.controls.some(c => c.id === id)
  function select(id: ElementId) {
    if (exists(id)) selected.value = id
  }
  const doUndo = () => {
    history.value = undo(history.value)
    draft.value = history.value.present
  }
  const doRedo = () => {
    history.value = redo(history.value)
    draft.value = history.value.present
  }

  /** The size slider: live while held (`final` false), one step when let go. */
  function setSize(size: number, final: boolean) {
    const c = current.value
    if (!c) return
    const next = replace(draft.value, resizeTo(c, size, area.value, boxOf(c.id), limits.value))
    if (final) apply(next)
    else draft.value = next
  }
  /** The screen size slider (per cent of the window's width): live while held, one step when
   * let go. */
  function setScreenSize(pct: number, final: boolean) {
    const s = currentScreen.value
    if (!s) return
    const { width, height } = area.value
    const resized = resizeScreen(
      s,
      (pct / 100) * width,
      s.h * height,
      keep.value,
      ratio(s.id),
      area.value,
    )
    const next = withScreen(draft.value, startScreens.value, resized)
    if (final) apply(next)
    else draft.value = next
  }
  function setKeep(on: boolean) {
    keep.value = on
  }
  function setOpacity(opacity: number, final: boolean) {
    const c = current.value
    if (!c) return
    const next = replace(draft.value, withOpacity(c, opacity, limits.value))
    if (final) apply(next)
    else draft.value = next
  }
  function setBehaviour(behaviour: TouchBehaviour) {
    if (current.value && isButton(current.value.id)) change(c => ({ ...c, behaviour }))
  }
  function addControl(id: TouchControlId) {
    apply(add(draft.value, id, o.console, orientation.value, area.value))
    selected.value = id
    panel.value = null
    focusSelected()
  }
  function removeSelected() {
    const id = selected.value
    if (!id || id === 'menu' || isScreen(id)) return
    apply(remove(draft.value, id))
    selected.value = draft.value.controls[0]?.id ?? null
  }
  function resetLayout() {
    const s = setup.value
    if (!s) return
    apply({ ...defaultLayout(o.console, orientation.value, area.value, s.available), screens: [] })
  }

  // ---- Saving and leaving ----
  async function save() {
    if (busy.value || !loaded.value || draft.value.controls.length === 0) return
    busy.value = true
    try {
      await emu.touch.setLayout(o.console, orientation.value, draft.value)
      o.close()
    } catch (e) {
      o.onError(e)
    } finally {
      busy.value = false
    }
  }
  function discard() {
    if (!busy.value) o.close()
  }

  // ---- The fingers: drag a control, pinch to resize the picked one; drag a screen or its corner ----
  const pointers = new Map<number, { x: number; y: number }>()
  let gesture: {
    layout: TouchLayout
    control: TouchControl
    /** One finger: where it went down; two: how far apart they started. */
    x: number
    y: number
    spread: number | null
  } | null = null
  /** A screen being dragged by one finger (`corner`: resized by its corner). */
  let screenGesture: {
    pointer: number
    layout: TouchLayout
    screen: CustomScreen
    x: number
    y: number
    corner: boolean
  } | null = null

  /** A press on a screen's frame (`corner`: on its resize corner; ScreenFrame's `press`). */
  function pressScreen(id: ScreenId, e: PointerEvent, corner: boolean) {
    if (!loaded.value || pointers.size > 0 || (e.pointerType === 'mouse' && e.button !== 0)) return
    const s = shownScreens.value.find(x => x.id === id)
    if (!s) return
    e.preventDefault()
    selected.value = id
    panel.value = panel.value === 'add' ? null : panel.value
    screenGesture = {
      pointer: e.pointerId,
      layout: draft.value,
      screen: s,
      x: e.clientX,
      y: e.clientY,
      corner,
    }
  }

  function moveScreenGesture(e: PointerEvent) {
    const g = screenGesture
    if (!g || g.pointer !== e.pointerId) return
    const dx = e.clientX - g.x
    const dy = e.clientY - g.y
    if (!gesturing.value && Math.hypot(dx, dy) < DRAG_START) return
    gesturing.value = true
    const { width, height } = area.value
    const s = g.screen
    const next = g.corner
      ? resizeScreen(s, s.w * width + dx, s.h * height + dy, keep.value, ratio(s.id), area.value)
      : moveScreen(s, s.x * width + dx, s.y * height + dy, area.value, true)
    draft.value = withScreen(g.layout, startScreens.value, next)
  }

  function endScreenGesture(e: PointerEvent) {
    if (screenGesture?.pointer !== e.pointerId) return
    screenGesture = null
    if (gesturing.value) apply(draft.value)
    gesturing.value = false
  }

  const spread = () => {
    const [a, b] = [...pointers.values()]
    return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : 0
  }

  function pointerDown(e: PointerEvent) {
    if (!loaded.value || (e.pointerType === 'mouse' && e.button !== 0)) return
    // A screen's frame takes its own presses (`pressScreen`); a screen being dragged takes no
    // second finger.
    if (screenGesture || (e.target as HTMLElement).closest(`[${SCREEN_ATTR}]`)) return
    const target = (e.target as HTMLElement).closest<HTMLElement>('[data-touch-control]')
    const id = target?.dataset.touchControl as TouchControlId | undefined
    if (pointers.size === 0 && !id) return
    e.preventDefault()
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
    if (pointers.size === 1 && id) {
      select(id)
      panel.value = panel.value === 'add' ? null : panel.value
    }
    const c = current.value
    if (!c) return
    gesture = {
      layout: draft.value,
      control: c,
      x: e.clientX,
      y: e.clientY,
      spread: pointers.size >= 2 ? spread() : null,
    }
  }

  function pointerMove(e: PointerEvent) {
    if (screenGesture) return moveScreenGesture(e)
    const p = pointers.get(e.pointerId)
    if (!p || !gesture) return
    p.x = e.clientX
    p.y = e.clientY
    const c = gesture.control
    const box = boxOf(c.id)
    if (gesture.spread !== null) {
      gesturing.value = true
      const size = pinchSize(c.size, gesture.spread, spread())
      draft.value = replace(gesture.layout, resizeTo(c, size, area.value, box, limits.value))
      return
    }
    const dx = e.clientX - gesture.x
    const dy = e.clientY - gesture.y
    if (!gesturing.value && Math.hypot(dx, dy) < DRAG_START) return
    gesturing.value = true
    const { width, height } = area.value
    draft.value = replace(
      gesture.layout,
      moveTo(c, c.x * width + dx, c.y * height + dy, area.value, box, true),
    )
  }

  function pointerUp(e: PointerEvent) {
    if (screenGesture) return endScreenGesture(e)
    if (!pointers.delete(e.pointerId)) return
    if (pointers.size > 0) {
      // A pinch lost a finger: the other one goes on dragging from where the control is now.
      const [rest] = [...pointers.values()]
      const c = current.value
      if (rest && c)
        gesture = { layout: draft.value, control: c, x: rest.x, y: rest.y, spread: null }
      return
    }
    gesture = null
    if (gesturing.value) apply(draft.value)
    gesturing.value = false
  }

  // ---- The controller and the keyboard (every menu works with them) ----
  const ATTR = 'data-touch-control'
  const ELEMENTS = `[${ATTR}], [${SCREEN_ATTR}]`
  const onElement = () => document.activeElement?.closest(ELEMENTS) !== null
  function focusSelected() {
    const id = selected.value
    void nextTick(() =>
      o.root.value
        ?.querySelector<HTMLElement>(`[${ATTR}="${id}"], [${SCREEN_ATTR}="${id}"]`)
        ?.focus({ preventScroll: true }),
    )
  }
  /** L1 / R1, PageUp / PageDown: the element before or after the picked one, the screens first. */
  function pickNext(dir: -1 | 1) {
    const ids = elementIds(draft.value, startScreens.value)
    if (ids.length === 0) return
    selected.value = nextElement(ids, selected.value, dir)
    focusSelected()
  }
  function toggleCollapsed() {
    collapsed.value = !collapsed.value
    panel.value = null
    if (collapsed.value) focusSelected()
  }
  const ARROWS: Record<string, [-1 | 0 | 1, -1 | 0 | 1]> = {
    up: [0, -1],
    down: [0, 1],
    left: [-1, 0],
    right: [1, 0],
  }
  function nudgeBy([dx, dy]: [-1 | 0 | 1, -1 | 0 | 1]) {
    if (currentScreen.value) changeScreen(s => nudgeScreen(s, dx, dy, area.value))
    else change((c, box) => nudge(c, dx, dy, area.value, box))
  }
  const grows = (action: NavAction) => action === 'stickUp' || action === 'stickRight'
  /** The right stick on a screen: one grid step wider / narrower (keeping the proportions the
   * height follows); without them left / right change the width, up / down the height. */
  function stretchScreen(s: CustomScreen, action: NavAction): CustomScreen {
    const by = grows(action) ? GRID_PX : -GRID_PX
    const across = keep.value || action === 'stickLeft' || action === 'stickRight'
    const { width, height } = area.value
    return resizeScreen(
      s,
      s.w * width + (across ? by : 0),
      s.h * height + (across ? 0 : by),
      keep.value,
      ratio(s.id),
      area.value,
    )
  }
  /** The right stick: up / right grow the picked element, down / left shrink it. */
  function stretch(action: NavAction) {
    if (currentScreen.value) return changeScreen(s => stretchScreen(s, action))
    const by = grows(action) ? SIZE_STEP : -SIZE_STEP
    change((c, box) => resizeTo(c, c.size + by, area.value, box, limits.value))
  }

  /**
   * The editor's own steps (nav.ts `own`): Select folds the toolbar, the right stick resizes the
   * picked element, X steps a control's opacity (nothing on a screen); on an element (or folded)
   * the d-pad moves it and A saves.
   * Anything else is nav.ts's (the toolbar's buttons, B = `back`).
   */
  function own(action: NavAction): boolean {
    if (action === 'select') {
      toggleCollapsed()
      return true
    }
    if (action.startsWith('stick')) {
      stretch(action)
      return true
    }
    if (!collapsed.value && !onElement()) return false
    const arrow = ARROWS[action]
    if (arrow) {
      nudgeBy(arrow)
      return true
    }
    if (action === 'more') {
      // On a screen (no control picked) X has nothing to step.
      const c = current.value
      if (c) {
        const next = c.opacity - OPACITY_STEP
        setOpacity(next < limits.value.opacityMin ? limits.value.opacityMax : next, true)
      }
      return true
    }
    if (action === 'confirm') {
      void save()
      return true
    }
    return false
  }

  /** Keys nav.ts has no step for: PageUp / PageDown and [ ] pick, Shift + arrows resize,
   * Ctrl + Z / Ctrl + Y undo and redo. */
  function onKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && (e.code === 'KeyZ' || e.code === 'KeyY')) {
      e.preventDefault()
      return e.code === 'KeyZ' && !e.shiftKey ? doUndo() : doRedo()
    }
    const dir =
      e.code === 'PageUp' || e.code === 'BracketLeft'
        ? -1
        : e.code === 'PageDown' || e.code === 'BracketRight'
          ? 1
          : 0
    const grow = e.shiftKey && (e.code === 'ArrowUp' || e.code === 'ArrowRight')
    const shrink = e.shiftKey && (e.code === 'ArrowDown' || e.code === 'ArrowLeft')
    if (!dir && !grow && !shrink) return
    e.preventDefault()
    e.stopImmediatePropagation()
    if (dir) pickNext(dir)
    else stretch(grow ? 'stickUp' : 'stickDown')
  }

  onMounted(() => {
    window.addEventListener('keydown', onKey, { capture: true })
    window.addEventListener('pointermove', pointerMove)
    window.addEventListener('pointerup', pointerUp)
    window.addEventListener('pointercancel', pointerUp)
    void load()
    void loadSwapped()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey, { capture: true })
    window.removeEventListener('pointermove', pointerMove)
    window.removeEventListener('pointerup', pointerUp)
    window.removeEventListener('pointercancel', pointerUp)
  })

  return {
    area,
    inset,
    orientation,
    setup,
    prefs,
    draft,
    selected,
    current,
    currentScreen,
    shownScreens,
    placement,
    keep,
    swapped,
    collapsed,
    panel,
    gesturing,
    busy,
    loaded,
    addable,
    limits,
    isButton,
    minSizeOf: (id: TouchControlId) => minSize(boxOf(id), limits.value),
    canUndo: computed(() => canUndo(history.value)),
    canRedo: computed(() => canRedo(history.value)),
    canSave: computed(() => loaded.value && draft.value.controls.length > 0),
    undo: doUndo,
    redo: doRedo,
    select,
    setSize,
    setScreenSize,
    setKeep,
    ratioOf: ratio,
    setOpacity,
    setBehaviour,
    addControl,
    removeSelected,
    resetLayout,
    save,
    discard,
    pointerDown,
    pressScreen,
    pickNext,
    toggleCollapsed,
    own,
  }
}
export type TouchEditorState = ReturnType<typeof useTouchEditor>
