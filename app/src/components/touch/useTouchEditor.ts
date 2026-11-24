// The C06 touch controls editor's state and what it does (Trello #29, option A of the owner's
// mockup): the layout of this console for the way the phone is held now, edited over the game.
// Nothing is picked at first, and a tap on the ground picks nothing again (#140: the tools act on
// an element only once one is picked). Tap a control to pick it, drag it to move it (its centre on the 8 px grid), pinch to resize the
// picked one; the toolbar's tools act on it; every change is a step of undo / redo, a gesture one
// step. Turning the phone switches to that orientation's draft and back again keeps everything
// (orientationDrafts.ts: one draft, history and pick per orientation). «Salva» stores every
// orientation changed in this session (else the one on screen) on this device for the console, in
// one write, and makes them the ones shown (`emu.touch.setLayouts`); leaving without saving changes
// nothing. The geometry is touchLayout.ts's.
// The game screens are elements too (Trello #134, editorScreens.ts): picked, dragged (snapped to
// the grid), resized by their corner or the size slider, keeping their proportions while `keep`;
// the layout holds every screen or none, and starts from Rust's placement (`useStartScreens`)
// until the user changes a screen; a finger on a screen is useScreenGesture's.
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
  type TouchOrientation,
  type TouchSetup,
} from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { NO_INSET } from '@/components/game/inset'
import { SCREEN_ATTR } from '@/components/screen-editor/screenEditor'
import { latest } from '@/lib/latest'
import { canRedo, canUndo, commit, historyOf, redo, undo, type History } from './editorHistory'
import {
  asShown,
  elementIds,
  isScreen,
  nextElement,
  nudgeScreen,
  ratioOf,
  resizeScreen,
  screensShown,
  withScreen,
  type ElementId,
} from './editorScreens'
import { draftOf, layoutsToSave, turnHint, withDraft, type Drafts } from './orientationDrafts'
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
import { DRAG_START, useScreenGesture } from './useScreenGesture'
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
  /** On a phone: turning it switches orientation, so the hint suggests it. */
  phone: () => boolean
}

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
  const rustStart = useStartScreens(o.console, scope(), area, reserved, o.onError)
  /** The open game shows its screens swapped: Rust draws each screen in the other's rectangle,
   * so its start and the placement sent go through `asShown` (the editor keeps each screen by
   * what it is, #140). */
  const swapped = ref(false)
  /** Where each screen starts, by what it is (Rust's start is by rectangle). */
  const startScreens = computed(() => asShown(rustStart.value, swapped.value))
  /** The screens drawn: the layout's placement, else where they start. */
  const shownScreens = computed(() => screensShown(draft.value, startScreens.value))
  /** «Mantieni le proporzioni»: a screen's height follows its width. */
  const keep = ref(true)

  /** The picked control (none while a screen is picked). */
  const current = computed(() => draft.value.controls.find(c => c.id === selected.value))
  /** The picked screen (none while a control is picked). */
  const currentScreen = computed(() => shownScreens.value.find(s => s.id === selected.value))
  const ratio = (id: ScreenId) => ratioOf(setup.value?.screenShapes ?? [], id)
  const isButton = (id: TouchControlId) => setup.value?.buttons.includes(id) ?? false
  const limits = computed(() => ({
    sizeMin: setup.value?.sizeMin ?? 50,
    sizeMax: setup.value?.sizeMax ?? 200,
    opacityMin: setup.value?.opacityMin ?? 20,
    opacityMax: setup.value?.opacityMax ?? 100,
  }))
  const addable = computed(() => missing(draft.value, setup.value?.available ?? []))
  /** Where the layout puts the screens, live while one is dragged (empty: none placed), as Rust
   * reads it over the game as it is now (swapped or not): each screen's picture in its own frame. */
  const placement = computed(() => asShown(draft.value.screens, swapped.value))
  /** Over a game, the edges kept clear of it sent with `placement` (Rust fits a held placement
   * into the inset it has, so the two go as a pair): of the draft, so the first drag of a screen
   * already places the game over the whole window. The start frames are the game's composition
   * in `reserved`, so the switch shows no jump. */
  const placementInset = computed(() =>
    draft.value.screens.length > 0 ? NO_INSET : reserved.value,
  )

  /** The orientations left in this session, as they were left (the one on screen is live). */
  const drafts = ref<Drafts>({})
  /** The orientations shown in this session: the hint asks to turn to the other one. */
  const shown = ref<TouchOrientation[]>([])

  // ---- Loading (rule 7: the newest answer only) ----
  const loads = latest()

  /** The orientation on screen starts from its saved layout (or the default). */
  function start() {
    const s = setup.value
    if (!s) return
    if (!shown.value.includes(orientation.value)) shown.value = [...shown.value, orientation.value]
    const layout = activeLayout(
      o.console,
      s,
      orientation.value,
      prefs.value[orientation.value],
      area.value,
    )
    history.value = historyOf(layout)
    draft.value = layout
    // Nothing picked at first (#140): the tools act on an element only once the user picks one.
    selected.value = null
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
  /** The orientation on screen as a draft. */
  const onScreen = () => ({ history: history.value, selected: selected.value })
  /** The phone turned: the orientation left is kept as it is, the other one comes back as it was
   * left, or starts. */
  function turned(now: TouchOrientation, before: TouchOrientation) {
    if (!setup.value) return
    // A drag or pinch on its way holds the old orientation's layout: it ends here, with no step.
    endGestures()
    drafts.value = withDraft(drafts.value, before, onScreen())
    const kept = draftOf(drafts.value, now)
    if (!kept) return start()
    history.value = kept.history
    draft.value = kept.history.present
    selected.value = kept.selected
  }
  watch(orientation, turned)

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
    unpick()
  }
  /** Nothing picked (a tap on the ground, a removal): an element's tool panel closes with it; the
   * controls to add stay. A turn to an orientation with nothing picked closes it the same way. */
  function unpick() {
    selected.value = null
    if (panel.value !== 'add') panel.value = null
  }
  watch(selected, id => {
    if (id === null) unpick()
  })
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
      const all = withDraft(drafts.value, orientation.value, onScreen())
      const { portrait, landscape } = layoutsToSave(all, orientation.value)
      await emu.touch.setLayouts(o.console, portrait, landscape)
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
  /** A screen's frame under one finger (useScreenGesture). */
  const screenGesture = useScreenGesture({
    loaded,
    area,
    keep,
    ratio,
    draft,
    startScreens,
    shownScreens,
    gesturing,
    fingersDown: () => pointers.size > 0,
    pick: id => {
      selected.value = id
      panel.value = panel.value === 'add' ? null : panel.value
    },
    apply,
  })

  /** Every finger let go of without a step: the layout shown goes back to the history's present. */
  function endGestures() {
    screenGesture.cancel()
    pointers.clear()
    gesture = null
    gesturing.value = false
    draft.value = history.value.present
  }

  const spread = () => {
    const [a, b] = [...pointers.values()]
    return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : 0
  }

  function pointerDown(e: PointerEvent) {
    if (!loaded.value || (e.pointerType === 'mouse' && e.button !== 0)) return
    // A screen's frame takes its own presses (`pressScreen`); a screen being dragged takes no
    // second finger.
    if (screenGesture.active() || (e.target as HTMLElement).closest(`[${SCREEN_ATTR}]`)) return
    const target = (e.target as HTMLElement).closest<HTMLElement>('[data-touch-control]')
    const id = target?.dataset.touchControl as TouchControlId | undefined
    if (pointers.size === 0 && !id) {
      // A tap on the ground itself (not the toolbar or a panel over it) picks nothing.
      if (e.target === e.currentTarget) unpick()
      return
    }
    e.preventDefault()
    pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
    pressed()
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
    if (screenGesture.active()) return screenGesture.move(e)
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

  /** The window the fingers went down in: a gesture let go of in another one (the phone turned
   * under it) makes no step. */
  let pressedIn: { width: number; height: number } | null = null
  function pressed() {
    if (pointers.size <= 1) pressedIn = { width: area.value.width, height: area.value.height }
  }
  const turnedSincePress = () =>
    !!pressedIn && (pressedIn.width !== area.value.width || pressedIn.height !== area.value.height)
  function pressScreen(id: ScreenId, e: PointerEvent, corner: boolean) {
    screenGesture.press(id, e, corner)
    if (screenGesture.active()) pressed()
  }

  /** The system took the fingers away (Android sends it as the phone turns under a drag, before
   * the window turns, #140): the gesture is dropped, no step. */
  function pointerCancel(e: PointerEvent) {
    if (screenGesture.active() || pointers.has(e.pointerId)) endGestures()
  }

  function pointerUp(e: PointerEvent) {
    if ((screenGesture.active() || pointers.has(e.pointerId)) && turnedSincePress())
      return endGestures()
    if (screenGesture.active()) return screenGesture.end(e)
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
    window.addEventListener('pointercancel', pointerCancel)
    void load()
    void loadSwapped()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey, { capture: true })
    window.removeEventListener('pointermove', pointerMove)
    window.removeEventListener('pointerup', pointerUp)
    window.removeEventListener('pointercancel', pointerCancel)
  })

  return {
    area,
    orientation,
    setup,
    prefs,
    draft,
    selected,
    current,
    currentScreen,
    shownScreens,
    placement,
    placementInset,
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
    /** The orientation to turn the phone to, while it was not shown in this session. */
    turnHint: computed(() =>
      loaded.value ? turnHint(shown.value, orientation.value, o.phone()) : null,
    ),
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
