import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick, ref } from 'vue'
import { mount, type VueWrapper } from '@vue/test-utils'
import { useQuickMenuNavigation } from './useQuickMenuNavigation'
import { FocusMark, QuickMenuView } from './view'

// A stand-in menu: the main screen's opener buttons, and per page a marked control or only the
// back button, as the real parts render them.
function harness() {
  const resume = vi.fn()
  const home = vi.fn()
  let nav!: ReturnType<typeof useQuickMenuNavigation>
  const Menu = defineComponent({
    setup() {
      const root = ref<HTMLElement | null>(null)
      nav = useQuickMenuNavigation(root, { resume, home })
      return () =>
        h('section', { ref: root }, [
          nav.view.value === QuickMenuView.Home
            ? [
                h('button', {
                  id: 'controls',
                  onClick: (e: Event) => nav.open(QuickMenuView.Controls, e),
                }),
                h('button', {
                  id: 'slots',
                  onClick: (e: Event) => nav.open(QuickMenuView.Slots, e),
                }),
                h('button', { id: 'chosen', [FocusMark.ChosenSlot]: '' }),
              ]
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
