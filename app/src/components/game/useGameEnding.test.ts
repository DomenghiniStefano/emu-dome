import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { GameInfo, Stats } from '@/emu'
import { HANG_MS, SLOW_MS, flow, gameStarted } from './gameFlow'
import { createGameState } from './gameState'
import { SAVE_WAIT_MS, useGameEnding } from './useGameEnding'

const bridge = vi.hoisted(() => ({ stats: vi.fn(), closeGame: vi.fn(), saveState: vi.fn() }))
vi.mock('@/emu', () => ({
  emu: { stats: bridge.stats, closeGame: bridge.closeGame, saveState: bridge.saveState },
}))

const stats = (frames: number, over: Partial<Stats> = {}) =>
  ({
    running: true,
    emuFps: 60,
    targetFps: 60,
    speed: null,
    paused: false,
    frames,
    ...over,
  }) as Stats

function setup() {
  const state = createGameState()
  state.game.value = { title: 'MINISH', consoleId: 'gba', autosaveOnExit: true } as GameInfo
  let ending!: ReturnType<typeof useGameEnding>
  const wrapper = mount(
    defineComponent({
      setup() {
        ending = useGameEnding(state)
        ending.watchStats()
        return () => h('div')
      },
    }),
  )
  return { state, wrapper, ending }
}

beforeEach(() => {
  vi.useFakeTimers()
  vi.clearAllMocks()
  gameStarted()
  Object.assign(flow, {
    crashed: null,
    slow: null,
    closing: null,
    closeAsk: null,
    ending: false,
    resumeAsk: false,
  })
})
afterEach(() => vi.useRealTimers())

describe('the stats poll', () => {
  it('asks once at a time: a slow answer is not overtaken by the next tick', async () => {
    let answer!: (s: Stats) => void
    bridge.stats.mockImplementationOnce(() => new Promise<Stats>(r => (answer = r)))
    const { state, wrapper } = setup()
    await vi.advanceTimersByTimeAsync(1600)
    expect(bridge.stats).toHaveBeenCalledOnce()
    answer(stats(10))
    bridge.stats.mockResolvedValue(stats(20))
    await vi.advanceTimersByTimeAsync(500)
    expect(bridge.stats).toHaveBeenCalledTimes(2)
    expect(state.stats.value?.frames).toBe(20)
    wrapper.unmount()
  })

  it('drops an answer for a game that is no longer open', async () => {
    let answer!: (s: Stats) => void
    bridge.stats.mockImplementationOnce(() => new Promise<Stats>(r => (answer = r)))
    const { state, wrapper } = setup()
    await vi.advanceTimersByTimeAsync(500)
    state.game.value = { title: 'KIRBY' } as GameInfo
    answer(stats(10))
    await vi.advanceTimersByTimeAsync(0)
    expect(state.stats.value).toBeNull()
    wrapper.unmount()
  })

  it('a failed read is logged and the next tick asks again', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    bridge.stats.mockRejectedValueOnce(new Error('ipc')).mockResolvedValue(stats(5))
    const { state, wrapper } = setup()
    await vi.advanceTimersByTimeAsync(1000)
    expect(warn).toHaveBeenCalledOnce()
    expect(state.stats.value?.frames).toBe(5)
    warn.mockRestore()
    wrapper.unmount()
  })

  it('frames standing still while the game runs open E11, once, and close the menu', async () => {
    bridge.stats.mockResolvedValue(stats(7))
    const { state, wrapper } = setup()
    await vi.advanceTimersByTimeAsync(HANG_MS + 1000)
    expect(flow.crashed?.console).toBe('gba')
    const first = flow.crashed
    state.menuOpen.value = false
    await vi.advanceTimersByTimeAsync(HANG_MS * 2)
    expect(flow.crashed).toBe(first)
    wrapper.unmount()
  })

  it('a paused game or an open menu is never a hang', async () => {
    bridge.stats.mockResolvedValue(stats(7))
    const { state, wrapper } = setup()
    state.paused.value = true
    await vi.advanceTimersByTimeAsync(HANG_MS * 2)
    state.paused.value = false
    state.menuOpen.value = true
    await vi.advanceTimersByTimeAsync(HANG_MS * 2)
    expect(flow.crashed).toBeNull()
    wrapper.unmount()
  })

  it('a slow game at 1× opens E12 with what was measured', async () => {
    let f = 0
    bridge.stats.mockImplementation(() =>
      Promise.resolve(stats(f++, { emuFps: 40, speed: { kind: 'factor', value: 1 } })),
    )
    const { wrapper } = setup()
    await vi.advanceTimersByTimeAsync(SLOW_MS + 1000)
    expect(flow.slow).toEqual({ fps: 40, target: 60 })
    wrapper.unmount()
  })
})

describe('the game ending', () => {
  it('G14: the closing shows while the core saves, at least 400 ms, then the game is gone', async () => {
    let done!: () => void
    bridge.closeGame.mockImplementation(() => new Promise<void>(r => (done = r)))
    const { state, ending, wrapper } = setup()
    state.menuOpen.value = true
    const closing = ending.close()
    expect(flow.closing).toEqual({ title: 'MINISH', autosave: true })
    expect(state.menuOpen.value).toBe(false)
    // The core's `stopped` while closing on purpose is expected: not E11.
    ending.stopped()
    expect(flow.crashed).toBeNull()
    done()
    await vi.advanceTimersByTimeAsync(399)
    expect(state.game.value).not.toBeNull()
    await vi.advanceTimersByTimeAsync(1)
    await closing
    expect(flow.closing).toBeNull()
    expect(state.game.value).toBeNull()
    wrapper.unmount()
  })

  it('a failed close is reported and still goes back to the shell', async () => {
    const error = vi.spyOn(console, 'error').mockImplementation(() => {})
    bridge.closeGame.mockRejectedValue(new Error('ipc'))
    const { state, ending, wrapper } = setup()
    const closing = ending.close()
    await vi.advanceTimersByTimeAsync(400)
    await closing
    expect(error).toHaveBeenCalledWith(new Error('ipc'))
    expect(state.game.value).toBeNull()
    error.mockRestore()
    wrapper.unmount()
  })

  it('nothing unsaved: «Chiudi il gioco» closes without asking', async () => {
    bridge.stats.mockResolvedValue(stats(10, { unsaved: false }))
    bridge.closeGame.mockResolvedValue(undefined)
    const { state, ending, wrapper } = setup()
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(400)
    expect(await closing).toBe(true)
    expect(flow.closeAsk).toBeNull()
    expect(bridge.closeGame).toHaveBeenCalledOnce()
    expect(state.game.value).toBeNull()
    wrapper.unmount()
  })

  it('progress not saved: asks first; «Annulla» keeps the game open', async () => {
    bridge.stats.mockResolvedValue(stats(10, { unsaved: true }))
    const { state, ending, wrapper } = setup()
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(0)
    expect(flow.closeAsk).toEqual({ autosave: true, saving: false })
    // A second press while asked does nothing.
    expect(await ending.requestClose()).toBe(false)
    ending.answerClose('cancel')
    expect(await closing).toBe(false)
    expect(flow.closeAsk).toBeNull()
    expect(bridge.closeGame).not.toHaveBeenCalled()
    expect(state.game.value).not.toBeNull()
    wrapper.unmount()
  })

  it('«Chiudi senza salvare» closes as before, with no quick save', async () => {
    bridge.stats.mockResolvedValue(stats(10, { unsaved: true }))
    bridge.closeGame.mockResolvedValue(undefined)
    const { state, ending, wrapper } = setup()
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(0)
    ending.answerClose('discard')
    await vi.advanceTimersByTimeAsync(400)
    expect(await closing).toBe(true)
    expect(bridge.saveState).not.toHaveBeenCalled()
    expect(state.game.value).toBeNull()
    wrapper.unmount()
  })

  it('«Salva e chiudi» saves into the quick slot and closes once it is written', async () => {
    bridge.stats.mockResolvedValue(stats(10, { unsaved: true }))
    bridge.closeGame.mockResolvedValue(undefined)
    bridge.saveState.mockResolvedValue(undefined)
    const { state, ending, wrapper } = setup()
    state.comboSlot.value = 3
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(0)
    ending.answerClose('save')
    await vi.advanceTimersByTimeAsync(0)
    expect(bridge.saveState).toHaveBeenCalledWith(3)
    expect(flow.closeAsk?.saving).toBe(true)
    // Another slot's save, or the auto quick save, is not the one waited for.
    ending.onSaveEvent({ type: 'stateSaved', slot: 1 })
    await vi.advanceTimersByTimeAsync(0)
    expect(bridge.closeGame).not.toHaveBeenCalled()
    ending.onSaveEvent({ type: 'stateSaved', slot: 3 })
    await vi.advanceTimersByTimeAsync(400)
    expect(await closing).toBe(true)
    expect(bridge.closeGame).toHaveBeenCalledOnce()
    expect(state.game.value).toBeNull()
    wrapper.unmount()
  })

  it('«Salva e chiudi» whose save fails keeps the game open', async () => {
    bridge.stats.mockResolvedValue(stats(10, { unsaved: true }))
    bridge.saveState.mockResolvedValue(undefined)
    const { state, ending, wrapper } = setup()
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(0)
    ending.answerClose('save')
    await vi.advanceTimersByTimeAsync(0)
    ending.onSaveEvent({
      type: 'error',
      error: { code: 'stateSaveFailed', params: { slot: 1, cause: 'other' } },
    })
    expect(await closing).toBe(false)
    expect(flow.closeAsk).toBeNull()
    expect(bridge.closeGame).not.toHaveBeenCalled()
    expect(state.game.value).not.toBeNull()
    wrapper.unmount()
  })

  it('«Salva e chiudi» with no answer in time reports it and keeps the game open', async () => {
    const error = vi.spyOn(console, 'error').mockImplementation(() => {})
    bridge.stats.mockResolvedValue(stats(10, { unsaved: true }))
    bridge.saveState.mockResolvedValue(undefined)
    const { ending, wrapper } = setup()
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(0)
    ending.answerClose('save')
    await vi.advanceTimersByTimeAsync(SAVE_WAIT_MS)
    expect(await closing).toBe(false)
    expect(error).toHaveBeenCalledWith({
      code: 'stateSaveFailed',
      params: { slot: 1, cause: 'other' },
    })
    expect(bridge.closeGame).not.toHaveBeenCalled()
    error.mockRestore()
    wrapper.unmount()
  })

  it('the game gone while asked (the window closed) drops the question', async () => {
    bridge.stats.mockResolvedValue(stats(10, { unsaved: true }))
    const { ending, wrapper } = setup()
    const closing = ending.requestClose()
    await vi.advanceTimersByTimeAsync(0)
    ending.end()
    expect(await closing).toBe(false)
    expect(flow.closeAsk).toBeNull()
    expect(bridge.closeGame).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('the core stopping by itself is E11; during the close flow (#86) it is not', () => {
    const { ending, wrapper } = setup()
    flow.ending = true
    ending.stopped()
    expect(flow.crashed).toBeNull()
    flow.ending = false
    ending.stopped()
    expect(flow.crashed?.console).toBe('gba')
    wrapper.unmount()
  })
})
