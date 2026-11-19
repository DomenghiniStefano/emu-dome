import { describe, expect, it } from 'vitest'
import { dpadDirections } from './dpad'

describe('dpadDirections', () => {
  it('holds nothing in the centre', () => {
    expect(dpadDirections(0, 0)).toEqual([])
    expect(dpadDirections(0.2, 0.1)).toEqual([])
  })
  it('holds one direction along an axis', () => {
    expect(dpadDirections(1, 0)).toEqual(['right'])
    expect(dpadDirections(-1, 0)).toEqual(['left'])
    expect(dpadDirections(0, 1)).toEqual(['down'])
    expect(dpadDirections(0, -1)).toEqual(['up'])
  })
  it('holds two on a diagonal', () => {
    expect(dpadDirections(0.7, -0.7)).toEqual(['right', 'up'])
    expect(dpadDirections(-0.7, 0.7)).toEqual(['left', 'down'])
  })
  it('keeps one direction up to 22.5° off its axis', () => {
    expect(dpadDirections(1, Math.tan(Math.PI / 9))).toEqual(['right'])
  })
})
