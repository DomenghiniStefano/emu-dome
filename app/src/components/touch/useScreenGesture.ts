// The C06 editor's game screens under one finger (Trello #134): a press on a screen's frame picks
// it, a drag past `DRAG_START` moves it (snapped to the grid) or, from its corner, resizes it
// (keeping its proportions while `keep`); letting go makes the drag one step of the history.
// The geometry is editorScreens.ts's; the editor (useTouchEditor) owns the state it changes.
import type { Ref } from 'vue'
import type { CustomScreen, ScreenId, TouchLayout } from '@/emu'
import { moveScreen, resizeScreen, withScreen } from './editorScreens'
import type { Area } from './touchLayout'

/** How far a finger travels before a press on an element becomes a drag (CSS px). */
export const DRAG_START = 4

export interface ScreenGestureOptions {
  /** The editor has its layout: before that a press does nothing. */
  loaded: Readonly<Ref<boolean>>
  area: Readonly<Ref<Area>>
  /** «Mantieni le proporzioni». */
  keep: Readonly<Ref<boolean>>
  /** Width over height of the picture the frame `id` shows. */
  ratio: (id: ScreenId) => number
  /** The layout shown, written live while dragging. */
  draft: Ref<TouchLayout>
  /** Where the screens start while the layout has none of its own. */
  startScreens: Readonly<Ref<CustomScreen[]>>
  /** The screens drawn: the layout's, else where they start. */
  shownScreens: Readonly<Ref<CustomScreen[]>>
  /** A drag is on its way (the toolbar fades out). */
  gesturing: Ref<boolean>
  /** Fingers already on a control: a screen takes no press meanwhile. */
  fingersDown: () => boolean
  /** The screen pressed becomes the picked element. */
  pick: (id: ScreenId) => void
  /** One step of the history. */
  apply: (next: TouchLayout) => void
}

export function useScreenGesture(o: ScreenGestureOptions) {
  /** A screen being dragged by one finger (`corner`: resized by its corner). */
  let gesture: {
    pointer: number
    layout: TouchLayout
    screen: CustomScreen
    x: number
    y: number
    corner: boolean
  } | null = null

  /** A press on a screen's frame (`corner`: on its resize corner; ScreenFrame's `press`). */
  function press(id: ScreenId, e: PointerEvent, corner: boolean) {
    if (!o.loaded.value || o.fingersDown() || (e.pointerType === 'mouse' && e.button !== 0)) return
    const s = o.shownScreens.value.find(x => x.id === id)
    if (!s) return
    e.preventDefault()
    o.pick(id)
    gesture = {
      pointer: e.pointerId,
      layout: o.draft.value,
      screen: s,
      x: e.clientX,
      y: e.clientY,
      corner,
    }
  }

  function move(e: PointerEvent) {
    const g = gesture
    if (!g || g.pointer !== e.pointerId) return
    const dx = e.clientX - g.x
    const dy = e.clientY - g.y
    if (!o.gesturing.value && Math.hypot(dx, dy) < DRAG_START) return
    o.gesturing.value = true
    const { width, height } = o.area.value
    const s = g.screen
    const next = g.corner
      ? resizeScreen(
          s,
          s.w * width + dx,
          s.h * height + dy,
          o.keep.value,
          o.ratio(s.id),
          o.area.value,
        )
      : moveScreen(s, s.x * width + dx, s.y * height + dy, o.area.value, true)
    o.draft.value = withScreen(g.layout, o.startScreens.value, next)
  }

  function end(e: PointerEvent) {
    if (gesture?.pointer !== e.pointerId) return
    gesture = null
    if (o.gesturing.value) o.apply(o.draft.value)
    o.gesturing.value = false
  }

  /** Drops the gesture on its way without a step (the phone turned under it). */
  function cancel() {
    gesture = null
  }

  return {
    press,
    move,
    end,
    cancel,
    /** A screen is under a finger: the pointer's moves and release are the gesture's. */
    active: () => gesture !== null,
  }
}
