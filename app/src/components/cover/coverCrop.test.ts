import { describe, expect, it } from 'vitest'
import {
  baseSize,
  clampCrop,
  CropMode,
  initialCrop,
  moveBy,
  panStep,
  placement,
  viewOf,
  withMode,
  withZoom,
  zoomPercent,
  zoomStep,
  ZOOM_MAX,
} from './coverCrop'

// A GBA screenshot (3:2), a DS screenshot (both screens stacked: 256 × 384, exactly 2:3), a
// tall phone photo.
const GBA = { width: 240, height: 160 }
const DS = { width: 256, height: 384 }
const TALL = { width: 1000, height: 3000 }

describe('coverCrop', () => {
  it('starts with the whole image centred in the 2:3 frame', () => {
    expect(viewOf(GBA, initialCrop(GBA))).toEqual({ x: 0, y: -100, width: 240, height: 360 })
    // The DS is already 2:3: whole and fill are the same picture.
    expect(viewOf(DS, initialCrop(DS))).toEqual({ x: 0, y: 0, width: 256, height: 384 })
    expect(viewOf(DS, initialCrop(DS, CropMode.Fill))).toEqual({
      x: 0,
      y: 0,
      width: 256,
      height: 384,
    })
    expect(viewOf(TALL, initialCrop(TALL))).toEqual({
      x: -500,
      y: 0,
      width: 2000,
      height: 3000,
    })
  })

  it('fills the frame with the middle of the image', () => {
    const base = baseSize(GBA, CropMode.Fill)
    expect(base.width).toBeCloseTo(320 / 3)
    expect(base.height).toBe(160)
    const v = viewOf(GBA, initialCrop(GBA, CropMode.Fill))
    expect(v.x).toBeCloseTo(120 - 160 / 3)
    expect(v.y).toBe(0)
    expect(v.width / v.height).toBeCloseTo(2 / 3)
  })

  it('never lets a filled view leave the image', () => {
    let crop = initialCrop(GBA, CropMode.Fill)
    crop = moveBy(GBA, crop, 5, 5)
    const v = viewOf(GBA, crop)
    expect(v.x).toBeCloseTo(0)
    expect(v.y).toBeCloseTo(0)
    crop = moveBy(GBA, crop, -9, -9)
    const w = viewOf(GBA, crop)
    expect(w.x + w.width).toBeCloseTo(240)
    expect(w.y + w.height).toBeCloseTo(160)
  })

  it('keeps the centre of a whole view on the image', () => {
    const crop = moveBy(GBA, initialCrop(GBA), 10, -10)
    expect(crop.cx).toBe(0)
    expect(crop.cy).toBe(160)
  })

  it('moves the image the way of the drag and of the d-pad', () => {
    const start = withZoom(GBA, initialCrop(GBA), 2)
    // Dragging the image right shows more of its left: the view goes left.
    expect(moveBy(GBA, start, 0.1, 0).cx).toBeLessThan(start.cx)
    expect(panStep(GBA, start, 'right').cx).toBeLessThan(start.cx)
    expect(panStep(GBA, start, 'down').cy).toBeLessThan(start.cy)
    expect(panStep(GBA, start, 'up').cy).toBeGreaterThan(start.cy)
  })

  it('zooms in steps within its range and resets it with the mode', () => {
    let crop = initialCrop(GBA)
    crop = zoomStep(GBA, crop, -1)
    expect(crop.zoom).toBe(1)
    for (let i = 0; i < 3; i++) crop = zoomStep(GBA, crop, 1)
    expect(crop.zoom).toBe(1.3)
    expect(zoomPercent(crop.zoom)).toBe(130)
    expect(viewOf(GBA, crop).width).toBeCloseTo(240 / 1.3)
    expect(withZoom(GBA, crop, 99).zoom).toBe(ZOOM_MAX)
    const filled = withMode(GBA, crop, CropMode.Fill)
    expect(filled).toMatchObject({ mode: CropMode.Fill, zoom: 1 })
  })

  it('places the image in the frame in percent, as Rust draws it', () => {
    const v = viewOf(GBA, initialCrop(GBA))
    expect(placement(GBA, v)).toEqual({
      left: 0,
      top: (100 / 360) * 100,
      width: 100,
      height: (160 / 360) * 100,
    })
  })

  it('clamps a crop from outside', () => {
    const c = clampCrop(GBA, { mode: CropMode.Whole, zoom: 0, cx: -50, cy: 500 })
    expect(c).toEqual({ mode: CropMode.Whole, zoom: 1, cx: 0, cy: 160 })
  })
})
