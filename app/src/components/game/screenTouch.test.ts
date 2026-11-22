import { describe, expect, it } from 'vitest'
import { bottomInset, insetShares, NO_INSET } from './inset'
import { isInterfaceTarget, pointerArgs, viewArea } from './screenTouch'

const view = { innerWidth: 1280, innerHeight: 720, devicePixelRatio: 1.5 }
const none = insetShares(NO_INSET, 1, 1)

describe('pointerArgs', () => {
  it('measures in physical pixels for the native video', () => {
    expect(pointerArgs(100, 200, view, 'native', NO_INSET)).toEqual({
      x: 150,
      y: 300,
      area: { width: 1920, height: 1080, inset: none },
    })
  })

  it('measures in CSS pixels for the webview canvas', () => {
    expect(pointerArgs(100, 200, view, 'webview', NO_INSET)).toEqual({
      x: 100,
      y: 200,
      area: { width: 1280, height: 720, inset: none },
    })
  })

  it('gives the inset as shares of the window, clamped', () => {
    expect(pointerArgs(0, 0, view, 'webview', bottomInset(216)).area.inset.bottom).toBeCloseTo(0.3)
    expect(pointerArgs(0, 0, view, 'webview', bottomInset(5000)).area.inset.bottom).toBe(0.9)
    const empty = { ...view, innerHeight: 0 }
    expect(pointerArgs(0, 0, empty, 'webview', bottomInset(10)).area.inset.bottom).toBe(0)
    const sides = { ...NO_INSET, left: 128, right: 320 }
    expect(pointerArgs(0, 0, view, 'native', sides).area.inset).toEqual({
      ...none,
      left: 0.1,
      right: 0.25,
    })
  })
})

describe('viewArea', () => {
  it('is the area the canvas asks its placements in and the touch is measured against', () => {
    const bar = bottomInset(216)
    expect(viewArea(view, 1, bar)).toEqual(pointerArgs(0, 0, view, 'webview', bar).area)
    expect(viewArea(view, 1.5, NO_INSET)).toEqual(pointerArgs(0, 0, view, 'native', NO_INSET).area)
  })
})

describe('isInterfaceTarget', () => {
  it('tells interface elements from the game', () => {
    const pad = document.createElement('div')
    pad.setAttribute('role', 'group')
    const arm = document.createElement('span')
    pad.append(arm)
    const button = document.createElement('button')
    const plain = document.createElement('div')
    expect(isInterfaceTarget(arm)).toBe(true)
    expect(isInterfaceTarget(button)).toBe(true)
    expect(isInterfaceTarget(plain)).toBe(false)
    expect(isInterfaceTarget(null)).toBe(false)
  })
})
