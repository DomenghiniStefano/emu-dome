import { describe, expect, it } from 'vitest'
import { isInterfaceTarget, pointerArgs } from './screenTouch'

const view = { innerWidth: 1280, innerHeight: 720, devicePixelRatio: 1.5 }

describe('pointerArgs', () => {
  it('measures in physical pixels for the native video', () => {
    expect(pointerArgs(100, 200, view, 'native', 0)).toEqual({
      x: 150,
      y: 300,
      area: { width: 1920, height: 1080, inset: 0 },
    })
  })

  it('measures in CSS pixels for the webview canvas', () => {
    expect(pointerArgs(100, 200, view, 'webview', 0)).toEqual({
      x: 100,
      y: 200,
      area: { width: 1280, height: 720, inset: 0 },
    })
  })

  it('gives the inset as a share of the height, clamped', () => {
    expect(pointerArgs(0, 0, view, 'webview', 216).area.inset).toBeCloseTo(0.3)
    expect(pointerArgs(0, 0, view, 'webview', 5000).area.inset).toBe(0.9)
    expect(pointerArgs(0, 0, { ...view, innerHeight: 0 }, 'webview', 10).area.inset).toBe(0)
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
