// Button's own icon (Trello #104): its size follows the button's, icon + words by default, and with
// `label` the icon alone, named by the label (aria-label and the app's tooltip, ui/tooltip), a
// caller's `title` winning.
import { Play } from '@lucide/vue'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { h } from 'vue'
import { Button, ButtonSize, buttonIconClass, buttonIconPx } from '@/components/ui/button'
import { TOOLTIP_DELAY_MS } from '@/components/ui/tooltip'

describe('buttonIconPx', () => {
  it('sizes the icon by the button', () => {
    expect(buttonIconPx(undefined)).toBe(16)
    expect(buttonIconPx(ButtonSize.Medium)).toBe(16)
    expect(buttonIconPx(ButtonSize.Small)).toBe(14)
    expect(buttonIconPx(ButtonSize.IconRow)).toBe(14)
    expect(buttonIconPx(ButtonSize.Large)).toBe(18)
    expect(buttonIconPx(ButtonSize.ToolIcon)).toBe(18)
    expect(buttonIconPx(ButtonSize.Tv)).toBe(24)
  })
})

describe('Button icon', () => {
  it('draws the icon before the words, hidden from screen readers', () => {
    const w = mount(Button, { props: { icon: Play }, slots: { default: () => 'Riprendi' } })
    const svg = w.find('svg')
    expect(svg.exists()).toBe(true)
    expect(svg.attributes('aria-hidden')).toBe('true')
    expect(svg.attributes('width')).toBe('16')
    expect(svg.attributes('stroke-width')).toBe('1.75')
    expect(w.text()).toBe('Riprendi')
    expect(w.attributes('aria-label')).toBeUndefined()
    expect(w.attributes('title')).toBeUndefined()
  })

  it('draws a filled icon as a solid shape', () => {
    const w = mount(Button, { props: { icon: Play, iconFilled: true } })
    expect(w.find('svg').attributes('fill')).toBe('currentColor')
    expect(w.find('svg').attributes('stroke-width')).toBe('0')
  })

  it('with a label, draws the icon alone and names the button with it', () => {
    const w = mount(Button, {
      props: { icon: Play, label: 'Riprendi', size: ButtonSize.IconRow },
      slots: { default: () => 'never shown' },
    })
    expect(w.text()).toBe('')
    expect(w.attributes('aria-label')).toBe('Riprendi')
    // The name shows in the app's tooltip, never also as the browser's own.
    expect(w.attributes('title')).toBeUndefined()
    expect(w.find('svg').attributes('width')).toBe('14')
  })

  it('with a label and no icon, names a button that draws its own picture', () => {
    const w = mount(Button, {
      props: { label: 'Altro' },
      slots: { default: () => h('svg', { 'data-own': '' }) },
    })
    expect(w.find('svg[data-own]').exists()).toBe(true)
    expect(w.attributes('aria-label')).toBe('Altro')
  })

  it('sizes the icon one step larger on the phone', () => {
    expect(buttonIconClass(ButtonSize.IconRow)).toBe('size-14 touch:size-16')
    expect(buttonIconClass(undefined)).toBe('size-16 touch:size-18')
    expect(buttonIconClass(ButtonSize.Large)).toBe('size-18 touch:size-20')
    expect(buttonIconClass(ButtonSize.Tv)).toBe('size-24')
    const w = mount(Button, { props: { icon: Play }, slots: { default: () => 'Riprendi' } })
    expect(w.find('svg').classes()).toEqual(expect.arrayContaining(['size-16', 'touch:size-18']))
  })
})

describe('Button tooltip', () => {
  afterEach(() => {
    delete document.documentElement.dataset.nav
  })

  // Reka repeats the words in a visually hidden `role="tooltip"` for screen readers: read that one.
  const tooltipText = () => document.querySelector('[role="tooltip"]')?.textContent

  it('shows the label on the controller’s focus, and hides it on blur', async () => {
    document.documentElement.dataset.nav = ''
    const w = mount(Button, {
      props: { icon: Play, label: 'Riprendi', size: ButtonSize.IconRow },
      attachTo: document.body,
    })
    await w.trigger('focus')
    await flushPromises()
    expect(tooltipText()?.trim()).toBe('Riprendi')
    await w.trigger('blur')
    await flushPromises()
    expect(tooltipText()).toBeUndefined()
    w.unmount()
  })

  it('shows a caller’s title in place of the label', async () => {
    document.documentElement.dataset.nav = ''
    const w = mount(
      { render: () => h(Button, { icon: Play, label: 'Riprendi', title: 'Più avanti' }) },
      { attachTo: document.body },
    )
    expect(w.find('button').attributes('title')).toBeUndefined()
    await w.find('button').trigger('focus')
    await flushPromises()
    expect(tooltipText()?.trim()).toBe('Più avanti')
    w.unmount()
  })

  it('does not show on a focus that follows a press, nor on a mouse focus', async () => {
    const w = mount(Button, {
      props: { icon: Play, label: 'Riprendi' },
      attachTo: document.body,
    })
    await w.trigger('focus')
    await flushPromises()
    expect(tooltipText()).toBeUndefined()
    document.documentElement.dataset.nav = ''
    await w.trigger('pointerdown')
    await w.trigger('focus')
    await flushPromises()
    expect(tooltipText()).toBeUndefined()
    w.unmount()
  })

  it('shows after the mouse rests on it, and Esc hides it', async () => {
    vi.useFakeTimers()
    try {
      const w = mount(Button, {
        props: { icon: Play, label: 'Riprendi' },
        attachTo: document.body,
      })
      w.element.dispatchEvent(new PointerEvent('pointerenter', { pointerType: 'mouse' }))
      expect(tooltipText()).toBeUndefined()
      vi.advanceTimersByTime(TOOLTIP_DELAY_MS)
      await flushPromises()
      expect(tooltipText()?.trim()).toBe('Riprendi')
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
      await flushPromises()
      expect(tooltipText()).toBeUndefined()
      w.unmount()
    } finally {
      vi.useRealTimers()
    }
  })

  it('never shows for a button with words', async () => {
    document.documentElement.dataset.nav = ''
    const w = mount(Button, {
      props: { icon: Play },
      slots: { default: () => 'Riprendi' },
      attachTo: document.body,
    })
    await w.trigger('focus')
    await flushPromises()
    expect(tooltipText()).toBeUndefined()
    w.unmount()
  })
})
