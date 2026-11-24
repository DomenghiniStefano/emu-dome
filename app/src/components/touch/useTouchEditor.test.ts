// The C06 editor's game screens (Trello #134): pressed, dragged, resized by their corner, by the
// size slider and by the right stick, nudged by the d-pad, walked by L1 / R1 before the controls,
// undone in one step, saved whole or not at all, and reset. The geometry is editorScreens.ts's
// (tested there); these hold the editor's wiring of it.
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { CustomScreen, TouchSetup } from '@/emu'
import { NO_INSET } from '@/components/game/inset'
import { useTouchEditor } from './useTouchEditor'

const bridge = vi.hoisted(() => ({
  layouts: vi.fn(),
  setLayouts: vi.fn(),
  page: vi.fn(),
  status: vi.fn(),
  phoneStart: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    touch: { layouts: bridge.layouts, setLayouts: bridge.setLayouts },
    settings: { page: bridge.page },
    screens: { status: bridge.status, phoneStart: bridge.phoneStart },
  },
}))

/** A phone on its side, 800 × 400 CSS px: one grid step (8 px) is 0.01 across, 0.02 down. */
const WIDTH = 800
const HEIGHT = 400
/** Where Rust starts the two screens: side by side, each 240 × 180 px (4:3). */
const START: CustomScreen[] = [
  { id: 'top', x: 0.1, y: 0.1, w: 0.3, h: 0.45 },
  { id: 'bottom', x: 0.5, y: 0.1, w: 0.3, h: 0.45 },
]
const SETUP: TouchSetup = {
  available: ['dpad', 'a', 'b', 'menu'],
  buttons: ['a', 'b'],
  screenShapes: [
    { id: 'top', width: 256, height: 192 },
    { id: 'bottom', width: 256, height: 192 },
  ],
  portrait: { controls: [], screens: [] },
  landscape: { controls: [], screens: [] },
  sizeMin: 50,
  sizeMax: 200,
  opacityMin: 20,
  opacityMax: 100,
}

const flush = () => new Promise(r => setTimeout(r, 0))

const pointer = (type: string, x: number, y: number, id = 1) =>
  new PointerEvent(type, { pointerId: id, clientX: x, clientY: y, pointerType: 'touch' })

async function setup() {
  const close = vi.fn()
  const onError = vi.fn()
  let editor!: ReturnType<typeof useTouchEditor>
  const wrapper = mount(
    defineComponent({
      setup() {
        const root = ref<HTMLElement | null>(null)
        editor = useTouchEditor({ console: 'nds', live: true, root, onError, close })
        return () => h('div', { ref: root })
      },
    }),
    { attachTo: document.body },
  )
  await flush()
  return { editor, close, onError, wrapper }
}

/** Drags the screen `id` by (`dx`, `dy`) px from a point inside it; `corner`: by its corner. */
function drag(
  editor: ReturnType<typeof useTouchEditor>,
  id: 'top' | 'bottom',
  dx: number,
  dy: number,
  corner = false,
) {
  const [x, y] = [200, 100]
  editor.pressScreen(id, pointer('pointerdown', x, y), corner)
  window.dispatchEvent(pointer('pointermove', x + dx, y + dy))
  window.dispatchEvent(pointer('pointerup', x + dx, y + dy))
}

const screen = (screens: CustomScreen[], id: string) => screens.find(s => s.id === id)

/** The phone turned: the window `width` × `height`. */
async function turn(width: number, height: number) {
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: width })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: height })
  window.dispatchEvent(new Event('resize'))
  await flush()
}

beforeEach(() => {
  vi.clearAllMocks()
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: WIDTH })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: HEIGHT })
  bridge.layouts.mockResolvedValue(SETUP)
  bridge.page.mockResolvedValue([])
  bridge.status.mockResolvedValue({ swapped: false })
  bridge.phoneStart.mockResolvedValue(START)
  bridge.setLayouts.mockResolvedValue(undefined)
})
afterEach(() => {
  document.body.innerHTML = ''
})

describe("the editor's game screens", () => {
  it('start where Rust puts them, with no placement of the layout', async () => {
    const { editor } = await setup()
    expect(editor.shownScreens.value).toEqual(START)
    expect(editor.placement.value).toEqual([])
    expect(editor.placementInset.value).not.toEqual(NO_INSET)
  })

  it('a press picks a screen; a drag moves it on the grid and takes every screen in', async () => {
    const { editor } = await setup()
    drag(editor, 'top', 17, 9)
    expect(editor.selected.value).toBe('top')
    // 80 + 17 = 97 px → 96 (the grid), 40 + 9 = 49 px → 48.
    expect(screen(editor.placement.value, 'top')).toEqual({ ...START[0], x: 0.12, y: 0.12 })
    expect(screen(editor.placement.value, 'bottom')).toEqual(START[1])
    // A held placement goes over the whole window.
    expect(editor.placementInset.value).toEqual(NO_INSET)
    expect(editor.gesturing.value).toBe(false)
  })

  it('a press that travels less than the drag threshold changes nothing', async () => {
    const { editor } = await setup()
    drag(editor, 'top', 2, 1)
    expect(editor.selected.value).toBe('top')
    expect(editor.placement.value).toEqual([])
    expect(editor.canUndo.value).toBe(false)
  })

  it('its corner resizes it keeping its proportions', async () => {
    const { editor } = await setup()
    drag(editor, 'top', 80, 0, true)
    // 240 + 80 = 320 px wide, 4:3 → 240 px high.
    expect(screen(editor.placement.value, 'top')).toEqual({ ...START[0], w: 0.4, h: 0.6 })
  })

  it('a drag is one step of undo, back to no placement', async () => {
    const { editor } = await setup()
    drag(editor, 'top', 17, 9)
    expect(editor.canUndo.value).toBe(true)
    editor.undo()
    expect(editor.placement.value).toEqual([])
    expect(editor.shownScreens.value).toEqual(START)
    expect(editor.canUndo.value).toBe(false)
  })

  it('the size slider is live while held and one step when let go', async () => {
    const { editor } = await setup()
    editor.select('top')
    editor.setScreenSize(50, false)
    expect(screen(editor.draft.value.screens, 'top')).toEqual({ ...START[0], w: 0.5, h: 0.75 })
    expect(editor.canUndo.value).toBe(false)
    editor.setScreenSize(50, true)
    expect(editor.canUndo.value).toBe(true)
  })

  it('the right stick grows the picked screen by one grid step', async () => {
    const { editor } = await setup()
    editor.select('bottom')
    expect(editor.own('stickRight')).toBe(true)
    // 240 + 8 = 248 px wide, 4:3 → 186 px high.
    expect(screen(editor.placement.value, 'bottom')).toEqual({
      ...START[1],
      w: 0.31,
      h: 0.465,
    })
  })

  it('the d-pad nudges the picked screen by one grid step', async () => {
    const { editor } = await setup()
    editor.select('top')
    // Folded (Select), the d-pad moves the picked element wherever the focus is.
    editor.own('select')
    expect(editor.own('right')).toBe(true)
    expect(editor.own('down')).toBe(true)
    expect(screen(editor.placement.value, 'top')).toEqual({ ...START[0], x: 0.11, y: 0.12 })
  })

  it('L1 / R1 walk the screens first, then the controls', async () => {
    const { editor } = await setup()
    const controls = editor.draft.value.controls.map(c => c.id)
    editor.select('bottom')
    editor.pickNext(1)
    expect(editor.selected.value).toBe(controls[0])
    editor.pickNext(-1)
    editor.pickNext(-1)
    expect(editor.selected.value).toBe('top')
    editor.pickNext(-1)
    expect(editor.selected.value).toBe(controls.at(-1))
  })

  it('saves no screens while none was changed', async () => {
    const { editor, close } = await setup()
    await editor.save()
    expect(bridge.setLayouts).toHaveBeenCalledWith(
      'nds',
      null,
      expect.objectContaining({ screens: [] }),
    )
    expect(close).toHaveBeenCalledOnce()
  })

  it('saves every screen once one was moved', async () => {
    const { editor } = await setup()
    drag(editor, 'bottom', 17, 9)
    await editor.save()
    const [, portrait, saved] = bridge.setLayouts.mock.calls[0] ?? []
    // The portrait layout was never touched: not written.
    expect(portrait).toBeNull()
    expect(saved.screens).toEqual([START[0], { ...START[1], x: 0.52, y: 0.12 }])
  })

  it('the reset gives the screens back to the game', async () => {
    const { editor } = await setup()
    drag(editor, 'top', 17, 9)
    editor.resetLayout()
    expect(editor.placement.value).toEqual([])
    expect(editor.shownScreens.value).toEqual(START)
  })
})

describe('both orientations in one session', () => {
  it('turning and back keeps the edit of the orientation left, and its undo', async () => {
    const { editor } = await setup()
    drag(editor, 'top', 17, 9)
    const edited = editor.placement.value
    await turn(HEIGHT, WIDTH)
    expect(editor.orientation.value).toBe('portrait')
    expect(editor.placement.value).toEqual([])
    expect(editor.canUndo.value).toBe(false)
    await turn(WIDTH, HEIGHT)
    expect(editor.placement.value).toEqual(edited)
    expect(editor.selected.value).toBe('top')
    editor.undo()
    expect(editor.placement.value).toEqual([])
  })

  it('«Salva» writes both orientations changed, in one call', async () => {
    const { editor, close } = await setup()
    drag(editor, 'top', 17, 9)
    const landscape = editor.draft.value
    await turn(HEIGHT, WIDTH)
    editor.select('bottom')
    editor.own('stickRight')
    const portrait = editor.draft.value
    await editor.save()
    expect(bridge.setLayouts).toHaveBeenCalledOnce()
    expect(bridge.setLayouts).toHaveBeenCalledWith('nds', portrait, landscape)
    expect(close).toHaveBeenCalledOnce()
  })

  it('suggests turning the phone until the other orientation was shown', async () => {
    const { editor } = await setup()
    expect(editor.turnHint.value).toBe('portrait')
    await turn(HEIGHT, WIDTH)
    expect(editor.turnHint.value).toBeNull()
    await turn(WIDTH, HEIGHT)
    expect(editor.turnHint.value).toBeNull()
  })
})
