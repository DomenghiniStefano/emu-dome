import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { EmuEvent, SlotInfo } from '@/emu'
import { useQuickSlots } from './useQuickSlots'

// The core's events reach the slots through `emu.onEvent`; `late` holds the subscription until
// the test lets it answer.
const bridge = vi.hoisted(() => ({
  listSlots: vi.fn(),
  undoSave: vi.fn(),
  events: [] as Array<(e: EmuEvent) => void>,
  unlisten: vi.fn(),
  late: null as null | (() => void),
}))
vi.mock('@/emu', () => ({
  AUTO_SLOT: 0,
  emu: {
    listSlots: bridge.listSlots,
    undoSave: bridge.undoSave,
    onEvent: (cb: (e: EmuEvent) => void) =>
      new Promise<() => void>(resolve => {
        bridge.events.push(cb)
        bridge.late = () => resolve(bridge.unlisten)
      }),
  },
}))

const slot = (n: number, savedAt: number | null): SlotInfo => ({ slot: n, savedAt, thumbUrl: null })
const flush = () => new Promise(r => setTimeout(r, 0))
const emit = (e: Partial<EmuEvent>) => bridge.events.forEach(cb => cb(e as EmuEvent))

function setup() {
  const o = { onSave: vi.fn(), onLoad: vi.fn(), onError: vi.fn() }
  let slots!: ReturnType<typeof useQuickSlots>
  const wrapper = mount(
    defineComponent({
      setup() {
        slots = useQuickSlots({ count: () => 4, initial: 1, ...o })
        return () => h('div')
      },
    }),
  )
  return { slots, wrapper, ...o }
}

beforeEach(() => {
  vi.clearAllMocks()
  bridge.events.length = 0
  bridge.listSlots.mockResolvedValue([slot(0, 5), slot(1, null), slot(2, 9)])
})

describe('useQuickSlots', () => {
  it('reads as many slots as the setting asks for', async () => {
    const { slots } = setup()
    await slots.load()
    expect(bridge.listSlots).toHaveBeenCalledWith(4)
    expect(slots.chosenEmpty.value).toBe(true)
  })

  it('Carica does nothing on an empty slot', async () => {
    const { slots, onLoad } = setup()
    await slots.load()
    slots.loadChosen()
    expect(onLoad).not.toHaveBeenCalled()
    slots.pick(2)
    slots.loadChosen()
    expect(onLoad).toHaveBeenCalledWith(2)
  })

  it('a save made here leaves its «Annulla» line when its event arrives', async () => {
    const { slots, onSave } = setup()
    await flush()
    slots.pick(2)
    slots.save()
    expect(onSave).toHaveBeenCalledWith(2)
    emit({ type: 'stateSaved', slot: 2 })
    expect(slots.savedSlot.value).toBe(2)
    emit({ type: 'saveUndone' })
    expect(slots.savedSlot.value).toBeNull()
  })

  it('a save made with a shortcut leaves no line', async () => {
    const { slots } = setup()
    await flush()
    emit({ type: 'stateSaved', slot: 3 })
    expect(slots.savedSlot.value).toBeNull()
  })

  it('stops following the core when the menu closes before the bridge answers', async () => {
    const { wrapper } = setup()
    await flush()
    wrapper.unmount()
    bridge.late?.()
    await flush()
    expect(bridge.unlisten).toHaveBeenCalledOnce()
  })
})
