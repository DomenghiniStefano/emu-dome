import { describe, expect, it, vi } from 'vitest'
import { fromPhysicalKeyboard, keysShown, showKeyboardKeys, showNavKeys } from './inputMode'

// inputMode.ts reaches the bridge through useDeviceLayout's schema; nothing here calls it.
vi.mock('@/emu', () => ({ emu: {} }))

const key = (over: Partial<KeyboardEvent> = {}) => ({
  isTrusted: true,
  code: 'KeyA',
  key: 'a',
  isComposing: false,
  ...over,
})

describe('fromPhysicalKeyboard', () => {
  it('takes a trusted press with a key code', () => {
    expect(fromPhysicalKeyboard(key())).toBe(true)
    expect(fromPhysicalKeyboard(key({ code: 'Escape', key: 'Escape' }))).toBe(true)
    expect(fromPhysicalKeyboard(key({ code: 'F11', key: 'F11' }))).toBe(true)
  })
  it('refuses a virtual keyboard: no code, Unidentified, composing', () => {
    expect(fromPhysicalKeyboard(key({ code: '', key: 'Enter' }))).toBe(false)
    expect(fromPhysicalKeyboard(key({ code: '', key: 'Unidentified' }))).toBe(false)
    expect(fromPhysicalKeyboard(key({ code: 'Unidentified', key: 'a' }))).toBe(false)
    expect(fromPhysicalKeyboard(key({ key: 'Process' }))).toBe(false)
    expect(fromPhysicalKeyboard(key({ isComposing: true }))).toBe(false)
  })
  it('refuses the phone’s own media and volume keys', () => {
    expect(fromPhysicalKeyboard(key({ code: 'AudioVolumeUp', key: 'AudioVolumeUp' }))).toBe(false)
    expect(fromPhysicalKeyboard(key({ code: 'MediaPlayPause', key: 'MediaPlayPause' }))).toBe(false)
    expect(fromPhysicalKeyboard(key({ code: 'BrowserBack', key: 'BrowserBack' }))).toBe(false)
  })
  it('refuses nav.ts’s own key events for a controller step (untrusted)', () => {
    expect(fromPhysicalKeyboard(key({ isTrusted: false, code: 'ArrowDown' }))).toBe(false)
  })
})

describe('showKeyboardKeys', () => {
  it('always on a computer', () => {
    expect(showKeyboardKeys(false, false)).toBe(true)
    expect(showKeyboardKeys(false, true)).toBe(true)
  })
  it('on the phone only once a physical keyboard was used', () => {
    expect(showKeyboardKeys(true, false)).toBe(false)
    expect(showKeyboardKeys(true, true)).toBe(true)
  })
})

describe('showNavKeys', () => {
  it('always on a computer', () => {
    expect(showNavKeys(false, false, false)).toBe(true)
  })
  it('on the phone with a keyboard used or a controller connected, never on touch alone', () => {
    expect(showNavKeys(true, false, false)).toBe(false)
    expect(showNavKeys(true, true, false)).toBe(true)
    expect(showNavKeys(true, false, true)).toBe(true)
  })
})

describe('keysShown (#148)', () => {
  it('names both inputs on a computer', () => {
    expect(keysShown(false, false, false)).toEqual({ keyboard: true, controller: true })
  })
  it('on the phone, each input only once it is there', () => {
    expect(keysShown(true, false, false)).toEqual({ keyboard: false, controller: false })
    expect(keysShown(true, true, false)).toEqual({ keyboard: true, controller: false })
    expect(keysShown(true, false, true)).toEqual({ keyboard: false, controller: true })
  })
})
