import { describe, expect, it } from 'vitest'
import type { PadInfo } from '@/emu'
import {
  activePad,
  askAfter,
  asksOnConnect,
  asksOnDisconnect,
  decideUiLayout,
  padsAfter,
  rememberedPreference,
  tvScale,
  uiPreference,
  type AskInputs,
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
  answered: null,
  ...over,
})

describe('decideUiLayout', () => {
  it('Automatica never switches by itself: the desktop until the user answers TV', () => {
    expect(decideUiLayout(inputs({}))).toBe('desktop')
    expect(decideUiLayout(inputs({ answered: 'tv' }))).toBe('tv')
    expect(decideUiLayout(inputs({ answered: 'desktop' }))).toBe('desktop')
  })

  it('follows a fixed choice whatever was answered', () => {
    expect(decideUiLayout(inputs({ preference: 'tv' }))).toBe('tv')
    expect(decideUiLayout(inputs({ preference: 'desktop', answered: 'tv' }))).toBe('desktop')
  })

  it('never puts a phone in TV mode', () => {
    expect(decideUiLayout(inputs({ device: 'phone', preference: 'tv' }))).toBe('desktop')
    expect(decideUiLayout(inputs({ device: 'phone', answered: 'tv' }))).toBe('desktop')
  })

  it('decides before the device is known as on the PC', () => {
    expect(decideUiLayout(inputs({ device: null, preference: 'tv' }))).toBe('tv')
  })
})

describe('the TV question', () => {
  const ask = (over: Partial<AskInputs>): AskInputs => ({
    preference: 'auto',
    device: 'desktop',
    tv: false,
    controllers: 1,
    ...over,
  })

  it('asks to switch when a controller is there, Automatica, on the desktop', () => {
    expect(asksOnConnect(ask({}))).toBe(true)
    expect(asksOnConnect(ask({ tv: true }))).toBe(false)
    expect(asksOnConnect(ask({ preference: 'desktop' }))).toBe(false)
    expect(asksOnConnect(ask({ preference: 'tv' }))).toBe(false)
    expect(asksOnConnect(ask({ device: 'phone' }))).toBe(false)
    expect(asksOnConnect(ask({ device: null }))).toBe(false)
    expect(asksOnConnect(ask({ controllers: 0 }))).toBe(false)
  })

  it('asks to go back when the last controller leaves TV mode, Automatica only', () => {
    expect(asksOnDisconnect(ask({ tv: true, controllers: 0 }))).toBe(true)
    expect(asksOnDisconnect(ask({ tv: true, controllers: 1 }))).toBe(false)
    expect(asksOnDisconnect(ask({ tv: false, controllers: 0 }))).toBe(false)
    expect(asksOnDisconnect(ask({ preference: 'tv', tv: true, controllers: 0 }))).toBe(false)
  })

  it('a reconnection closes «go back», the last controller leaving closes «switch»', () => {
    expect(askAfter('disconnect', ask({ controllers: 1 }))).toBeNull()
    expect(askAfter('disconnect', ask({ controllers: 0 }))).toBe('disconnect')
    expect(askAfter('connect', ask({ controllers: 0 }))).toBeNull()
    expect(askAfter('connect', ask({ controllers: 2 }))).toBe('connect')
    expect(askAfter(null, ask({}))).toBeNull()
  })

  it('«Non chiederlo più» saves the answer as the fixed layout', () => {
    expect(rememberedPreference('tv')).toBe('tv')
    expect(rememberedPreference('desktop')).toBe('desktop')
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
