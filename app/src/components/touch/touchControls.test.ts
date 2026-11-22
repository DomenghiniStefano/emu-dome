import { describe, expect, it } from 'vitest'
import type { SettingRow } from '@/emu'
import { gameButton, prefsFrom } from './touchControls'
import { DEFAULT_PREFS } from './touchVisibility'

const row = (key: SettingRow['key'], value: SettingRow['value']): SettingRow => ({
  key,
  value,
  source: 'global',
  changed: true,
})

describe('touch controls', () => {
  it('maps the buttons to the game, not the d-pad, the stick or the menu', () => {
    expect(gameButton('a')).toBe('a')
    expect(gameButton('select')).toBe('select')
    expect(gameButton('dpad')).toBeNull()
    expect(gameButton('circlePad')).toBeNull()
    expect(gameButton('menu')).toBeNull()
  })

  it('reads the touch page, with the defaults for what is missing or unknown', () => {
    expect(prefsFrom([])).toEqual({ ...DEFAULT_PREFS, lost: 'pause' })
    expect(
      prefsFrom([
        row('touch.tport', 'custom'),
        row('touch.tland', 'nonsense'),
        row('touch.tskin', 'contrast'),
        row('touch.topac', 35),
        row('touch.thapt', 0),
        row('touch.thide', '10s'),
        row('touch.tpad', false),
        row('touch.tlost', 'touch'),
      ]),
    ).toEqual({
      portrait: 'custom',
      landscape: 'default',
      skin: 'contrast',
      opacity: 35,
      haptics: 0,
      hide: '10s',
      hideWithPad: false,
      lost: 'touch',
    })
  })
})
