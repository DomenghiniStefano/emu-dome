import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { FirstRunStep, PadInfo } from '@/emu'
import { useFirstRunSteps } from './useFirstRunSteps'

const bridge = vi.hoisted(() => ({ steps: vi.fn(), done: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { firstRun: { steps: bridge.steps, done: bridge.done } } }))

const WITH_PAD: FirstRunStep[] = ['welcome', 'games', 'controller', 'ready']
const pad = { id: 1 } as PadInfo

function setup(pads: PadInfo[] = []) {
  const on = { error: vi.fn(), done: vi.fn() }
  const padList = ref(pads)
  return { steps: useFirstRunSteps(padList, on), pads: padList, ...on }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.steps.mockResolvedValue(WITH_PAD)
  bridge.done.mockResolvedValue(undefined)
})

describe('useFirstRunSteps', () => {
  it('starts on welcome with the steps without a controller, «Inizia» on the button', () => {
    const { steps } = setup()
    expect(steps.steps.value).toEqual(['welcome', 'games', 'ready'])
    expect(steps.step.value).toBe('welcome')
    expect(steps.index.value).toBe(0)
    expect(steps.mainKey.value).toBe('firstRun.start')
  })

  it('asks Rust for the steps of the controllers connected now', async () => {
    const { steps, pads } = setup()
    await steps.refresh()
    expect(bridge.steps).toHaveBeenLastCalledWith(false)
    pads.value = [pad]
    await steps.refresh()
    expect(bridge.steps).toHaveBeenLastCalledWith(true)
    expect(steps.steps.value).toEqual(WITH_PAD)
  })

  it('moves forward and back, without going before the first step', () => {
    const { steps } = setup()
    steps.back()
    expect(steps.step.value).toBe('welcome')
    steps.next()
    expect(steps.step.value).toBe('games')
    expect(steps.mainKey.value).toBe('firstRun.next')
    steps.back()
    expect(steps.step.value).toBe('welcome')
  })

  it('the controller unplugged on its step moves to the step now in its place', async () => {
    const { steps } = setup([pad])
    await steps.refresh()
    steps.next()
    steps.next()
    expect(steps.step.value).toBe('controller')
    bridge.steps.mockResolvedValueOnce(['welcome', 'games', 'ready'])
    await steps.refresh()
    expect(steps.step.value).toBe('ready')
    expect(steps.mainKey.value).toBe('firstRun.finish')
  })

  it('a refresh that fails reports it and keeps the steps and the current one', async () => {
    const { steps, error } = setup()
    steps.next()
    bridge.steps.mockRejectedValueOnce({ code: 'io' })
    await steps.refresh()
    expect(error).toHaveBeenCalledWith({ code: 'io' })
    expect(steps.steps.value).toEqual(['welcome', 'games', 'ready'])
    expect(steps.step.value).toBe('games')
  })

  it('«Fine» on the last step marks the first run done, then closes it', async () => {
    const { steps, done } = setup()
    steps.next()
    steps.next()
    expect(steps.mainKey.value).toBe('firstRun.finish')
    steps.next()
    await vi.waitFor(() => expect(done).toHaveBeenCalledOnce())
    expect(bridge.done).toHaveBeenCalledOnce()
    expect(steps.step.value).toBe('ready')
  })

  it('closes the first run even when marking it done fails, and reports the failure', async () => {
    const { steps, done, error } = setup()
    bridge.done.mockRejectedValueOnce({ code: 'io' })
    steps.next()
    steps.next()
    steps.next()
    await vi.waitFor(() => expect(done).toHaveBeenCalledOnce())
    expect(error).toHaveBeenCalledWith({ code: 'io' })
  })
})
