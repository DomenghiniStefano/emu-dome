// The sections' room the phone's shell reports for its toasts (#142): measured again when the
// phone turns from one side to the other, which changes no size (only where the bars are).
import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, ref, type Ref } from 'vue'
import { provideShellArea, useShellAreaReport, type ShellArea } from './useShellArea'

const W = 780
const H = 360
/** The sections' box: the column on one side, the navigation bar on the other. */
let box = { left: 72, right: W - 48 }
const rect = () =>
  ({ ...box, top: 0, bottom: H, width: box.right - box.left, height: H }) as DOMRect

afterEach(() => {
  document.body.innerHTML = ''
  vi.unstubAllGlobals()
})

async function setup() {
  const orientation = new EventTarget()
  vi.stubGlobal('screen', { orientation })
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: W })
  Object.defineProperty(window, 'innerHeight', { configurable: true, value: H })
  let area!: Ref<ShellArea | null>
  const Shell = defineComponent({
    setup() {
      const el = ref<HTMLElement | null>(null)
      useShellAreaReport(el, () => true)
      return () => h('div', { ref: el })
    },
    mounted() {
      ;(this.$el as HTMLElement).getBoundingClientRect = rect
    },
  })
  mount(
    defineComponent({
      setup() {
        area = provideShellArea()
        return () => h(Shell)
      },
    }),
    { attachTo: document.body },
  )
  await nextTick()
  return { area, orientation }
}

describe("the phone shell's room for the toasts", () => {
  it('follows a turn to the other side, where no size changes (#142)', async () => {
    box = { left: 72, right: W - 48 }
    const { area, orientation } = await setup()
    window.dispatchEvent(new Event('resize'))
    expect(area.value).toEqual({ left: 72, right: 48, bottom: 0 })
    // Turned over: the navigation bar on the left, the column on the right; same sizes.
    box = { left: 48, right: W - 72 }
    orientation.dispatchEvent(new Event('change'))
    expect(area.value).toEqual({ left: 48, right: 72, bottom: 0 })
  })
})
