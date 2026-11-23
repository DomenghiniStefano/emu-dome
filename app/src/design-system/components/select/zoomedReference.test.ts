import { afterEach, describe, expect, it } from 'vitest'
import { pageZoom, zoomedBoundary, zoomedReference, zoomedViewport } from './zoomedReference'

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

describe('zoomedViewport', () => {
  afterEach(() => {
    document.documentElement.style.removeProperty('zoom')
  })

  it('is the window at 100%, and the window over the zoom when scaled, read each time', () => {
    const v = zoomedViewport()
    expect([v.x, v.y, v.width, v.height]).toEqual([0, 0, window.innerWidth, window.innerHeight])
    document.documentElement.style.setProperty('zoom', '1.2')
    expect(v.width).toBeCloseTo(window.innerWidth / 1.2)
    expect(v.height).toBeCloseTo(window.innerHeight / 1.2)
  })

  it('is the boundary every popper gets, read each time', () => {
    const b = zoomedBoundary() as unknown as DOMRectReadOnly
    document.documentElement.style.setProperty('zoom', '1.2')
    expect([b.x, b.y]).toEqual([0, 0])
    expect(b.width).toBeCloseTo(window.innerWidth / 1.2)
    expect(b.height).toBeCloseTo(window.innerHeight / 1.2)
  })
})
