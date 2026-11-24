// The C09 screens editor's state and what it does (Trello #28). With a game open, every change is
// shown on the game at once through the Session level (`screens.custom` «in prova» and the
// custom layout shown); «Salva» stores it for the console's games or for this game only and drops
// the preview, «Annulla» drops it and shows again the layout of before. From Settings (no game)
// it edits the page's scope directly, nothing previewed. The geometry is screenEditor.ts's.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  emu,
  type ConsoleId,
  type CustomLayout,
  type NavAction,
  type ScreenLayoutId,
  type ScreenStatus,
  type SettingsScope,
} from '@/emu'
import { latest } from '@/lib/latest'
import { readSetting } from '@/lib/settingsModel'
import type { ErrorSink } from '@/i18n/labels'
import {
  aspect,
  SCREEN_ATTR,
  grow,
  moveTo,
  pick,
  replace,
  resizeTo,
  screenOf,
  setField,
  step,
  toFront,
  front,
  type Area,
  type EditorScreen,
  type Field,
  type Proportion,
} from './screenEditor'

const SESSION: SettingsScope = { level: 'session' }

/** Where «Salva» stores the layout with a game open («Si applica a»). */
export type SaveTarget = 'console' | 'game'

export interface ScreenEditorOptions {
  console: ConsoleId
  /** Settings: the page's scope, saved to directly (no game, nothing previewed). Null: a game
   * is open, previewed live and saved for its console or for it. */
  scope: SettingsScope | null
  /** The open game's screens (null from Settings): swapped, the layout to go back to. */
  status: ScreenStatus | null
  onError: ErrorSink
  /** The editor is done (saved or cancelled). */
  close: () => void
}

/** The side panel's root carries this one. */
export const PANEL_ATTR = 'data-editor-panel'

/** How far a pointer travels before a press on a screen becomes a drag (CSS px). */
const DRAG_START = 3

export function useScreenEditor(o: ScreenEditorOptions) {
  const live = o.scope === null
  const layout = ref<CustomLayout>({ screens: [] })
  const selected = ref<EditorScreen>('bottom')
  const keep = ref(true)
  const snap = ref(true)
  /** The panel folded into the pill at the top. */
  const collapsed = ref(false)
  /** A screen is being dragged: the panel fades out. */
  const dragging = ref(false)
  /** How many «Parti da» layouts were applied: the panel does not move for them (#156). */
  const presets = ref(0)
  const target = ref<SaveTarget>('console')
  const loaded = ref(false)
  const busy = ref(false)
  const area = ref<Area>({ width: window.innerWidth, height: window.innerHeight })
  const swapped = o.status?.swapped ?? false
  /** The layout shown when the editor opened: «Annulla» goes back to it. */
  const before: ScreenLayoutId | null = o.status?.layout ?? null
  /** The open game's own scope («Solo questo gioco»). */
  const gameScope = ref<SettingsScope | null>(null)

  const current = computed(() => screenOf(layout.value, selected.value))
  const proportion = (id: EditorScreen): Proportion => ({
    keep: keep.value,
    ratio: aspect(o.console, id, swapped),
    area: area.value,
  })
  /** The scope the layout is read from and the presets resolve their gap and ratio at. */
  const readScope = (): SettingsScope => o.scope ?? SESSION
  /** Where «Salva» writes. */
  const saveScope = computed<SettingsScope | null>(() => {
    if (o.scope) return o.scope
    return target.value === 'game' ? gameScope.value : { level: 'console', console: o.console }
  })

  // ---- Loading, and «Parti da» (rule 7: only the newest answer is applied) ----
  const loads = latest()

  function apply(next: CustomLayout) {
    layout.value = next
    selected.value = front(next) ?? 'bottom'
  }

  /** Fills the screens from where `id` puts them (Rust composes it). */
  async function startFrom(id: ScreenLayoutId) {
    const fresh = loads.next()
    try {
      const { width, height } = area.value
      const next = await emu.screens.presetCustom(readScope(), o.console, id, width, height)
      if (fresh() && next.screens.length > 0) {
        apply(next)
        presets.value++
      }
    } catch (e) {
      if (fresh()) o.onError(e)
    }
  }

  async function load() {
    const fresh = loads.next()
    try {
      if (live) {
        gameScope.value = await emu.settings.gameScope()
        // Saved for this game only: «Solo questo gioco» is where it goes back to.
        const row = await readSetting('screens.custom', SESSION)
        if (row?.source === 'game') target.value = 'game'
      }
      const saved = await emu.screens.custom(readScope())
      if (!fresh()) return
      if (saved.screens.length > 0) apply(saved)
      else await startFrom(before && before !== 'custom' ? before : 'vertical')
      loaded.value = true
      // The d-pad moves the front screen at once (the hints at the bottom say so).
      focusScreen()
    } catch (e) {
      if (fresh()) o.onError(e)
    }
  }

  // ---- The live preview: one call at a time, the newest layout sent last ----
  let sending: Promise<void> | null = null
  let again = false
  let previewed = false
  let done = false

  async function sendPreview() {
    do {
      again = false
      await emu.screens.setCustom(SESSION, layout.value)
      if (!previewed) await emu.screens.setLayout('custom')
      previewed = true
    } while (again && !done)
  }
  function preview() {
    if (!live || !loaded.value || done) return
    if (sending) {
      again = true
      return
    }
    sending = sendPreview()
      .catch(o.onError)
      .finally(() => (sending = null))
  }
  watch(layout, preview, { deep: true })
  watch(loaded, preview)

  // ---- Saving and cancelling ----
  /** Waits for the preview on its way, then drops it from the Session level. */
  async function dropPreview() {
    done = true
    await sending
    if (previewed) await emu.settings.reset('screens.custom', SESSION)
  }

  async function save() {
    const to = saveScope.value
    if (busy.value || !to || layout.value.screens.length === 0) return
    busy.value = true
    try {
      await emu.screens.setCustom(to, layout.value)
      if (live) {
        await dropPreview()
        await emu.screens.setLayout('custom')
      }
      o.close()
    } catch (e) {
      o.onError(e)
    } finally {
      busy.value = false
    }
  }

  async function cancel() {
    if (busy.value) return
    busy.value = true
    try {
      if (live) {
        await dropPreview()
        if (previewed && before) await emu.screens.setLayout(before)
      }
    } catch (e) {
      o.onError(e)
    } finally {
      busy.value = false
      o.close()
    }
  }

  // ---- Changing the selected screen ----
  function update(change: (s: NonNullable<typeof current.value>) => typeof s) {
    const s = current.value
    if (s) layout.value = replace(layout.value, change(s))
  }
  const setFieldOf = (field: Field, value: number) =>
    update(s => setField(s, field, value, proportion(selected.value)))
  function select(id: EditorScreen) {
    if (screenOf(layout.value, id)) selected.value = id
  }
  function bringToFront(id: EditorScreen) {
    layout.value = toFront(layout.value, id)
  }
  /** «Mantieni le proporzioni» switched on: the height follows the width again. */
  watch(keep, on => {
    if (on) update(s => resizeTo(s, s.w, s.h, proportion(s.id as EditorScreen), false))
  })

  // ---- The pointer: drag a screen to move it, its corner to resize it ----
  let drag: {
    id: EditorScreen
    corner: boolean
    x: number
    y: number
    start: NonNullable<typeof current.value>
    pointer: number
  } | null = null

  function pointerDown(e: PointerEvent, id: EditorScreen, corner: boolean) {
    const s = screenOf(layout.value, id)
    if (!s || e.button !== 0) return
    e.preventDefault()
    select(id)
    drag = { id, corner, x: e.clientX, y: e.clientY, start: s, pointer: e.pointerId }
    window.addEventListener('pointermove', pointerMove)
    window.addEventListener('pointerup', pointerUp)
    window.addEventListener('pointercancel', pointerUp)
  }
  function pointerMove(e: PointerEvent) {
    if (!drag || e.pointerId !== drag.pointer) return
    const px = e.clientX - drag.x
    const py = e.clientY - drag.y
    if (!dragging.value && Math.hypot(px, py) < DRAG_START) return
    dragging.value = true
    const dx = px / area.value.width
    const dy = py / area.value.height
    const s = drag.start
    const next = drag.corner
      ? resizeTo(s, s.w + dx, s.h + dy, proportion(drag.id), snap.value)
      : moveTo(s, s.x + dx, s.y + dy, snap.value)
    layout.value = replace(layout.value, next)
  }
  function pointerUp(e: PointerEvent) {
    if (drag && e.pointerId !== drag.pointer) return
    drag = null
    dragging.value = false
    stopPointer()
  }
  function stopPointer() {
    window.removeEventListener('pointermove', pointerMove)
    window.removeEventListener('pointerup', pointerUp)
    window.removeEventListener('pointercancel', pointerUp)
  }

  // ---- The controller and the keyboard ----
  const onScreen = () => document.activeElement?.closest(`[${SCREEN_ATTR}]`) !== null
  /** Puts the focus on the selected screen's frame (L1 / R1, the panel folded). */
  function focusScreen() {
    void nextTick(() =>
      document.querySelector<HTMLElement>(`[${SCREEN_ATTR}="${selected.value}"]`)?.focus(),
    )
  }
  /** L1 / R1, PageUp / PageDown, [ ]: the other screen, focused so the d-pad moves it. */
  function pickNext(dir: -1 | 1) {
    select(pick(layout.value, selected.value, dir))
    focusScreen()
  }
  /** Select, «Riduci», «Mostra le opzioni»: folded, the d-pad moves the selected screen; open
   * again, the focus goes back into the panel. */
  function toggleCollapsed() {
    collapsed.value = !collapsed.value
    if (collapsed.value) return focusScreen()
    void nextTick(() =>
      document.querySelector<HTMLElement>(`[${PANEL_ATTR}] button:not(:disabled)`)?.focus(),
    )
  }
  const ARROWS: Record<string, [-1 | 0 | 1, -1 | 0 | 1]> = {
    up: [0, -1],
    down: [0, 1],
    left: [-1, 0],
    right: [1, 0],
  }
  const STICK: Record<string, [-1 | 0 | 1, -1 | 0 | 1]> = {
    stickUp: [0, -1],
    stickDown: [0, 1],
    stickLeft: [-1, 0],
    stickRight: [1, 0],
  }
  const nudge = ([dx, dy]: [-1 | 0 | 1, -1 | 0 | 1]) => update(s => step(s, dx, dy))
  const stretch = ([dx, dy]: [-1 | 0 | 1, -1 | 0 | 1]) =>
    update(s => grow(s, dx, dy, proportion(s.id as EditorScreen)))

  /**
   * The editor's own steps (nav.ts `own`): Select folds the panel, the right stick resizes the
   * selected screen; on a screen (or with the panel folded) the d-pad moves it and A saves.
   * Anything else is nav.ts's (the panel's controls, B = `back`).
   */
  function own(action: NavAction): boolean {
    if (action === 'select') {
      toggleCollapsed()
      return true
    }
    const stick = STICK[action]
    if (stick) {
      stretch(stick)
      return true
    }
    if (!collapsed.value && !onScreen()) return false
    const arrow = ARROWS[action]
    if (arrow) {
      nudge(arrow)
      return true
    }
    if (action === 'confirm') {
      void save()
      return true
    }
    return false
  }

  /** Keys nav.ts has no step for: PageUp / PageDown and [ ] pick, Shift + arrows resize. */
  function onKey(e: KeyboardEvent) {
    const dir =
      e.code === 'PageUp' || e.code === 'BracketLeft'
        ? -1
        : e.code === 'PageDown' || e.code === 'BracketRight'
          ? 1
          : 0
    const arrow = e.shiftKey ? ARROWS[e.code.replace('Arrow', '').toLowerCase()] : undefined
    if (!dir && !arrow) return
    e.preventDefault()
    e.stopImmediatePropagation()
    if (dir) pickNext(dir)
    else if (arrow) stretch(arrow)
  }

  function onResize() {
    area.value = { width: window.innerWidth, height: window.innerHeight }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKey, { capture: true })
    window.addEventListener('resize', onResize)
    void load()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey, { capture: true })
    window.removeEventListener('resize', onResize)
    stopPointer()
  })

  return {
    live,
    layout,
    selected,
    current,
    keep,
    snap,
    collapsed,
    dragging,
    presets,
    target,
    loaded,
    busy,
    swapped,
    canSave: computed(() => !!saveScope.value && layout.value.screens.length > 0),
    startFrom,
    select,
    bringToFront,
    setField: setFieldOf,
    pointerDown,
    pickNext,
    toggleCollapsed,
    own,
    save,
    cancel,
  }
}
export type ScreenEditor = ReturnType<typeof useScreenEditor>
