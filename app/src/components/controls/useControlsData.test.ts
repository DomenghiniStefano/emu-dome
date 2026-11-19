import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { PadInfo, PadState, SettingsChanged, SettingsSchema } from '@/emu'
import { padChanged, padConnected, padGone } from './devices'
import { useControlsData } from './useControlsData'

// The bridge's controller calls and events: each `on…` keeps its callback for the test to play,
// and answers at once with its own unlisten.
const bridge = vi.hoisted(() => {
  const cbs: Record<string, (x: never) => void> = {}
  const unlisten = vi.fn()
  const on = (name: string) => (cb: (x: never) => void) => {
    cbs[name] = cb
    return Promise.resolve(unlisten)
  }
  return {
    cbs,
    unlisten,
    on,
    loadSchema: vi.fn(),
    page: vi.fn(),
    pads: vi.fn(),
    watch: vi.fn(),
  }
})
vi.mock('@/emu', () => ({
  emu: {
    settings: { page: bridge.page, onChanged: bridge.on('settings') },
    controls: { pads: bridge.pads, watch: bridge.watch, onPadState: bridge.on('state') },
    onGamepadConnected: bridge.on('connected'),
    onGamepadDisconnected: bridge.on('disconnected'),
    onGamepadChanged: bridge.on('changed'),
  },
}))
// The schema is cached for the whole app run; each test here answers it afresh.
vi.mock('@/lib/settingsModel', async importOriginal => ({
  ...(await importOriginal<typeof import('@/lib/settingsModel')>()),
  loadSchema: bridge.loadSchema,
}))

const flush = () => new Promise(r => setTimeout(r, 0))
const play = <T>(name: string, value: T) => (bridge.cbs[name] as (x: T) => void)(value)
const pad = (id: number, battery: number | null = null) =>
  ({ id, name: `Pad ${id}`, family: 'xbox', vendor: 0, product: 0, battery }) as PadInfo
const schema = { entries: [] } as unknown as SettingsSchema
const abRow = { key: 'pad.ab', value: 'printed', source: 'global', changed: false }

function setup() {
  const options = { onError: vi.fn(), onPadGone: vi.fn() }
  let data!: ReturnType<typeof useControlsData>
  const wrapper = mount(
    defineComponent({
      setup() {
        data = useControlsData(options)
        return () => h('div')
      },
    }),
  )
  return { data, wrapper, ...options }
}

beforeEach(() => {
  vi.clearAllMocks()
  for (const k of Object.keys(bridge.cbs)) delete bridge.cbs[k]
  bridge.loadSchema.mockResolvedValue(schema)
  bridge.page.mockResolvedValue([abRow])
  bridge.pads.mockResolvedValue([pad(1)])
  bridge.watch.mockResolvedValue(undefined)
})

describe('pad lists', () => {
  it('adds a controller once, at the end', () => {
    expect(padConnected([pad(1)], pad(2)).map(p => p.id)).toEqual([1, 2])
    const same = [pad(1)]
    expect(padConnected(same, pad(1))).toBe(same)
  })

  it('removes the one that went, and replaces the one that changed in place', () => {
    expect(padGone([pad(1), pad(2)], 1).map(p => p.id)).toEqual([2])
    expect(padChanged([pad(1), pad(2)], pad(1, 40))).toEqual([pad(1, 40), pad(2)])
    expect(padChanged([pad(1)], pad(3))).toEqual([pad(1)])
  })
})

describe('useControlsData', () => {
  it('reads the schema, the `pad` page at «Tutti i giochi» and the controllers, then watches them', async () => {
    const { data } = setup()
    await flush()
    expect(data.schema.value).toEqual(schema)
    expect(bridge.page).toHaveBeenCalledWith('pad', { level: 'global' })
    expect(data.globalRows.value).toEqual({ 'pad.ab': abRow })
    expect(data.pads.value).toEqual([pad(1)])
    expect(bridge.watch).toHaveBeenCalledWith(true)
  })

  it('follows controllers connecting, changing and going, and says which one went', async () => {
    const { data, onPadGone } = setup()
    await flush()
    play('connected', pad(2))
    play('connected', pad(2))
    expect(data.pads.value.map(p => p.id)).toEqual([1, 2])
    play('changed', pad(2, 15))
    expect(data.pads.value[1]?.battery).toBe(15)
    play('disconnected', pad(1))
    expect(data.pads.value.map(p => p.id)).toEqual([2])
    expect(onPadGone).toHaveBeenCalledWith(1)
  })

  it('keeps the live state of the controllers as it arrives', async () => {
    const { data } = setup()
    await flush()
    const states: PadState[] = [{ id: 1, held: [], axes: [0, 0, 0, 0, 0, 0] }]
    play('state', states)
    expect(data.states.value).toEqual(states)
  })

  it('reads the page again when a `pad.*` setting changes, not for other settings', async () => {
    setup()
    await flush()
    expect(bridge.page).toHaveBeenCalledOnce()
    play<SettingsChanged>('settings', { key: 'video.fps', level: 'global' })
    play<SettingsChanged>('settings', { key: 'pad.ab', level: 'global' })
    await flush()
    expect(bridge.page).toHaveBeenCalledTimes(2)
  })

  it('a page or controller list that cannot be read is reported, and the rest still loads', async () => {
    bridge.page.mockRejectedValueOnce({ code: 'io' })
    bridge.pads.mockRejectedValueOnce({ code: 'gamepad' })
    const { data, onError } = setup()
    await flush()
    expect(onError.mock.calls).toEqual([[{ code: 'io' }], [{ code: 'gamepad' }]])
    expect(data.globalRows.value).toEqual({})
    expect(data.pads.value).toEqual([])
    play('connected', pad(3))
    expect(data.pads.value.map(p => p.id)).toEqual([3])
    expect(bridge.watch).toHaveBeenCalledWith(true)
  })

  it('a schema that cannot be read is reported, and the controllers are still followed', async () => {
    bridge.loadSchema.mockRejectedValueOnce({ code: 'io' })
    const { data, onError } = setup()
    await flush()
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
    expect(data.schema.value).toBeNull()
    expect(data.pads.value).toEqual([pad(1)])
    expect(bridge.watch).toHaveBeenCalledWith(true)
  })

  it('stops watching the live state and every listener when the section closes', async () => {
    const { wrapper } = setup()
    await flush()
    wrapper.unmount()
    expect(bridge.watch).toHaveBeenLastCalledWith(false)
    expect(bridge.unlisten).toHaveBeenCalledTimes(5)
  })

  it('closed before it finished loading, it does not start watching afterwards', async () => {
    let answer!: (s: SettingsSchema) => void
    bridge.loadSchema.mockReturnValueOnce(new Promise(r => (answer = r)))
    const { wrapper } = setup()
    wrapper.unmount()
    answer(schema)
    await flush()
    expect(bridge.watch).not.toHaveBeenCalledWith(true)
  })
})
