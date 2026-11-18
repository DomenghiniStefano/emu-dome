// Every primitive with the app's navigation (nav.ts), one describe each: the controller's steps
// (`dispatchNav`, what `emu://nav` calls) and the keyboard (a window listener doing what App.vue's
// does: arrows and Enter become the same steps). Layout is stubbed: happy-dom computes none, and
// nav.ts moves by bounding rects, so each test places the elements it moves between.
//
// happy-dom does not run Reka's roving focus (a tab list's own arrow moves), so for Tabs and
// Segmented these tests check nav.ts's side of the rule: the key handed to the element, nav.ts not
// moving, the ends left to nav.ts. The moves themselves are checked in a real browser by
// scripts/headless/kit-nav.mjs.
import { mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, ref } from 'vue'
import { dispatchNav, keyToNav, ownsArrow, ownsEnter, useNavScope } from '@/nav'
import { Button } from './button'
import { Card, CardVariant } from './card'
import { Chip } from './chip'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from './dialog'
import { Segmented, SegmentedItem } from './segmented'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './select'
import { Slider } from './slider'
import { Switch } from './switch'
import { Tabs, TabsList, TabsTrigger } from './tabs'

// --- layout stub: a rect per element, 10x10 at the origin unless placed

const rects = new WeakMap<Element, DOMRect>()
const place = (el: Element, x: number, y: number, w = 10, h = 10) =>
  rects.set(el, new DOMRect(x, y, w, h))
const row = (els: Element[], y = 0) => els.forEach((el, i) => place(el, i * 20, y))

beforeAll(() => {
  HTMLElement.prototype.getBoundingClientRect = function () {
    return rects.get(this) ?? new DOMRect(0, 0, 10, 10)
  }
  HTMLElement.prototype.getClientRects = function () {
    return [this.getBoundingClientRect()] as unknown as DOMRectList
  }
  HTMLElement.prototype.scrollIntoView = () => {}
  HTMLElement.prototype.hasPointerCapture = () => false
  HTMLElement.prototype.releasePointerCapture = () => {}
})

// --- App.vue's keyboard path, outside a game: arrows and Enter become nav steps

const appKeys = (e: KeyboardEvent) => {
  const nav = keyToNav(e)
  if (!nav) return
  e.preventDefault()
  dispatchNav(nav)
}
beforeEach(() => window.addEventListener('keydown', appKeys))

let wrapper: VueWrapper | null = null
afterEach(() => {
  window.removeEventListener('keydown', appKeys)
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
})

/** Mounts `render` inside a screen: a root registered with useNavScope, as every screen does. */
function screen(render: () => ReturnType<typeof h>, back?: () => void) {
  const Screen = defineComponent({
    setup() {
      const root = ref<HTMLElement | null>(null)
      useNavScope(root, { back })
      return () => h('main', { ref: root }, [render()])
    },
  })
  wrapper = mount(Screen, { attachTo: document.body })
  return wrapper
}

const key = (el: Element, code: string) =>
  el.dispatchEvent(new KeyboardEvent('keydown', { code, key: code, bubbles: true }))
const all = (selector: string) => [...document.querySelectorAll<HTMLElement>(selector)]
const one = (selector: string) => {
  const el = document.querySelector<HTMLElement>(selector)
  if (!el) throw new Error(`nothing matches ${selector}`)
  return el
}

describe('Button', () => {
  it('is clicked once by confirm and once by Enter', async () => {
    const click = vi.fn()
    screen(() => h(Button, { onClick: click }, () => 'Go'))
    const button = one('[data-slot="button"]')
    button.focus()
    dispatchNav('confirm')
    expect(click).toHaveBeenCalledTimes(1)
    key(button, 'Enter')
    expect(click).toHaveBeenCalledTimes(2)
  })
})

describe('Card as a button', () => {
  it('is reached and clicked like a button', () => {
    const click = vi.fn()
    screen(() =>
      h('div', [
        h(Button, () => 'Before'),
        h(Card, { as: 'button', variant: CardVariant.Raised, onClick: click }, () => 'Card'),
      ]),
    )
    row([one('[data-slot="button"]'), one('[data-slot="card"]')])
    one('[data-slot="button"]').focus()
    dispatchNav('right')
    expect(document.activeElement).toBe(one('[data-slot="card"]'))
    dispatchNav('confirm')
    expect(click).toHaveBeenCalledTimes(1)
  })
})

describe('Chip', () => {
  it('moves along a row and toggles once per confirm', () => {
    const picked = vi.fn()
    screen(() =>
      h('div', [
        h(Chip, { onClick: () => picked('a') }, () => 'A'),
        h(Chip, { onClick: () => picked('b') }, () => 'B'),
      ]),
    )
    const chips = all('[data-slot="chip"]')
    row(chips)
    chips[0]?.focus()
    key(chips[0] as HTMLElement, 'ArrowRight')
    expect(document.activeElement).toBe(chips[1])
    dispatchNav('confirm')
    expect(picked.mock.calls).toEqual([['b']])
  })
})

/** Every keydown reaching `el`: what nav.ts hands a primitive. */
function keysOn(el: Element) {
  const seen: string[] = []
  el.addEventListener('keydown', e => seen.push((e as KeyboardEvent).key))
  return seen
}
const settle = async () => {
  for (let i = 0; i < 6; i++) await nextTick()
}

describe('the rule: only Reka elements own keys', () => {
  it('leaves the screens of today alone (their own tabs, switches and options)', () => {
    document.body.innerHTML = [
      '<div role="tablist"><button role="tab" id="t">a</button></div>',
      '<button role="switch" id="s"></button>',
      '<div role="listbox"><button role="option" id="o">x</button></div>',
    ].join('')
    for (const id of ['t', 's', 'o']) {
      const el = document.getElementById(id) as HTMLElement
      expect(ownsEnter(el)).toBe(false)
      expect(ownsArrow(el, 'right')).toBe(false)
      expect(ownsArrow(el, 'down')).toBe(false)
    }
  })
})

describe('Switch (Reka)', () => {
  // Every change the switch asks for, not only where it ends: Reka reads its controlled value
  // from the prop, which updates on the next tick, so two toggles in one press both ask for
  // `true` and the end state alone would hide the second.
  const mountSwitch = () => {
    const on = ref(false)
    const asked: boolean[] = []
    screen(() =>
      h(Switch, {
        modelValue: on.value,
        'onUpdate:modelValue': (v: boolean) => {
          asked.push(v)
          on.value = v
        },
        'aria-label': 'Switch',
      }),
    )
    return { on, asked }
  }

  it('toggles once on A: nav.ts sends it the Enter it acts on', async () => {
    const { asked } = mountSwitch()
    one('[data-slot="switch"]').focus()
    dispatchNav('confirm')
    await nextTick()
    expect(asked).toEqual([true])
  })

  it('toggles once per Enter from the keyboard', async () => {
    const { asked } = mountSwitch()
    const el = one('[data-slot="switch"]')
    el.focus()
    key(el, 'Enter')
    await nextTick()
    key(el, 'Enter')
    await nextTick()
    expect(asked).toEqual([true, false])
  })
})

describe('Tabs (Reka)', () => {
  const mountTabs = () => {
    const tab = ref('one')
    screen(() =>
      h('div', [
        h(
          Tabs,
          {
            modelValue: tab.value,
            'onUpdate:modelValue': (v: string | number) => (tab.value = String(v)),
          },
          () =>
            h(TabsList, () =>
              ['one', 'two', 'three'].map(v => h(TabsTrigger, { key: v, value: v }, () => v)),
            ),
        ),
        h(Button, { id: 'beside' }, () => 'Beside'),
      ]),
    )
    const triggers = all('[data-slot="tabs-trigger"]')
    const beside = one('#beside')
    row([...triggers, beside])
    return { tab, triggers, beside }
  }

  it('hands the arrow along the row to Reka, from the keyboard and from the controller', () => {
    const { triggers } = mountTabs()
    const first = triggers[0] as HTMLElement
    first.focus()
    const seen = keysOn(first)
    key(first, 'ArrowRight')
    dispatchNav('right')
    // The keyboard's own key, then the controller's step sent as the same key; nav.ts moved nothing.
    expect(seen).toEqual(['ArrowRight', 'ArrowRight'])
    expect(document.activeElement).toBe(first)
  })

  it('leaves the row at its end, and across it, by nav.ts', () => {
    const { triggers, beside } = mountTabs()
    const last = triggers[2] as HTMLElement
    last.focus()
    const seen = keysOn(last)
    dispatchNav('right')
    expect(seen).toEqual([])
    expect(document.activeElement).toBe(beside)
  })

  it('opens the focused tab on A (Enter), not on focus', async () => {
    const { tab, triggers } = mountTabs()
    ;(triggers[1] as HTMLElement).focus()
    await settle()
    expect(tab.value).toBe('one')
    dispatchNav('confirm')
    await settle()
    expect(tab.value).toBe('two')
  })
})

describe('Segmented (Reka ToggleGroup)', () => {
  const mountSegmented = () => {
    const speed = ref('normal')
    screen(() =>
      h('div', [
        h(
          Segmented,
          { modelValue: speed.value, 'onUpdate:modelValue': (v: string) => (speed.value = v) },
          () =>
            ['normal', 'fast', 'slow'].map(v => h(SegmentedItem, { key: v, value: v }, () => v)),
        ),
        h(Button, { id: 'below' }, () => 'Below'),
      ]),
    )
    const items = all('[data-slot="segmented-item"]')
    row(items)
    place(one('#below'), 0, 40)
    return { speed, items }
  }

  it('hands the arrow along the group to Reka, and leaves it downwards by nav.ts', () => {
    const { items } = mountSegmented()
    const first = items[0] as HTMLElement
    first.focus()
    const seen = keysOn(first)
    dispatchNav('right')
    expect(seen).toEqual(['ArrowRight'])
    expect(document.activeElement).toBe(first)
    dispatchNav('down')
    expect(document.activeElement).toBe(one('#below'))
  })

  it('chooses on A, and a second A on the chosen segment keeps it', async () => {
    const { speed, items } = mountSegmented()
    ;(items[1] as HTMLElement).focus()
    dispatchNav('confirm')
    await settle()
    expect(speed.value).toBe('fast')
    dispatchNav('confirm')
    await settle()
    expect(speed.value).toBe('fast')
  })
})

describe('Slider (Reka)', () => {
  // happy-dom does not run Reka's slider keys either: the value's step is checked in a real browser
  // (scripts/headless/kit-nav.mjs). Here: left / right go to the thumb, up / down leave it.
  it('hands left / right to the thumb, without an end; up / down leave it', () => {
    const volume = ref(100)
    screen(() =>
      h('div', [
        h(Slider, {
          modelValue: volume.value,
          'onUpdate:modelValue': (v: number) => (volume.value = v),
          label: 'Volume',
        }),
        h(Button, { id: 'beside' }, () => 'Beside'),
        h(Button, { id: 'below' }, () => 'Below'),
      ]),
    )
    const thumb = one('[role="slider"]')
    place(thumb, 0, 0)
    place(one('#beside'), 40, 0)
    place(one('#below'), 0, 40)
    thumb.focus()
    const seen = keysOn(thumb)
    dispatchNav('right')
    key(thumb, 'ArrowLeft')
    expect(seen).toEqual(['ArrowRight', 'ArrowLeft'])
    expect(document.activeElement).toBe(thumb)
    dispatchNav('down')
    expect(document.activeElement).toBe(one('#below'))
  })
})

describe('Select (Reka)', () => {
  const mountSelect = (screenBack = vi.fn()) => {
    const value = ref('b')
    screen(
      () =>
        h(
          Select,
          {
            modelValue: value.value,
            'onUpdate:modelValue': (v: unknown) => (value.value = String(v)),
          },
          () => [
            h(SelectTrigger, { 'aria-label': 'Letter' }, () => h(SelectValue)),
            h(SelectContent, () =>
              ['a', 'b', 'c'].map(v => h(SelectItem, { key: v, value: v }, () => v.toUpperCase())),
            ),
          ],
        ),
      screenBack,
    )
    return value
  }
  const isOpen = () => document.querySelector('[data-slot="select-content"]') !== null

  it('opens on A, hands every arrow to the list, picks on A', async () => {
    const value = mountSelect()
    one('[data-slot="select-trigger"]').focus()
    dispatchNav('confirm')
    await settle()
    expect(isOpen()).toBe(true)
    const options = all('[role="option"]')
    ;(options[1] as HTMLElement).focus()
    const seen = keysOn(options[1] as HTMLElement)
    dispatchNav('down')
    dispatchNav('left')
    expect(seen).toEqual(['ArrowDown', 'ArrowLeft'])
    ;(options[2] as HTMLElement).focus()
    dispatchNav('confirm')
    await settle()
    expect(value.value).toBe('c')
    expect(isOpen()).toBe(false)
  })

  it('while closed, leaves B to the screen (Reka renders its closed list out of the page)', async () => {
    const screenBack = vi.fn()
    mountSelect(screenBack)
    one('[data-slot="select-trigger"]').focus()
    dispatchNav('back')
    expect(screenBack).toHaveBeenCalledTimes(1)
  })

  it('closes on B without choosing, and B does not reach the screen', async () => {
    const screenBack = vi.fn()
    const value = mountSelect(screenBack)
    one('[data-slot="select-trigger"]').focus()
    dispatchNav('confirm')
    await settle()
    ;(all('[role="option"]')[0] as HTMLElement).focus()
    dispatchNav('back')
    await settle()
    expect(isOpen()).toBe(false)
    expect(value.value).toBe('b')
    expect(screenBack).not.toHaveBeenCalled()
  })
})

describe('Dialog (Reka)', () => {
  const mountDialog = (screenBack = vi.fn()) => {
    const isOpen = ref(false)
    screen(
      () =>
        h(
          Dialog,
          { open: isOpen.value, 'onUpdate:open': (v: boolean) => (isOpen.value = v) },
          () => [
            h(DialogTrigger, { asChild: true }, () => h(Button, () => 'Open')),
            h(DialogContent, () => [
              h(DialogTitle, () => 'Title'),
              h(DialogDescription, () => 'Text'),
              h(DialogClose, { asChild: true }, () => h(Button, () => 'Cancel')),
              h(DialogClose, { asChild: true }, () =>
                h(Button, { 'data-nav-initial': '' }, () => 'Confirm'),
              ),
            ]),
          ],
        ),
      screenBack,
    )
    return isOpen
  }

  it('opens on A on the data-nav-initial control; B closes it, not the screen; the focus goes back', async () => {
    const screenBack = vi.fn()
    const isOpen = mountDialog(screenBack)
    const trigger = one('[data-slot="dialog-trigger"]')
    trigger.focus()
    dispatchNav('confirm')
    await settle()
    expect(isOpen.value).toBe(true)
    expect(document.activeElement?.textContent).toBe('Confirm')
    dispatchNav('back')
    await settle()
    expect(isOpen.value).toBe(false)
    expect(screenBack).not.toHaveBeenCalled()
    expect(document.activeElement).toBe(trigger)
  })

  it('moves among its own controls', async () => {
    mountDialog()
    one('[data-slot="dialog-trigger"]').focus()
    dispatchNav('confirm')
    await settle()
    const [cancel, confirm] = all('[data-slot="dialog-close"]')
    row([cancel as HTMLElement, confirm as HTMLElement])
    dispatchNav('left')
    expect(document.activeElement).toBe(cancel)
  })
})
