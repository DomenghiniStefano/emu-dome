import { describe, expect, it } from 'vitest'
import type { ScreenPlacement } from '@/emu'
import { screenDraws, screenSmoothing } from './screenDraw'

const bottom = (rotation: ScreenPlacement['rotation']): ScreenPlacement => ({
  id: 'bottom',
  src: { x: 0, y: 192, w: 256, h: 192 },
  dst: { x: 100, y: 50, w: 384, h: 512 },
  rotation,
})

describe('screenDraws', () => {
  it('draws the screen’s part of the frame into its rectangle, device pixels', () => {
    const [d] = screenDraws([{ ...bottom('none'), dst: { x: 10, y: 20, w: 512, h: 384 } }], 2)
    expect(d).toEqual({
      sx: 0,
      sy: 192,
      sw: 256,
      sh: 192,
      cx: (10 + 256) * 2,
      cy: (20 + 192) * 2,
      angle: 0,
      w: 1024,
      h: 768,
    })
  })

  it('turns the picture inside the rectangle Rust gave', () => {
    const [left] = screenDraws([bottom('left')], 1)
    // A quarter turn: the picture is as wide as the rectangle is tall.
    expect([left!.w, left!.h, left!.angle]).toEqual([512, 384, -Math.PI / 2])
    expect([left!.cx, left!.cy]).toEqual([100 + 192, 50 + 256])
    expect(screenDraws([bottom('right')], 1)[0]!.angle).toBe(Math.PI / 2)
    const [flip] = screenDraws([bottom('flip')], 1)
    expect([flip!.w, flip!.h, flip!.angle]).toEqual([384, 512, Math.PI])
  })

  it('keeps Rust’s order: the last one is in front', () => {
    const top: ScreenPlacement = {
      ...bottom('none'),
      id: 'top',
      src: { x: 0, y: 0, w: 256, h: 192 },
    }
    expect(screenDraws([top, bottom('none')], 1).map(d => d.sy)).toEqual([0, 192])
  })
})

describe('screenSmoothing', () => {
  // A GBA 1.5× across a 360 px phone, at a device pixel ratio of 3: 4.5× on the canvas.
  const [gba] = screenDraws(
    [
      {
        id: 'top',
        src: { x: 0, y: 0, w: 240, h: 160 },
        dst: { x: 0, y: 0, w: 360, h: 240 },
        rotation: 'none',
      },
    ],
    3,
  )

  it('keeps the pixels sharp with none', () => {
    expect(screenSmoothing(gba!, 'none')).toEqual({ smooth: false, kx: 1, ky: 1 })
  })

  it('enlarges by the whole scale first with light, then smooths the rest', () => {
    expect(screenSmoothing(gba!, 'light')).toEqual({ smooth: true, kx: 4, ky: 4 })
    // Another value is the default, light.
    expect(screenSmoothing(gba!, 'blur')).toEqual({ smooth: true, kx: 4, ky: 4 })
  })

  it('smooths the frame’s own pixels with soft', () => {
    expect(screenSmoothing(gba!, 'soft')).toEqual({ smooth: true, kx: 1, ky: 1 })
  })

  it('reads the scale along the frame’s axes, never under 1 and not lost to rounding', () => {
    // Turned a quarter: the picture is 512 wide for the screen's 256.
    expect(screenSmoothing(screenDraws([bottom('left')], 1)[0]!, 'light')).toEqual({
      smooth: true,
      kx: 2,
      ky: 2,
    })
    const tiny = { ...gba!, w: 120, h: 80 }
    expect(screenSmoothing(tiny, 'light')).toEqual({ smooth: true, kx: 1, ky: 1 })
    const almost = { ...gba!, w: 240 * 3.9999999, h: 160 * 4 }
    expect(screenSmoothing(almost, 'light').kx).toBe(4)
  })
})
