import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { StorageStatus } from '@/emu'
import { useStorageStatus } from './useStorageStatus'

// The bridge: each `storage_status` answers when the test says so; `library://changed` plays a
// change to the live callback.
const bridge = vi.hoisted(() => {
  const answers: Array<(s: StorageStatus | null) => void> = []
  let changed: (() => void) | null = null
  return {
    answers,
    status: vi.fn(() => new Promise<StorageStatus | null>(resolve => answers.push(resolve))),
    onChanged: (cb: () => void) => {
      changed = cb
      return Promise.resolve(() => (changed = null))
    },
    change: () => changed?.(),
  }
})
vi.mock('@/emu', () => ({
  emu: { storage: { status: bridge.status }, library: { onChanged: bridge.onChanged } },
}))

const flush = () => new Promise(r => setTimeout(r, 0))
const GIB = 1024 ** 3
const low: StorageStatus = { free: 380 * 1024 * 1024, total: 128 * GIB, low: true }
const roomy: StorageStatus = { free: 40 * GIB, total: 128 * GIB, low: false }

function user() {
  let api!: ReturnType<typeof useStorageStatus>
  const C = defineComponent({
    setup() {
      api = useStorageStatus()
      return () => h('div')
    },
  })
  const wrapper = mount(C)
  return { api, wrapper }
}

describe('useStorageStatus', () => {
  beforeEach(() => {
    bridge.answers.length = 0
    bridge.status.mockClear()
  })

  it('asks when mounted and keeps the answer', async () => {
    const { api } = user()
    expect(bridge.status).toHaveBeenCalledTimes(1)
    bridge.answers[0]?.(low)
    await flush()
    expect(api.status.value).toEqual(low)
  })

  it('asks again when the library changes', async () => {
    user()
    await flush()
    bridge.change()
    expect(bridge.status).toHaveBeenCalledTimes(2)
  })

  it('drops an answer older than the latest request', async () => {
    const { api } = user()
    await flush()
    bridge.change()
    bridge.answers[1]?.(roomy)
    await flush()
    bridge.answers[0]?.(low)
    await flush()
    expect(api.status.value).toEqual(roomy)
  })

  it('keeps no status when the system does not say', async () => {
    const { api } = user()
    bridge.answers[0]?.(null)
    await flush()
    expect(api.status.value).toBeNull()
  })
})
