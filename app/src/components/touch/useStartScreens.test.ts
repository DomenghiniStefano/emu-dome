import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { CustomScreen } from '@/emu'
import { bottomInset, NO_INSET, type InsetPx } from '@/components/game/inset'
import type { Area } from './touchLayout'
import { useStartScreens } from './useStartScreens'

const bridge = vi.hoisted(() => ({ phoneStart: vi.fn() }))
vi.mock('@/emu', () => ({ emu: { screens: { phoneStart: bridge.phoneStart } } }))

const at = (y: number): CustomScreen[] => [{ id: 'main', x: 0, y, w: 1, h: 0.4 }]
const flush = () => new Promise(r => setTimeout(r, 0))

function setup() {
  const area = ref<Area>({ width: 360, height: 780 })
  const inset = ref<InsetPx>(NO_INSET)
  const onError = vi.fn()
  let screens!: ReturnType<typeof useStartScreens>
  mount(
    defineComponent({
      setup() {
        screens = useStartScreens('gba', { level: 'session' }, area, inset, onError)
        return () => h('div')
      },
    }),
  )
  return { area, inset, onError, screens }
}

beforeEach(() => {
  bridge.phoneStart.mockReset()
})

describe("where the editor's screens start", () => {
  it("is Rust's answer for the window and the edges kept clear", async () => {
    bridge.phoneStart.mockResolvedValue(at(0.1))
    const { screens } = setup()
    await flush()
    expect(screens.value).toEqual(at(0.1))
    expect(bridge.phoneStart).toHaveBeenCalledWith({ level: 'session' }, 'gba', {
      width: 360,
      height: 780,
      inset: { left: 0, top: 0, right: 0, bottom: 0, align: 'centre', between: 0 },
    })
  })

  it('is asked again when the window or the edges change, not for the same ones', async () => {
    bridge.phoneStart.mockResolvedValue(at(0.1))
    const { area, inset } = setup()
    await flush()
    area.value = { width: 360, height: 780 }
    inset.value = { ...NO_INSET }
    await flush()
    expect(bridge.phoneStart).toHaveBeenCalledTimes(1)
    inset.value = bottomInset(390)
    await flush()
    area.value = { width: 780, height: 360 }
    await flush()
    expect(bridge.phoneStart).toHaveBeenCalledTimes(3)
  })

  it('keeps the newest answer when an older one comes late', async () => {
    let first!: (s: CustomScreen[]) => void
    bridge.phoneStart.mockReturnValueOnce(new Promise(r => (first = r)))
    bridge.phoneStart.mockResolvedValueOnce(at(0.3))
    const { area, screens } = setup()
    area.value = { width: 780, height: 360 }
    await flush()
    first(at(0.1))
    await flush()
    expect(screens.value).toEqual(at(0.3))
  })

  it('says a failure, unless a newer question was asked meanwhile', async () => {
    bridge.phoneStart.mockRejectedValueOnce({ code: 'noGame', params: {} })
    const { onError } = setup()
    await flush()
    expect(onError).toHaveBeenCalledWith({ code: 'noGame', params: {} })

    let refuse!: (e: unknown) => void
    bridge.phoneStart.mockReturnValueOnce(new Promise((_, r) => (refuse = r)))
    bridge.phoneStart.mockResolvedValueOnce(at(0.2))
    const late = setup()
    late.area.value = { width: 780, height: 360 }
    await flush()
    refuse({ code: 'noGame', params: {} })
    await flush()
    expect(late.onError).not.toHaveBeenCalled()
    expect(late.screens.value).toEqual(at(0.2))
  })
})
