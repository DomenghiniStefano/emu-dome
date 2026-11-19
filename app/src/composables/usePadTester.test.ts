import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { PadInfo, RemapRow, SettingsSchema } from '../emu'
import { abLayout } from '../controls'
import { i18n } from '../i18n'
import { usePadTester } from './usePadTester'

const bridge = vi.hoisted(() => ({
  page: vi.fn(),
  set: vi.fn(),
  bindings: vi.fn(),
  watch: vi.fn(),
}))
vi.mock('../emu', () => ({
  emu: {
    settings: {
      schema: () =>
        Promise.resolve({ entries: [{ key: 'pad.ab', page: 'pad' }] } as unknown as SettingsSchema),
      page: bridge.page,
      set: bridge.set,
      onChanged: () => Promise.resolve(() => {}),
    },
    controls: { bindings: bridge.bindings, watch: bridge.watch },
  },
}))

const pad: PadInfo = { id: 7, name: 'Pad', family: 'xbox', vendor: 0, product: 0, battery: null }
const remap = [
  { button: 'a', source: 'east', default: 'east', changed: false },
  { button: 'b', source: 'south', default: 'south', changed: false },
] as RemapRow[]

function setup(active = true) {
  const onError = vi.fn()
  let tester!: ReturnType<typeof usePadTester>
  mount(
    defineComponent({
      setup() {
        tester = usePadTester(ref([pad]), () => active, onError)
        return () => h('div')
      },
    }),
    { global: { plugins: [i18n] } },
  )
  return { tester, onError }
}
const flush = () => new Promise(r => setTimeout(r, 0))

beforeEach(() => {
  vi.clearAllMocks()
  i18n.global.locale.value = 'en'
  bridge.page.mockResolvedValue([{ key: 'pad.ab', value: 'printed', source: 'global' }])
  bridge.bindings.mockResolvedValue({ remap })
  bridge.set.mockResolvedValue(undefined)
})

describe('abLayout', () => {
  it('is Nintendo’s for anything but `printed`', () => {
    expect(abLayout('printed')).toBe('printed')
    expect(abLayout('nintendo')).toBe('nintendo')
    expect(abLayout(undefined)).toBe('nintendo')
    expect(abLayout(3)).toBe('nintendo')
  })
})

describe('usePadTester', () => {
  it('reads the layout and the remap of the controller when the test starts', async () => {
    const { tester } = setup()
    tester.listen(true)
    await flush()
    expect(bridge.watch).toHaveBeenCalledWith(true)
    expect(tester.ab.value).toBe('printed')
    expect(bridge.bindings).toHaveBeenCalledWith('xbox', { level: 'global' })
  })

  it('a new press says what it is and what the game gets; a held one says nothing new', async () => {
    const { tester } = setup()
    tester.listen(true)
    await flush()
    tester.onPadState([{ id: 7, held: ['south'] } as never])
    expect(tester.pressed.value?.button).toBe('B')
    const first = tester.pressed.value
    tester.onPadState([{ id: 7, held: ['south'] } as never])
    expect(tester.pressed.value).toBe(first)
    tester.onPadState([{ id: 7, held: ['south', 'l1'] } as never])
    expect(tester.pressed.value?.button).toBeNull()
  })

  it('ignores presses while the step is not on screen', () => {
    const { tester } = setup(false)
    tester.onPadState([{ id: 7, held: ['south'] } as never])
    expect(tester.pressed.value).toBeNull()
  })

  it('writes the layout at «Tutti i giochi»; a failure goes to onError as it was thrown', async () => {
    const { tester, onError } = setup()
    await tester.setAb('nintendo')
    expect(tester.ab.value).toBe('nintendo')
    expect(bridge.set).toHaveBeenCalledWith('pad.ab', { level: 'global' }, 'nintendo')
    bridge.set.mockRejectedValueOnce({ code: 'io' })
    await tester.setAb('printed')
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
  })
})
