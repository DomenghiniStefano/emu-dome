// The area the touch controls are laid out in (#171): its safe edges read again when the phone
// turns from one side to the other, which changes neither the window's size nor the probe's.
import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, type Ref } from 'vue'
import type { Area } from './touchLayout'
import { useArea } from './useTouchView'

afterEach(() => {
  document.body.innerHTML = ''
  vi.unstubAllGlobals()
})

describe("the touch controls' area", () => {
  it('follows a turn to the other side, where no size changes (#171)', async () => {
    const orientation = new EventTarget()
    vi.stubGlobal('screen', { orientation })
    let area!: Ref<Area>
    mount(
      defineComponent({
        setup() {
          area = useArea()
          return () => h('div')
        },
      }),
      { attachTo: document.body },
    )
    await nextTick()
    const probe = document.body.querySelector<HTMLElement>('.pt-safe-top')
    expect(probe).not.toBeNull()
    if (!probe) return
    // The navigation bar on the right, measured.
    probe.style.paddingRight = '48px'
    window.dispatchEvent(new Event('resize'))
    expect(area.value.safe).toEqual({ top: 0, right: 48, left: 0 })
    // Turned over: the bar on the left; only the orientation event says so.
    probe.style.paddingRight = '0px'
    probe.style.paddingLeft = '48px'
    orientation.dispatchEvent(new Event('change'))
    expect(area.value.safe).toEqual({ top: 0, right: 0, left: 48 })
  })
})
