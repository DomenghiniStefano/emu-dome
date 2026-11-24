import { beforeEach, describe, expect, it, vi } from 'vitest'

const schema = vi.fn()
vi.mock('@/emu', () => ({ emu: { settings: { schema: () => schema() } } }))

describe('loadSchema', () => {
  beforeEach(() => {
    vi.resetModules()
    schema.mockReset()
  })

  it('asks the bridge once and shares the answer', async () => {
    schema.mockResolvedValue({ entries: [] })
    const { loadSchema } = await import('./settingsModel')
    await loadSchema()
    await loadSchema()
    expect(schema).toHaveBeenCalledTimes(1)
  })

  it('asks again after a read that failed', async () => {
    schema.mockRejectedValueOnce(new Error('busy')).mockResolvedValue({ entries: [] })
    const { loadSchema } = await import('./settingsModel')
    await expect(loadSchema()).rejects.toThrow('busy')
    await expect(loadSchema()).resolves.toEqual({ entries: [] })
    expect(schema).toHaveBeenCalledTimes(2)
  })
})

describe('showIfHolds', () => {
  const strength = (values: string[], not = false) =>
    ({ key: 'touch.thapt', showIf: { key: 'touch.tvib', values, not } }) as never

  it('reads a switch by its value as text, like ShowIf::holds', async () => {
    const { showIfHolds } = await import('./settingsModel')
    expect(showIfHolds(strength(['true']), () => true)).toBe(true)
    expect(showIfHolds(strength(['true']), () => false)).toBe(false)
    expect(showIfHolds(strength(['true'], true), () => false)).toBe(true)
    // No value is none of the listed ones.
    expect(showIfHolds(strength(['true']), () => null)).toBe(false)
  })
})

describe('lockReason of a layout editor row (Trello #111)', () => {
  const row = (type: string, platform = 'all', levels = ['console']) =>
    ({
      key: type === 'touchLayout' ? 'touch.tedit' : 'screens.custom',
      kind: { type },
      platform,
      levels,
      unsupportedConsoles: [],
      needsNativeVideo: false,
    }) as never
  const on = (device: string) => ({ device }) as never

  it('opens at «Tutti i giochi», where it asks for the console', async () => {
    const { lockReason } = await import('./settingsModel')
    expect(
      lockReason(row('screenLayout', 'all', ['console', 'game']), on('desktop'), 'global'),
    ).toBe(null)
    expect(lockReason(row('touchLayout', 'phone'), on('phone'), 'global')).toBe(null)
    expect(lockReason(row('touchLayout', 'phone'), on('phone'), 'console', 'gba')).toBe(null)
  })

  it('keeps the touch controls to the phone', async () => {
    const { lockReason } = await import('./settingsModel')
    expect(lockReason(row('touchLayout', 'phone'), on('desktop'), 'global')).toBe(
      'settingsUi.lockedPhone',
    )
  })
})

describe('isShown', () => {
  it('hides the touch screens entry, edited with the touch.tedit row', async () => {
    const { isShown } = await import('./settingsModel')
    const entry = { phase: 1, kind: { type: 'touchScreens' } }
    expect(isShown(entry as never, { currentPhase: 1 } as never)).toBe(false)
  })
})

describe('descWithKeys (#148)', () => {
  const texts: Record<string, string> = {
    'settings.video.fps.keys.keyboard': 'On the keyboard: F9.',
    'settings.video.fps.keys.controller': 'On the controller: L3 + X.',
    'settings.speed.ffmode.keys.controller': 'On the controller: hold L3 + R2.',
  }
  const has = (k: string) => k in texts
  const text = (k: string) => texts[k] ?? k
  const base = 'A counter in a corner of the screen.'

  it('adds the part of each input shown, keyboard first', async () => {
    const { descWithKeys } = await import('./settingsModel')
    expect(descWithKeys('video.fps', base, { keyboard: true, controller: true }, has, text)).toBe(
      'A counter in a corner of the screen. On the keyboard: F9. On the controller: L3 + X.',
    )
    expect(descWithKeys('video.fps', base, { keyboard: false, controller: true }, has, text)).toBe(
      'A counter in a corner of the screen. On the controller: L3 + X.',
    )
  })
  it('names no key on a touch-only phone', async () => {
    const { descWithKeys } = await import('./settingsModel')
    expect(descWithKeys('video.fps', base, { keyboard: false, controller: false }, has, text)).toBe(
      base,
    )
  })
  it('skips an input the entry has no part for, and an empty base', async () => {
    const { descWithKeys, keyWords } = await import('./settingsModel')
    expect(
      descWithKeys('speed.ffmode', 'Tab.', { keyboard: true, controller: false }, has, text),
    ).toBe('Tab.')
    expect(keyWords('speed.ffmode', has, text)).toBe('On the controller: hold L3 + R2.')
    expect(keyWords('video.res', has, text)).toBe('')
  })
})
