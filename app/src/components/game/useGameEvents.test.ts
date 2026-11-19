import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { i18n } from '@/i18n'
import { inputMode } from '@/lib/inputMode'
import { gameSettingsFrom } from './session'
import { createGameState } from './gameState'
import { useGameEvents } from './useGameEvents'
import type { Toasts } from '@/composables/useToasts'

const bridge = vi.hoisted(() => ({
  loadState: vi.fn(),
  undoLoad: vi.fn(),
  stepSlot: vi.fn(),
}))
vi.mock('@/emu', () => ({
  AUTO_SLOT: 0,
  emu: {
    loadState: bridge.loadState,
    undoLoad: bridge.undoLoad,
    controls: { stepSlot: bridge.stepSlot },
    settings: { onChanged: () => Promise.resolve(() => {}) },
  },
}))

function setup(undoAllowed = true) {
  const state = createGameState()
  const settings = ref({ ...gameSettingsFrom([]), undoAllowed })
  const toasts = { notify: vi.fn(), fail: vi.fn(), failWith: vi.fn() } as unknown as Toasts & {
    notify: ReturnType<typeof vi.fn>
    failWith: ReturnType<typeof vi.fn>
  }
  const setMenu = vi.fn(async () => {})
  const stopped = vi.fn()
  let events!: ReturnType<typeof useGameEvents>
  mount(
    defineComponent({
      setup() {
        events = useGameEvents(state, settings, toasts, { setMenu, stopped })
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { state, toasts, setMenu, stopped, events }
}

beforeEach(() => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
  inputMode.value = 'keyboard'
  bridge.loadState.mockResolvedValue(undefined)
})

describe('useGameEvents', () => {
  it('a save outside the menu says so; in the menu its own line does', () => {
    const { state, toasts, events } = setup()
    state.undoLoadSlot.value = 2
    events.onEmuEvent({ type: 'stateSaved', slot: 3 } as never)
    expect(toasts.notify).toHaveBeenCalledOnce()
    expect(state.undoLoadSlot.value).toBeNull()
    state.menuOpen.value = true
    events.onEmuEvent({ type: 'stateSaved', slot: 3 } as never)
    expect(toasts.notify).toHaveBeenCalledOnce()
  })

  it('the auto quick save works silently', () => {
    const { toasts, events } = setup()
    events.onEmuEvent({ type: 'stateSaved', slot: 0 } as never)
    expect(toasts.notify).not.toHaveBeenCalled()
  })

  it('a load from the menu closes it and shows G17 with «Annulla»', async () => {
    const { state, toasts, setMenu, events } = setup()
    await events.loadFromMenu(2)
    expect(bridge.loadState).toHaveBeenCalledWith(2)
    expect(setMenu).toHaveBeenCalledWith(false)
    events.onEmuEvent({ type: 'stateLoaded', slot: 2 } as never)
    expect(state.undoLoadSlot.value).toBe(2)
    const [, options] = toasts.notify.mock.calls[0]!
    expect(options).toMatchObject({ ms: 5000, action: { label: expect.any(String) } })
  })

  it('G17 without «Annulla» when undo is off, with a note on a controller', async () => {
    const off = setup(false)
    await off.events.loadFromMenu(1)
    off.events.onEmuEvent({ type: 'stateLoaded', slot: 1 } as never)
    expect(off.toasts.notify.mock.calls[0]).toHaveLength(1)

    inputMode.value = 'controller'
    const pad = setup()
    await pad.events.loadFromMenu(1)
    pad.events.onEmuEvent({ type: 'stateLoaded', slot: 1 } as never)
    expect(pad.toasts.notify.mock.calls[0]?.[1]).toMatchObject({ note: expect.any(String) })
  })

  it('a load that fails keeps the menu open and reports the error', async () => {
    const { toasts, setMenu, events } = setup()
    bridge.loadState.mockRejectedValueOnce({ code: 'io' })
    await events.loadFromMenu(2)
    expect(setMenu).not.toHaveBeenCalled()
    expect(toasts.failWith).toHaveBeenCalledWith({ code: 'io' })
  })

  it('combos change the HUD, the slot, the speed, turbo and pause', () => {
    const { state, toasts, events } = setup()
    events.onHotkey({ kind: 'fps' } as never)
    expect(state.showFps.value).toBe(true)
    events.onHotkey({ kind: 'slot', slot: 4 } as never)
    expect(state.comboSlot.value).toBe(4)
    expect(toasts.notify).toHaveBeenCalledOnce()
    events.onHotkey({ kind: 'turbo', held: true } as never)
    expect(state.turboHeld.value).toBe(true)
    events.onHotkey({ kind: 'paused', paused: true } as never)
    expect(state.paused.value).toBe(true)
  })

  it('F2 / F3 move the slot Rust holds', async () => {
    const { state, events } = setup()
    bridge.stepSlot.mockResolvedValueOnce(5)
    await events.stepSlot(true)
    expect(bridge.stepSlot).toHaveBeenCalledWith(true)
    expect(state.comboSlot.value).toBe(5)
  })

  it('the core started gives the frame size; stopped is passed on', () => {
    const { state, stopped, events } = setup()
    events.onEmuEvent({ type: 'started', width: 240, height: 160 } as never)
    expect(state.gameSize.value).toEqual({ w: 240, h: 160 })
    events.onEmuEvent({ type: 'stopped' } as never)
    expect(stopped).toHaveBeenCalledOnce()
  })

  it('an error is reported and forgets the pending menu load', async () => {
    const { toasts, events } = setup()
    await events.loadFromMenu(2)
    events.onEmuEvent({ type: 'error', error: { code: 'io' } } as never)
    expect(toasts.failWith).toHaveBeenCalledWith({ code: 'io' })
    events.onEmuEvent({ type: 'stateLoaded', slot: 2 } as never)
    // Not the menu's load any more: a plain toast, no «Annulla».
    expect(toasts.notify.mock.calls.at(-1)).toHaveLength(1)
  })
})
