import { beforeEach, describe, expect, it, vi } from 'vitest'
import catalog from '@/generated/settings-catalog.json'

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
      consoles: ['gb', 'gbc', 'gba', 'nds', '3ds'],
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

describe('lockReason of a level the row does not accept (#168)', () => {
  // The levels each reason sends the user to.
  const SENDS_TO: Record<string, string[]> = {
    'settingsUi.lockedGlobal': ['global'],
    'settingsUi.lockedForConsole': ['console'],
    'settingsUi.lockedForGame': ['game'],
    'settingsUi.lockedScoped': ['console', 'game'],
  }
  const desktop = { device: 'desktop' } as never

  it('names only levels every entry of the catalogue accepts, never the one it is locked at', async () => {
    const { lockReason } = await import('./settingsModel')
    let checked = 0
    for (const e of catalog.entries) {
      const entry = {
        key: e.key,
        kind: { type: e.kind },
        platform: 'all',
        levels: e.levels,
        consoles: e.consoles,
        unsupportedConsoles: [],
        needsNativeVideo: false,
      } as never
      for (const level of ['global', 'console', 'game'] as const) {
        if (e.levels.includes(level)) continue
        // A console the entry is about: the scope bar offers no other (#176).
        const console = level === 'global' ? null : (e.consoles[0] as never)
        const reason = lockReason(entry, desktop, level, console)
        if (reason === null) continue // actions, and a layout editor's row at «Tutti i giochi»
        const to = SENDS_TO[reason]
        expect(to, `${e.key} at ${level}: ${reason}`).toBeDefined()
        for (const l of to ?? []) {
          expect(e.levels, `${e.key} at ${level}: ${reason}`).toContain(l)
          expect(l, `${e.key} at ${level}`).not.toBe(level)
        }
        checked++
      }
    }
    expect(checked).toBeGreaterThan(0)
  })

  it('sends the game clock to the game’s level, from «Tutti i giochi» and from a console', async () => {
    const { lockReason } = await import('./settingsModel')
    const rtc = catalog.entries.find(e => e.key === 'saves.rtc')
    expect(rtc?.levels).toEqual(['game', 'session'])
    const entry = { ...rtc, kind: { type: 'choice' }, platform: 'all', unsupportedConsoles: [] }
    expect(lockReason(entry as never, desktop, 'global')).toBe('settingsUi.lockedForGame')
    expect(lockReason(entry as never, desktop, 'console', 'nds')).toBe('settingsUi.lockedForGame')
  })
})

describe('levelsReason', () => {
  it('has one text per set of levels that can lock a row', async () => {
    const { levelsReason } = await import('./settingsModel')
    expect(levelsReason(['global', 'game', 'session'])).toBe('settingsUi.lockedGlobal')
    expect(levelsReason(['console', 'session'])).toBe('settingsUi.lockedForConsole')
    expect(levelsReason(['game', 'session'])).toBe('settingsUi.lockedForGame')
    expect(levelsReason(['console', 'game', 'session'])).toBe('settingsUi.lockedScoped')
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
