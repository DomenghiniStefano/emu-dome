import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, ref } from 'vue'
import { mount, type VueWrapper } from '@vue/test-utils'
import { useQuickMenuNavigation } from './useQuickMenuNavigation'
import { FocusMark, QuickMenuView, openerMark, phoneParent } from './view'

// The main screen as its own component, unmounted while a page shows (as QuickMenuHome is), so
// a control remembered as an element is detached by the time back runs.
const Home = defineComponent({
  emits: ['open'],
  setup(_, { emit }) {
    return () => [
      h('button', {
        id: 'controls',
        ...openerMark(QuickMenuView.Controls),
        onClick: (e: Event) => emit('open', QuickMenuView.Controls, e),
      }),
      h('button', {
        id: 'slots',
        ...openerMark(QuickMenuView.Slots),
        onClick: (e: Event) => emit('open', QuickMenuView.Slots, e),
      }),
      // An opener without the mark: nothing to find again, so back lands on the chosen slot.
      h('button', {
        id: 'unmarked',
        onClick: (e: Event) => emit('open', QuickMenuView.Speed, e),
      }),
      h('button', { id: 'chosen', [FocusMark.ChosenSlot]: '' }),
      // The phone's «Altro».
      h('button', {
        id: 'more',
        ...openerMark(QuickMenuView.More),
        onClick: (e: Event) => emit('open', QuickMenuView.More, e),
      }),
    ]
  },
})

// A stand-in menu: the main screen, and per page a marked control or only the back button, as
// the real parts render them.
function harness(parent?: (v: QuickMenuView) => QuickMenuView) {
  const resume = vi.fn()
  const home = vi.fn()
  let nav!: ReturnType<typeof useQuickMenuNavigation>
  const Menu = defineComponent({
    setup() {
      const root = ref<HTMLElement | null>(null)
      nav = useQuickMenuNavigation(root, { resume, home, parent })
      return () =>
        h('section', { ref: root }, [
          nav.view.value === QuickMenuView.Home
            ? h(Home, { onOpen: (v: QuickMenuView, e: Event) => nav.open(v, e) })
            : [
                h('button', { id: 'back', [FocusMark.Back]: '' }),
                nav.view.value === QuickMenuView.Controls
                  ? h('button', {
                      id: 'shortcuts',
                      [FocusMark.Shortcuts]: '',
                      [FocusMark.SubInitial]: '',
                    })
                  : null,
                nav.view.value === QuickMenuView.Shortcuts
                  ? h('div', { [FocusMark.SubInitial]: '' }, [h('button', { id: 'first-combo' })])
                  : null,
                nav.view.value === QuickMenuView.More
                  ? [
                      h('button', { id: 'first-row', [FocusMark.SubInitial]: '' }),
                      h('button', {
                        id: 'saves-row',
                        ...openerMark('saves'),
                        onClick: (e: Event) => nav.open(QuickMenuView.Slots, e),
                      }),
                      h('button', {
                        id: 'layout-row',
                        ...openerMark('layout'),
                        onClick: (e: Event) => nav.open(QuickMenuView.TouchEditor, e),
                      }),
                    ]
                  : null,
                nav.view.value === QuickMenuView.Screens
                  ? h('button', {
                      id: 'customise',
                      [FocusMark.Customise]: '',
                      onClick: (e: Event) => nav.open(QuickMenuView.ScreenEditor, e),
                    })
                  : null,
              ],
        ])
    },
  })
  const wrapper = mount(Menu, { attachTo: document.body })
  return { wrapper, nav: () => nav, resume, home }
}

const focused = () => document.activeElement?.id

let mounted: VueWrapper | null = null
afterEach(() => {
  mounted?.unmount()
  mounted = null
})

describe('quick menu navigation', () => {
  it('opens a page on its marked control, else on its back button', async () => {
    const { wrapper } = harness()
    mounted = wrapper
    await wrapper.find('#controls').trigger('click')
    await nextTick()
    expect(focused()).toBe('shortcuts')
  })

  it('falls back to the back button on a page with no mark', async () => {
    const { wrapper } = harness()
    mounted = wrapper
    await wrapper.find('#slots').trigger('click')
    await nextTick()
    expect(focused()).toBe('back')
  })

  it('goes back one level and gives the focus to what opened the page', async () => {
    const { wrapper, nav, home } = harness()
    mounted = wrapper
    await wrapper.find('#controls').trigger('click')
    await nextTick()
    nav().open(QuickMenuView.Shortcuts, new Event('click'))
    await nextTick()
    expect(focused()).toBe('first-combo')

    nav().back()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.Controls)
    expect(focused()).toBe('shortcuts')

    nav().back()
    await nextTick()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.Home)
    expect(home).toHaveBeenCalledOnce()
    expect(focused()).toBe('controls')
  })

  it('finds the opener again on the remounted main screen, not the detached element', async () => {
    const { wrapper, nav } = harness()
    mounted = wrapper
    const before = wrapper.find('#slots').element
    await wrapper.find('#slots').trigger('click')
    await nextTick()
    expect(before.isConnected).toBe(false)

    nav().back()
    await nextTick()
    await nextTick()
    const after = wrapper.find('#slots').element
    expect(after).not.toBe(before)
    expect(document.activeElement).toBe(after)
  })

  it('lands on the chosen slot when the opener has no mark', async () => {
    const { wrapper, nav } = harness()
    mounted = wrapper
    await wrapper.find('#unmarked').trigger('click')
    await nextTick()
    nav().back()
    await nextTick()
    await nextTick()
    expect(focused()).toBe('chosen')
  })

  it('goes back from a page the search opened onto the chosen slot', async () => {
    const { wrapper, nav } = harness()
    mounted = wrapper
    // A page opened from the main screen first: the search must not bring the focus back to it.
    await wrapper.find('#controls').trigger('click')
    await nextTick()
    nav().back()
    await nextTick()
    await nextTick()
    nav().open(QuickMenuView.Speed, null)
    await nextTick()
    nav().back()
    await nextTick()
    await nextTick()
    expect(focused()).toBe('chosen')
  })

  it('resumes the game from the main screen', () => {
    const { wrapper, nav, resume } = harness()
    mounted = wrapper
    nav().back()
    expect(resume).toHaveBeenCalledOnce()
  })

  it('focuses the chosen slot when the menu opens', async () => {
    const { wrapper, nav } = harness()
    mounted = wrapper
    await nav().focusChosenSlot()
    expect(focused()).toBe('chosen')
  })

  it('closes the screens editor back onto G09, on «Personalizza gli schermi»', async () => {
    const { wrapper, nav, home } = harness()
    mounted = wrapper
    nav().open(QuickMenuView.Screens, new Event('click'))
    await nextTick()
    await wrapper.find('#customise').trigger('click')
    expect(nav().view.value).toBe(QuickMenuView.ScreenEditor)
    nav().back()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.Screens)
    expect(focused()).toBe('customise')
    expect(home).not.toHaveBeenCalled()
  })
})

describe('quick menu navigation on the phone (two levels)', () => {
  it('goes back from a page to «Altro», on the row that opened it, then to the main screen', async () => {
    const { wrapper, nav, home, resume } = harness(phoneParent)
    mounted = wrapper
    await wrapper.find('#more').trigger('click')
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.More)
    expect(focused()).toBe('first-row')

    await wrapper.find('#saves-row').trigger('click')
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.Slots)

    nav().back()
    await nextTick()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.More)
    expect(focused()).toBe('saves-row')
    expect(home).not.toHaveBeenCalled()

    nav().back()
    await nextTick()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.Home)
    expect(home).toHaveBeenCalledOnce()
    expect(focused()).toBe('more')

    nav().back()
    expect(resume).toHaveBeenCalledOnce()
  })

  it('closes the touch editor opened from «Altro» back onto its row (Trello #111)', async () => {
    const { wrapper, nav, home } = harness(phoneParent)
    mounted = wrapper
    await wrapper.find('#more').trigger('click')
    await nextTick()
    await wrapper.find('#layout-row').trigger('click')
    expect(nav().view.value).toBe(QuickMenuView.TouchEditor)
    nav().back()
    await nextTick()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.More)
    expect(focused()).toBe('layout-row')
    expect(home).not.toHaveBeenCalled()
  })

  it('goes back from a page the search opened to «Altro»', async () => {
    const { wrapper, nav } = harness(phoneParent)
    mounted = wrapper
    nav().open(QuickMenuView.Speed, null)
    await nextTick()
    nav().back()
    await nextTick()
    await nextTick()
    expect(nav().view.value).toBe(QuickMenuView.More)
    expect(focused()).toBe('first-row')
  })
})
