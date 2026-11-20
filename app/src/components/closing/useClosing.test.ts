import { beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import type { ClosingProgress } from '@/emu'
import { flow } from '@/components/game/gameFlow'
import type { Toasts } from '@/composables/useToasts'
import { useClosing } from './useClosing'

const bridge = vi.hoisted(() => ({
  listener: null as null | ((p: ClosingProgress) => void),
  closeWait: vi.fn(),
  closeAnyway: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    onClosing: (cb: (p: ClosingProgress) => void) => {
      bridge.listener = cb
      return Promise.resolve(() => {})
    },
    closeWait: bridge.closeWait,
    closeAnyway: bridge.closeAnyway,
  },
}))

async function setup() {
  const toasts = { notify: vi.fn(), fail: vi.fn(), failWith: vi.fn() }
  const closing = effectScope().run(() => useClosing(toasts as unknown as Toasts))!
  await closing.listen()
  const send = (p: ClosingProgress) => bridge.listener?.(p)
  return { closing, toasts, send }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.listener = null
  flow.ending = false
})

describe('useClosing', () => {
  it('follows the close and expects the core to stop while it runs', async () => {
    const { closing, send } = await setup()
    send({ phase: 'saving', purpose: 'quit', error: null })
    expect(closing.view.value).toEqual({ status: 'saving', ask: null })
    expect(flow.ending).toBe(true)
    send({ phase: 'saved', purpose: 'quit', error: null })
    expect(closing.view.value.status).toBe('saved')
    send({ phase: 'closed', purpose: 'game', error: null })
    expect(closing.view.value).toEqual({ status: null, ask: null })
    expect(flow.ending).toBe(false)
  })

  it('sends the choice back', async () => {
    const { closing } = await setup()
    await closing.wait()
    await closing.closeAnyway()
    expect(bridge.closeWait).toHaveBeenCalledOnce()
    expect(bridge.closeAnyway).toHaveBeenCalledOnce()
  })

  it('shows a choice that did not reach the backend', async () => {
    const { closing, toasts } = await setup()
    const error = { code: 'noGame' }
    bridge.closeAnyway.mockRejectedValueOnce(error)
    await closing.closeAnyway()
    expect(toasts.failWith).toHaveBeenCalledWith(error)
  })
})
