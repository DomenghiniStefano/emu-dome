import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { GameInfo, Stats } from '@/emu'
import { createGameState } from './gameState'
import { useGameEnding } from './useGameEnding'

const bridge = vi.hoisted(() => ({ stats: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { stats: bridge.stats } }))

const stats = (frames: number) =>
  ({ running: true, emuFps: 60, targetFps: 60, speed: null, paused: false, frames }) as Stats

function setup() {
  const state = createGameState()
  state.game.value = { title: 'MINISH' } as GameInfo
  const wrapper = mount(
    defineComponent({
      setup() {
        useGameEnding(state).watchStats()
        return () => h('div')
      },
    }),
  )
  return { state, wrapper }
}

beforeEach(() => {
  vi.useFakeTimers()
  vi.clearAllMocks()
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
})
