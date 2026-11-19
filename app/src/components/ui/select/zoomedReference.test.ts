import { afterEach, describe, expect, it } from 'vitest'
import { pageZoom, zoomedReference } from './zoomedReference'

describe('zoomedReference', () => {
  afterEach(() => {
    document.documentElement.style.removeProperty('zoom')
  })

  const trigger = () => {
    const el = document.createElement('button')
    el.getBoundingClientRect = () => new DOMRect(110, 220, 220, 33)
    return el
  }

  it('keeps the trigger’s rectangle at 100%', () => {
    expect(pageZoom()).toBe(1)
    const r = zoomedReference(trigger()).getBoundingClientRect()
    expect([r.x, r.y, r.width, r.height]).toEqual([110, 220, 220, 33])
  })

  it('divides it by the page’s zoom', () => {
    document.documentElement.style.setProperty('zoom', '1.1')
    const r = zoomedReference(trigger()).getBoundingClientRect()
    expect([r.x, r.y, r.width, r.height].map(v => Math.round(v))).toEqual([100, 200, 200, 30])
  })
})
