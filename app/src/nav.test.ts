// nav.ts beyond the primitives (components/ui/nav.test.ts): the first focus of a screen, Esc as
// `back`, the scope tree (who receives the steps, `back` / `tab` bubbling) and which elements a
// step can land on. happy-dom computes no layout: every element gets a rect (10x10 at the origin
// unless placed), so it counts as shown.
import { mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, ref, type Ref, type VNode } from 'vue'
import {
  dispatchNav,
  focusInitial,
  useEscapeBack,
  useNavGrid,
  useNavScope,
  useNavSection,
  type GridStep,
} from './nav'
import type { NavAction } from './emu'

const rects = new WeakMap<Element, DOMRect>()
const place = (el: Element, x: number, y: number) => rects.set(el, new DOMRect(x, y, 10, 10))

beforeAll(() => {
  HTMLElement.prototype.getBoundingClientRect = function () {
    return rects.get(this) ?? new DOMRect(0, 0, 10, 10)
  }
  HTMLElement.prototype.getClientRects = function () {
    return [this.getBoundingClientRect()] as unknown as DOMRectList
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

/** A component registering `register(root)` around `children`, rendered as a `<div>`. */
function scoped(
  register: (root: Ref<HTMLElement | null>) => void,
  children: () => VNode[] = () => [],
  attrs: Record<string, unknown> = {},
) {
  return defineComponent({
    setup() {
      const root = ref<HTMLElement | null>(null)
      register(root)
      return () => h('div', { ref: root, ...attrs }, children())
    },
  })
}

describe('the scope tree', () => {
  it('a child scope mounted in the same tick as its parent receives the steps', () => {
    const parentBack = vi.fn()
    const childBack = vi.fn()
    const Child = scoped(root => useNavScope(root, { back: childBack }))
    // Vue runs the child's onMounted before the parent's: a flat stack let the parent win.
    part(
      root => useNavScope(root, { back: parentBack }),
      () => [h(Child)],
    )
    dispatchNav('back')
    expect(childBack).toHaveBeenCalledOnce()
    expect(parentBack).not.toHaveBeenCalled()
  })

  it('back and tab bubble from a section to its screen when the section leaves them', () => {
    const screenBack = vi.fn()
    const screenTab = vi.fn()
    let deep = true
    const sectionBack = vi.fn(() => (deep ? undefined : false))
    const Section = scoped(root => useNavSection(root, { back: sectionBack }))
    part(
      root => useNavScope(root, { back: screenBack, tab: screenTab }),
      () => [h(Section)],
    )
    // One step down: the section goes back up itself.
    dispatchNav('back')
    expect(sectionBack).toHaveBeenCalledOnce()
    expect(screenBack).not.toHaveBeenCalled()
    // At its top it returns false: the screen takes the step.
    deep = false
    dispatchNav('back')
    expect(screenBack).toHaveBeenCalledOnce()
    // No `tab` of its own: L1 / R1 reach the screen.
    dispatchNav('next')
    expect(screenTab).toHaveBeenCalledWith(1)
  })

  it('Y (search) bubbles from a section to the screen, and a dialog above stops it', () => {
    const search = vi.fn()
    const Section = scoped(root => useNavSection(root, {}))
    const screen = part(
      root => useNavScope(root, { search }),
      () => [h(Section)],
    )
    dispatchNav('search')
    expect(search).toHaveBeenCalledOnce()
    screen.unmount()
    mounted.splice(mounted.indexOf(screen), 1)
    const behind = vi.fn()
    const Dialog = scoped(root => useNavScope(root, {}))
    part(
      root => useNavScope(root, { search: behind }),
      () => [h(Dialog)],
    )
    dispatchNav('search')
    expect(behind).not.toHaveBeenCalled()
  })

  it('a scope that owns its steps sees each one first, and leaves the rest to nav.ts', () => {
    const back = vi.fn()
    // Takes Select and the left arrow, nothing else.
    const own = vi.fn((a: NavAction) => (a === 'select' || a === 'left' ? undefined : false))
    part(
      root => useNavScope(root, { back, own }),
      () => [h('button', { id: 'a' })],
    )
    dispatchNav('select')
    dispatchNav('left')
    expect(own).toHaveBeenCalledWith('select')
    expect(back).not.toHaveBeenCalled()
    // Not taken: nav.ts goes on (B reaches `back`).
    dispatchNav('back')
    expect(own).toHaveBeenLastCalledWith('back')
    expect(back).toHaveBeenCalledOnce()
  })

  it('a dialog above a section ends the bubbling: the screen behind it gets nothing', () => {
    const screenBack = vi.fn()
    const sectionBack = vi.fn(() => false)
    const Dialog = scoped(root => useNavScope(root, { back: () => false }))
    const Section = scoped(
      root => useNavSection(root, { back: sectionBack }),
      () => [h(Dialog)],
    )
    part(
      root => useNavScope(root, { back: screenBack }),
      () => [h(Section)],
    )
    dispatchNav('back')
    expect(sectionBack).not.toHaveBeenCalled()
    expect(screenBack).not.toHaveBeenCalled()
  })

  it('a section does not hold the focus: a step leaves it for the rest of the screen', async () => {
    const Section = scoped(
      root => useNavSection(root, {}),
      () => [h('button', { id: 'inside' })],
    )
    part(
      root => useNavScope(root, {}),
      () => [h('button', { id: 'rail' }), h(Section)],
    )
    await nextTick()
    const rail = document.getElementById('rail')
    const inside = document.getElementById('inside')
    if (!rail || !inside) throw new Error('not rendered')
    place(rail, 0, 0)
    place(inside, 50, 0)
    inside.focus()
    dispatchNav('left')
    expect(document.activeElement).toBe(rail)
  })
})

describe('where a step lands', () => {
  /** A screen with `#from` at the origin and the other buttons placed right of it, in order. */
  async function row(buttons: Record<string, unknown>[]) {
    part(
      root => useNavScope(root, {}),
      () => [h('button', { id: 'from' }), ...buttons.map(attrs => h('button', attrs))],
    )
    await nextTick()
    const from = document.getElementById('from')
    if (!from) throw new Error('not rendered')
    place(from, 0, 0)
    buttons.forEach((b, i) => {
      const el = document.getElementById(String(b.id))
      if (el) place(el, 20 + i * 20, 0)
    })
    from.focus()
  }

  it('skips aria-disabled, inert and visibility:hidden elements', async () => {
    await row([
      { id: 'aria', 'aria-disabled': 'true' },
      { id: 'hidden', style: 'visibility: hidden' },
      { id: 'target' },
    ])
    dispatchNav('right')
    expect(document.activeElement?.id).toBe('target')

    document.getElementById('target')?.setAttribute('inert', '')
    document.getElementById('from')?.focus()
    dispatchNav('right')
    expect(document.activeElement?.id).toBe('from')
  })

  it('looks in the scroll container first, then in the whole screen', async () => {
    const List = defineComponent({
      setup() {
        const list = ref<HTMLElement | null>(null)
        return () =>
          h('ul', { ref: list, id: 'list', style: 'overflow-y: auto' }, [
            h('button', { id: 'a' }),
            h('button', { id: 'b' }),
          ])
      },
    })
    part(
      root => useNavScope(root, {}),
      () => [h(List), h('button', { id: 'out' })],
    )
    await nextTick()
    const [list, a, b, out] = ['list', 'a', 'b', 'out'].map(id => document.getElementById(id))
    if (!list || !a || !b || !out) throw new Error('not rendered')
    Object.defineProperty(list, 'scrollHeight', { value: 500 })
    Object.defineProperty(list, 'clientHeight', { value: 100 })
    place(a, 0, 0)
    // In the list, far down; out of it, just below but off to the side (it would win on score).
    place(b, 0, 200)
    place(out, 30, 20)
    const outRect = vi.spyOn(out, 'getBoundingClientRect')
    a.focus()
    dispatchNav('down')
    expect(document.activeElement).toBe(b)
    // The rest of the screen was not measured.
    expect(outRect).not.toHaveBeenCalled()
    // Nothing further down in the list: the whole screen.
    dispatchNav('right')
    expect(document.activeElement).toBe(out)
  })
})

describe('a grid that moves by itself (useNavGrid)', () => {
  /**
   * A screen with a grid of `a` and, once `far` is rendered, `far`; `out` outside the grid, below
   * it. The grid's step: down from `a` renders `far` and lands there, up leaves the grid.
   */
  async function screen() {
    const far = ref(false)
    const calls: string[] = []
    const step: GridStep = (from, dir) => {
      calls.push(`${from.id} ${dir}`)
      if (dir !== 'down') return false
      far.value = true
      return nextTick().then(() => document.getElementById('far'))
    }
    const Grid = defineComponent({
      setup() {
        const grid = ref<HTMLElement | null>(null)
        useNavGrid(grid, step)
        return () =>
          h('div', { ref: grid, id: 'grid' }, [
            h('button', { id: 'a' }),
            far.value ? h('button', { id: 'far' }) : null,
          ])
      },
    })
    part(
      root => useNavScope(root, {}),
      () => [h('button', { id: 'out' }), h(Grid)],
    )
    await nextTick()
    const [a, out] = ['a', 'out'].map(id => document.getElementById(id))
    if (!a || !out) throw new Error('not rendered')
    place(out, 0, 0)
    place(a, 0, 100)
    return { a, out, calls }
  }

  it('a tile out of the DOM becomes focusable when a step reaches it, nothing measured', async () => {
    const { a, out, calls } = await screen()
    const outRect = vi.spyOn(out, 'getBoundingClientRect')
    a.focus()
    expect(document.getElementById('far')).toBeNull()
    dispatchNav('down')
    await nextTick()
    await nextTick()
    expect(document.activeElement?.id).toBe('far')
    expect(calls).toEqual(['a down'])
    expect(outRect).not.toHaveBeenCalled()
  })

  it('a step the grid leaves moves spatially out of it', async () => {
    const { a } = await screen()
    a.focus()
    dispatchNav('up')
    expect(document.activeElement?.id).toBe('out')
  })

  it('an element outside the grid is never asked to the grid', async () => {
    const { out, calls } = await screen()
    out.focus()
    dispatchNav('down')
    expect(calls).toEqual([])
    expect(document.activeElement?.id).toBe('a')
  })
})
