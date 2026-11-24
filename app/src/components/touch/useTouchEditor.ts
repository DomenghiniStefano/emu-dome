// The C06 touch controls editor's state and what it does (Trello #29, option A of the owner's
// mockup): the layout of this console for the way the phone is held now, edited over the game.
// Tap a control to pick it, drag it to move it (its centre on the 8 px grid), pinch to resize the
// picked one; the toolbar's tools act on it; every change is a step of undo / redo, a gesture one
// step. «Salva» stores it on this device for the console and makes it the one shown
// (`emu.touch.setLayout`); leaving without saving changes nothing. The geometry is touchLayout.ts's.
// Turning the phone switches to that orientation's layout (unsaved changes are dropped).
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import {
  emu,
  type ConsoleId,
  type NavAction,
  type SettingsScope,
  type TouchBehaviour,
  type TouchControl,
  type TouchControlId,
  type TouchLayout,
  type TouchSetup,
} from '@/emu'
import type { ErrorSink } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import { canRedo, canUndo, commit, historyOf, redo, undo, type History } from './editorHistory'
import { prefsFrom } from './touchControls'
import {
  activeLayout,
  add,
  defaultLayout,
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
  const selected = ref<TouchControlId | null>(null)
  const collapsed = ref(false)
  const panel = ref<ToolPanel | null>(null)
  /** A control is being dragged or pinched: the toolbar fades out. */
  const gesturing = ref(false)
  const busy = ref(false)
  const loaded = ref(false)
  const { boxOf } = useControlBoxes(o.root, () => draft.value)

  const current = computed(() => draft.value.controls.find(c => c.id === selected.value))
  const isButton = (id: TouchControlId) => setup.value?.buttons.includes(id) ?? false
  const limits = computed(() => ({
    sizeMin: setup.value?.sizeMin ?? 50,
    sizeMax: setup.value?.sizeMax ?? 200,
    opacityMin: setup.value?.opacityMin ?? 20,
    opacityMax: setup.value?.opacityMax ?? 100,
  }))
  const addable = computed(() => missing(draft.value, setup.value?.available ?? []))
  /** The edges the layout keeps clear of the game, as in game (TouchLayer): with no game open
   * the editor draws where the screens would be in the rest (Trello #111). Of the history's
   * present, so a drag on its way asks for nothing. */
  const inset = computed(() =>
    reservedInset(history.value.present, area.value, boxOf, isDualScreen(o.console)),
  )

  // ---- Loading (rule 7: the newest answer only) ----
  const loads = latest()
  const scope = (): SettingsScope =>
    o.live ? { level: 'session' } : { level: 'console', console: o.console }

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

  // ---- Changes: each one a step of the history ----
  function apply(next: TouchLayout) {
    history.value = commit(history.value, next)
    draft.value = history.value.present
  }
  function change(edit: (c: TouchControl, box: Box) => TouchControl) {
    const c = current.value
    if (c) apply(replace(draft.value, edit(c, boxOf(c.id))))
  }
  function select(id: TouchControlId) {
    if (draft.value.controls.some(c => c.id === id)) selected.value = id
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
    if (!id || id === 'menu') return
    apply(remove(draft.value, id))
    selected.value = draft.value.controls[0]?.id ?? null
  }
  function resetLayout() {
    const s = setup.value
    if (!s) return
    apply(defaultLayout(o.console, orientation.value, area.value, s.available))
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

  // ---- The fingers: drag a control, pinch to resize the picked one ----
  const pointers = new Map<number, { x: number; y: number }>()
  let gesture: {
    layout: TouchLayout
    control: TouchControl
    /** One finger: where it went down; two: how far apart they started. */
    x: number
    y: number
    spread: number | null
  } | null = null

  const spread = () => {
    const [a, b] = [...pointers.values()]
    return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : 0
  }

  function pointerDown(e: PointerEvent) {
    if (!loaded.value || (e.pointerType === 'mouse' && e.button !== 0)) return
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
  const onControl = () => document.activeElement?.closest(`[${ATTR}]`) !== null
  function focusSelected() {
    void nextTick(() =>
      o.root.value
        ?.querySelector<HTMLElement>(`[${ATTR}="${selected.value}"]`)
        ?.focus({ preventScroll: true }),
    )
  }
  /** L1 / R1, PageUp / PageDown: the control before or after the picked one. */
  function pickNext(dir: -1 | 1) {
    const ids = draft.value.controls.map(c => c.id)
    if (ids.length === 0) return
    const i = selected.value ? ids.indexOf(selected.value) : -1
    selected.value = ids[(i + dir + ids.length) % ids.length] ?? null
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
  const nudgeBy = ([dx, dy]: [-1 | 0 | 1, -1 | 0 | 1]) =>
    change((c, box) => nudge(c, dx, dy, area.value, box))
  /** The right stick: up / right grow the picked control, down / left shrink it. */
  function stretch(action: NavAction) {
    const by = action === 'stickUp' || action === 'stickRight' ? SIZE_STEP : -SIZE_STEP
    change((c, box) => resizeTo(c, c.size + by, area.value, box, limits.value))
  }

  /**
   * The editor's own steps (nav.ts `own`): Select folds the toolbar, the right stick resizes the
   * picked control, X steps its opacity; on a control (or folded) the d-pad moves it and A saves.
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
    if (!collapsed.value && !onControl()) return false
    const arrow = ARROWS[action]
    if (arrow) {
      nudgeBy(arrow)
      return true
    }
    if (action === 'more') {
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
    setOpacity,
    setBehaviour,
    addControl,
    removeSelected,
    resetLayout,
    save,
    discard,
    pointerDown,
    pickNext,
    toggleCollapsed,
    own,
  }
}
export type TouchEditorState = ReturnType<typeof useTouchEditor>
