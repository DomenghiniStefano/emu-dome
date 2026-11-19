import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { pressedOutside, useDismiss } from './useDismiss'

describe('pressedOutside', () => {
  const area = document.createElement('div')
  const inside = area.appendChild(document.createElement('button'))
  const elsewhere = document.createElement('div')

  it('is outside only for a node out of the area', () => {
    expect(pressedOutside(area, elsewhere)).toBe(true)
    expect(pressedOutside(area, inside)).toBe(false)
    expect(pressedOutside(area, area)).toBe(false)
  })

  it('is never outside without an area or a node', () => {
    expect(pressedOutside(null, elsewhere)).toBe(false)
    expect(pressedOutside(area, null)).toBe(false)
  })
})

describe('useDismiss', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  /** A toggle and its popover side by side in one area, plus something else on the page. */
  function popover() {
    const close = vi.fn()
    const Popover = defineComponent({
      setup() {
        const root = ref<HTMLElement | null>(null)
        useDismiss(root, close)
        return () => h('div', { ref: root, class: 'popover' })
      },
    })
    const wrapper = mount(
      () => h('main', [h('div', { class: 'area' }, [h('button'), h(Popover)]), h('p')]),
      { attachTo: document.body },
    )
    const press = (selector: string) =>
      document
        .querySelector(selector)
        ?.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    return { close, wrapper, press }
  }

  it('closes on a press outside the area, not on the toggle or the popover', () => {
    const { close, wrapper, press } = popover()
    press('.area button')
    press('.popover')
    expect(close).not.toHaveBeenCalled()
    press('p')
    expect(close).toHaveBeenCalledOnce()
    wrapper.unmount()
  })

  it('leaves Esc to the nav scope', () => {
    const { close, wrapper } = popover()
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Escape' }))
    expect(close).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('stops listening once unmounted', () => {
    const { close, wrapper, press } = popover()
    wrapper.unmount()
    press('p')
    expect(close).not.toHaveBeenCalled()
  })
})
