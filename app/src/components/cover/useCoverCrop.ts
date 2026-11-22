// «Adatta la copertina» (Trello #85): the step's state and what the mouse, touch, keyboard and
// controller do to it. The geometry is coverCrop.ts's; Rust makes the cover from the view
// (`library_set_cover_image`).
// Controller: the d-pad moves the image, L1 / R1 zoom (LT / RT do not reach the interface as
// steps, NavAction), X or Select switch «Intera» / «Riempi», A uses the cover while the frame has
// the focus (on a button A presses it), B goes back. Keyboard: the arrows, - / +, Enter, Esc.
// Mouse and touch: drag the image, the wheel zooms, the slider and the segmented control.
import { computed, onBeforeUnmount, onMounted, ref, type Ref } from 'vue'
import { emu, type CoverSource, type GameId, type NavAction } from '@/emu'
import { errorText } from '@/i18n/labels'
import { latest } from '@/lib/latest'
import {
  CropMode,
  initialCrop,
  moveBy,
  panStep,
  placement,
  viewOf,
  withMode,
  withZoom,
  zoomStep,
  type Crop,
  type ImageSize,
} from './coverCrop'

/** What the step frames: where the cover comes from, the image to show, its size when known. */
export interface CropSource {
  source: CoverSource
  url: string
  width: number | null
  height: number | null
}

export interface CoverCropOptions {
  game: GameId
  source: CropSource
  /** The cover is the game's now. */
  done: () => void
}

const ARROWS: Partial<Record<NavAction, 'up' | 'down' | 'left' | 'right'>> = {
  up: 'up',
  down: 'down',
  left: 'left',
  right: 'right',
}

export function useCoverCrop(o: CoverCropOptions, frame: Ref<HTMLElement | null>) {
  const known = o.source.width && o.source.height
  const size = ref<ImageSize | null>(
    known ? { width: o.source.width ?? 0, height: o.source.height ?? 0 } : null,
  )
  const crop = ref<Crop | null>(size.value ? initialCrop(size.value) : null)
  const busy = ref(false)
  const error = ref<string | null>(null)
  const requests = latest()

  const view = computed(() => (size.value && crop.value ? viewOf(size.value, crop.value) : null))
  const place = computed(() =>
    size.value && view.value ? placement(size.value, view.value) : null,
  )
  const mode = computed(() => crop.value?.mode ?? CropMode.Whole)
  const zoom = computed(() => crop.value?.zoom ?? 1)

  function change(f: (img: ImageSize, c: Crop) => Crop) {
    if (size.value && crop.value) crop.value = f(size.value, crop.value)
  }

  /** The image loaded: its size when the backend did not say it. */
  function loaded(e: Event) {
    if (size.value) return
    const img = e.target as HTMLImageElement
    if (!img.naturalWidth || !img.naturalHeight) return
    size.value = { width: img.naturalWidth, height: img.naturalHeight }
    crop.value = initialCrop(size.value)
  }

  const setMode = (m: CropMode) => change((img, c) => withMode(img, c, m))
  const toggleMode = () => setMode(mode.value === CropMode.Whole ? CropMode.Fill : CropMode.Whole)
  const setZoom = (z: number) => change((img, c) => withZoom(img, c, z))
  const zoomBy = (dir: 1 | -1) => change((img, c) => zoomStep(img, c, dir))

  async function use() {
    if (busy.value || !view.value) return
    const fresh = requests.next()
    busy.value = true
    error.value = null
    try {
      await emu.library.setCoverImage(o.game, o.source.source, view.value)
      if (fresh()) o.done()
    } catch (e) {
      if (fresh()) error.value = errorText(e)
    } finally {
      if (fresh()) busy.value = false
    }
  }

  /** The steps of nav.ts, seen first (`useNavScope` `own`). */
  function own(action: NavAction): boolean {
    const arrow = ARROWS[action]
    if (arrow) {
      change((img, c) => panStep(img, c, arrow))
      return true
    }
    switch (action) {
      case 'prev':
        zoomBy(-1)
        return true
      case 'next':
        zoomBy(1)
        return true
      case 'more':
      case 'select':
        toggleMode()
        return true
      case 'confirm':
        // On a button or a segment, A presses it; on the frame it uses the cover.
        if (document.activeElement !== frame.value) return false
        void use()
        return true
      case 'back':
      case 'search':
      case 'stickUp':
      case 'stickDown':
      case 'stickLeft':
      case 'stickRight':
      case 'up':
      case 'down':
      case 'left':
      case 'right':
        return false
    }
  }

  // Dragging the image.
  let drag: { id: number; x: number; y: number } | null = null
  function pointerDown(e: PointerEvent) {
    const el = frame.value
    if (!el || e.button !== 0) return
    el.setPointerCapture(e.pointerId)
    el.focus()
    drag = { id: e.pointerId, x: e.clientX, y: e.clientY }
  }
  function pointerMove(e: PointerEvent) {
    const el = frame.value
    if (!el || !drag || drag.id !== e.pointerId) return
    const rect = el.getBoundingClientRect()
    if (!rect.width || !rect.height) return
    const dx = (e.clientX - drag.x) / rect.width
    const dy = (e.clientY - drag.y) / rect.height
    drag = { ...drag, x: e.clientX, y: e.clientY }
    change((img, c) => moveBy(img, c, dx, dy))
  }
  function pointerUp(e: PointerEvent) {
    if (drag?.id === e.pointerId) drag = null
  }
  function wheel(e: WheelEvent) {
    e.preventDefault()
    if (e.deltaY) zoomBy(e.deltaY < 0 ? 1 : -1)
  }

  /** Keys nav.ts has no step for: - / + zoom. */
  function onKey(e: KeyboardEvent) {
    const dir =
      e.code === 'Minus' || e.code === 'NumpadSubtract'
        ? -1
        : e.code === 'Equal' || e.code === 'NumpadAdd'
          ? 1
          : 0
    if (!dir) return
    e.preventDefault()
    zoomBy(dir)
  }

  onMounted(() => window.addEventListener('keydown', onKey))
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey)
    // An answer that arrives after the step closed changes nothing here.
    requests.next()
  })

  return {
    size,
    crop,
    view,
    place,
    mode,
    zoom,
    busy,
    error,
    loaded,
    setMode,
    setZoom,
    use,
    own,
    pointerDown,
    pointerMove,
    pointerUp,
    wheel,
  }
}
