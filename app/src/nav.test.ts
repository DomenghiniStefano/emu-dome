// nav.ts beyond the primitives (components/ui/nav.test.ts): the first focus of a screen, and Esc
// as `back`. happy-dom computes no layout: every element gets a rect, so it counts as shown.
import { mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref, type Ref, type VNode } from 'vue'
import { focusInitial, useEscapeBack, useNavScope } from './nav'

beforeAll(() => {
  HTMLElement.prototype.getClientRects = function () {
    return [new DOMRect(0, 0, 10, 10)] as unknown as DOMRectList
  }
  HTMLElement.prototype.scrollIntoView = () => {}
})

const mounted: VueWrapper[] = []
afterEach(() => {
  mounted.splice(0).forEach(w => w.unmount())
  document.body.innerHTML = ''
})

const esc = () => {
  const e = new KeyboardEvent('keydown', { code: 'Escape', cancelable: true })
  window.dispatchEvent(e)
  return e
}

/** A part registered the way a screen does it; `children` render inside its root. */
function part(setup: (root: Ref<HTMLElement | null>) => void, children: () => VNode[] = () => []) {
  const Part = defineComponent({
    setup() {
      const root = ref<HTMLElement | null>(null)
      setup(root)
      return () => h('section', { ref: root }, children())
    },
  })
  const w = mount(Part, { attachTo: document.body })
  mounted.push(w)
  return w
}

describe('focusInitial', () => {
  const root = (html: string) => {
    const el = document.createElement('div')
    el.innerHTML = html
    document.body.append(el)
    return el
  }

  it('focuses the marked element, else the first focusable', () => {
    const r = root('<button id="a"></button><button id="b" data-nav-initial></button>')
    focusInitial(r)
    expect(document.activeElement?.id).toBe('b')
    const plain = root('<button id="c"></button><button id="d"></button>')
    focusInitial(plain)
    expect(document.activeElement?.id).toBe('c')
  })

  it('skips a disabled mark for the first focusable', () => {
    const r = root('<button id="a"></button><button id="b" data-nav-initial disabled></button>')
    focusInitial(r)
    expect(document.activeElement?.id).toBe('a')
  })

  it('unlessMoved: takes the focus from nowhere or from the rail, never from elsewhere', () => {
    const rail = root('<button id="rail" aria-current="page"></button><button id="other"></button>')
    const r = root('<button id="first" data-nav-initial></button>')
    focusInitial(r, { unlessMoved: true })
    expect(document.activeElement?.id).toBe('first')

    rail.querySelector<HTMLElement>('#rail')?.focus()
    focusInitial(r, { unlessMoved: true })
    expect(document.activeElement?.id).toBe('first')

    rail.querySelector<HTMLElement>('#other')?.focus()
    focusInitial(r, { unlessMoved: true })
    expect(document.activeElement?.id).toBe('other')
  })
})

describe('Esc', () => {
  it('is the back of a scope that asks for it, and of no other', () => {
    const back = vi.fn()
    part(root => useNavScope(root, { back }))
    esc()
    expect(back).not.toHaveBeenCalled()

    const escBack = vi.fn()
    part(root => useNavScope(root, { back: escBack, escape: true }))
    const e = esc()
    expect(escBack).toHaveBeenCalledOnce()
    expect(e.defaultPrevented).toBe(true)
  })

  it('goes to the part inside the scope that receives the steps (B = Esc)', () => {
    const screenBack = vi.fn()
    const detailsBack = vi.fn()
    // A screen with a part that goes back on Esc by itself (the game details in the shell).
    const Details = defineComponent({
      setup() {
        const root = ref<HTMLElement | null>(null)
        useEscapeBack(root, detailsBack)
        return () => h('div', { ref: root })
      },
    })
    part(
      root => useNavScope(root, { back: screenBack }),
      () => [h(Details)],
    )
    esc()
    expect(detailsBack).toHaveBeenCalledOnce()
    expect(screenBack).not.toHaveBeenCalled()

    // A popover opened above it takes the steps: Esc closes the popover only.
    const close = vi.fn()
    const popover = part(root => useNavScope(root, { back: close, escape: true }))
    esc()
    expect(close).toHaveBeenCalledOnce()
    expect(detailsBack).toHaveBeenCalledOnce()

    // Closed again: Esc is the details' once more.
    popover.unmount()
    mounted.splice(mounted.indexOf(popover), 1)
    esc()
    expect(detailsBack).toHaveBeenCalledTimes(2)
  })

  it('stops with the part', () => {
    const back = vi.fn()
    const w = part(root => useNavScope(root, { back, escape: true }))
    w.unmount()
    mounted.splice(mounted.indexOf(w), 1)
    esc()
    expect(back).not.toHaveBeenCalled()
  })
})
