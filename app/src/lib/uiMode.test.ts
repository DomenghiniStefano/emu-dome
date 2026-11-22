import { describe, expect, it } from 'vitest'
import type { PadInfo } from '@/emu'
import {
  activePad,
  decideUiLayout,
  padsAfter,
  tvScale,
  uiPreference,
  type UiInputs,
} from './uiMode'

const pad = (id: number, battery: number | null = null): PadInfo => ({
  id,
  name: `Pad ${id}`,
  family: 'xbox',
  vendor: 0,
  product: 0,
  battery,
})

const inputs = (over: Partial<UiInputs>): UiInputs => ({
  preference: 'auto',
  device: 'desktop',
  fullscreen: false,
  controllers: 0,
  ...over,
})

describe('decideUiLayout', () => {
  it('turns TV on by itself only full screen with a controller', () => {
    expect(decideUiLayout(inputs({ fullscreen: true, controllers: 1 }))).toBe('tv')
    expect(decideUiLayout(inputs({ fullscreen: true, controllers: 0 }))).toBe('desktop')
    expect(decideUiLayout(inputs({ fullscreen: false, controllers: 2 }))).toBe('desktop')
  })

  it('follows a fixed choice whatever the window and the controllers', () => {
    expect(decideUiLayout(inputs({ preference: 'tv' }))).toBe('tv')
    const full = { fullscreen: true, controllers: 1 }
    expect(decideUiLayout(inputs({ preference: 'desktop', ...full }))).toBe('desktop')
  })

  it('never puts a phone in TV mode', () => {
    expect(decideUiLayout(inputs({ device: 'phone', preference: 'tv' }))).toBe('desktop')
    const full = { fullscreen: true, controllers: 1 }
    expect(decideUiLayout(inputs({ device: 'phone', ...full }))).toBe('desktop')
  })

  it('decides before the device is known as on the PC', () => {
    expect(decideUiLayout(inputs({ device: null, preference: 'tv' }))).toBe('tv')
  })
})

describe('uiPreference', () => {
  it('reads the three options and falls back to Automatica', () => {
    expect(uiPreference('tv')).toBe('tv')
    expect(uiPreference('desktop')).toBe('desktop')
    expect(uiPreference('auto')).toBe('auto')
    expect(uiPreference(undefined)).toBe('auto')
    expect(uiPreference('big')).toBe('auto')
  })
})

describe('padsAfter / activePad', () => {
  it('keeps the controller connected last as the active one', () => {
    let pads = padsAfter([], pad(1), 'connected')
    pads = padsAfter(pads, pad(2), 'connected')
    expect(activePad(pads)?.id).toBe(2)
    pads = padsAfter(pads, pad(1), 'connected')
    expect(pads.map(p => p.id)).toEqual([2, 1])
  })

  it('updates a change in place and drops a disconnected one', () => {
    let pads = [pad(1), pad(2)]
    pads = padsAfter(pads, pad(1, 40), 'changed')
    expect(pads.map(p => [p.id, p.battery])).toEqual([
      [1, 40],
      [2, null],
    ])
    expect(padsAfter(pads, pad(3), 'changed').map(p => p.id)).toEqual([1, 2])
    pads = padsAfter(pads, pad(2), 'disconnected')
    expect(activePad(pads)?.id).toBe(1)
    expect(activePad(padsAfter(pads, pad(1), 'disconnected'))).toBeNull()
  })
})

describe('tvScale', () => {
  it('keeps the mockup sizes up to 1920 × 1080, grows them on a larger screen', () => {
    expect(tvScale(1280, 720)).toBe(1)
    expect(tvScale(1920, 1080)).toBe(1)
    expect(tvScale(3840, 2160)).toBe(2)
    expect(tvScale(3840, 1080)).toBe(1)
    expect(tvScale(2560, 1600)).toBeCloseTo(1.333, 3)
  })
})
