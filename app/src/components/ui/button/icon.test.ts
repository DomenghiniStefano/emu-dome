// Button's own icon (Trello #104): its size follows the button's, icon + words by default, and with
// `label` the icon alone, named by the label (aria-label and tooltip), a caller's `title` winning.
import { Play } from '@lucide/vue'
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { h } from 'vue'
import { Button, ButtonSize, buttonIconPx } from '@/components/ui/button'

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
    expect(w.attributes('title')).toBe('Riprendi')
    expect(w.find('svg').attributes('width')).toBe('14')
  })

  it("keeps a caller's own title", () => {
    const w = mount({
      render: () => h(Button, { icon: Play, label: 'Riprendi', title: 'Disponibile più avanti' }),
    })
    expect(w.find('button').attributes('title')).toBe('Disponibile più avanti')
  })
})
