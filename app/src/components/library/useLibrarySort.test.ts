import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { SettingsChanged, SettingsSchema } from '../../emu'
import { DEFAULT_SORT } from '../../librarySort'
import { useLibrarySort } from './useLibrarySort'

const bridge = vi.hoisted(() => ({
  page: vi.fn(),
  set: vi.fn(),
  reset: vi.fn(),
  changed: [] as Array<(e: SettingsChanged) => void>,
}))
vi.mock('../../emu', () => ({
  emu: {
    settings: {
      schema: () =>
        Promise.resolve({
          entries: [{ key: 'library.sort', page: 'library' }],
        } as unknown as SettingsSchema),
      page: bridge.page,
      set: bridge.set,
      reset: bridge.reset,
      onChanged: (cb: (e: SettingsChanged) => void) => {
        bridge.changed.push(cb)
        return Promise.resolve(() => {})
      },
    },
  },
}))

const stored = (value: unknown) => [{ key: 'library.sort', value, source: 'global' }]
const flush = () => new Promise(r => setTimeout(r, 0))

function setup() {
  const onError = vi.fn()
  let sort!: ReturnType<typeof useLibrarySort>
  mount(
    defineComponent({
      setup() {
        sort = useLibrarySort(onError)
        return () => h('div')
      },
    }),
  )
  return { sort, onError }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.changed.length = 0
  bridge.page.mockResolvedValue(stored('name'))
  bridge.set.mockResolvedValue(undefined)
  bridge.reset.mockResolvedValue(undefined)
})

describe('useLibrarySort', () => {
  it('reads library.sort on mount; an unknown value is the default', async () => {
    const { sort } = setup()
    await flush()
    expect(bridge.page).toHaveBeenCalledWith('library', { level: 'global' })
    expect(sort.sort.value).toBe('name')

    bridge.page.mockResolvedValue(stored('nonsense'))
    const other = setup()
    await flush()
    expect(other.sort.sort.value).toBe(DEFAULT_SORT)
  })

  it('writes the order picked; a failure is reported and the order read back', async () => {
    const { sort, onError } = setup()
    await flush()
    bridge.page.mockResolvedValue(stored('name'))
    bridge.set.mockRejectedValueOnce({ code: 'io' })
    await sort.setSort('added')
    expect(bridge.set).toHaveBeenCalledWith('library.sort', { level: 'global' }, 'added')
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
    expect(sort.sort.value).toBe('name')
  })

  it('reads the order again when the setting changes elsewhere, and only then', async () => {
    const { sort } = setup()
    await flush()
    const reads = bridge.page.mock.calls.length
    bridge.page.mockResolvedValue(stored('added'))
    bridge.changed.forEach(cb => cb({ key: 'general.theme', level: 'global' }))
    await flush()
    expect(bridge.page.mock.calls.length).toBe(reads)
    bridge.changed.forEach(cb => cb({ key: 'library.sort', level: 'global' }))
    await flush()
    expect(sort.sort.value).toBe('added')
  })
})
