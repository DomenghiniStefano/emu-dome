import { describe, expect, it } from 'vitest'
import type { SettingDevice } from '@/emu'
import { asksOnDisconnect, type UiPreference } from '@/lib/uiMode'
import { lostChoice, onControllerLost, type LossInputs } from './controllerLoss'
import { hideAfter, hideAfterMs, touchShown, type ShownInputs } from './touchVisibility'

describe('when the touch controls show', () => {
  const base: ShownInputs = {
    touch: true,
    inGame: true,
    menuOpen: false,
    editing: false,
    hideWithPad: true,
    controllers: 0,
  }

  it('in game on a touch screen, not under the menu or the editor', () => {
    expect(touchShown(base)).toBe(true)
    expect(touchShown({ ...base, touch: false })).toBe(false)
    expect(touchShown({ ...base, inGame: false })).toBe(false)
    expect(touchShown({ ...base, menuOpen: true })).toBe(false)
    expect(touchShown({ ...base, editing: true })).toBe(false)
  })

  it('hidden with a controller only when `touch.tpad` says so', () => {
    expect(touchShown({ ...base, controllers: 1 })).toBe(false)
    expect(touchShown({ ...base, controllers: 1, hideWithPad: false })).toBe(true)
  })

  it('reads `touch.thide`', () => {
    expect(hideAfterMs(hideAfter('5s'))).toBe(5000)
    expect(hideAfterMs(hideAfter('10s'))).toBe(10000)
    expect(hideAfterMs(hideAfter('never'))).toBeNull()
    expect(hideAfterMs(hideAfter(42))).toBeNull()
  })
})

describe('E14: the last controller gone while playing', () => {
  const phone: LossInputs = { device: 'phone', inGame: true, menuOpen: false, busy: false, left: 0 }

  it('asks or switches to touch on a phone, as `touch.tlost` says', () => {
    expect(onControllerLost(phone, lostChoice('pause'))).toBe('ask')
    expect(onControllerLost(phone, lostChoice('touch'))).toBe('toast')
    expect(lostChoice(undefined)).toBe('pause')
  })

  it('does nothing while another controller is left, out of a game, under the menu or a dialog', () => {
    expect(onControllerLost({ ...phone, left: 1 }, 'pause')).toBeNull()
    expect(onControllerLost({ ...phone, inGame: false }, 'pause')).toBeNull()
    expect(onControllerLost({ ...phone, menuOpen: true }, 'pause')).toBeNull()
    expect(onControllerLost({ ...phone, busy: true }, 'touch')).toBeNull()
  })

  it('never shows together with the TV question: E14 on phones, the TV question elsewhere', () => {
    const devices: (SettingDevice | null)[] = ['phone', 'desktop', null]
    const prefs: UiPreference[] = ['auto', 'desktop', 'tv']
    for (const device of devices)
      for (const preference of prefs)
        for (const tv of [false, true]) {
          const e14 = onControllerLost({ ...phone, device }, 'pause') !== null
          const tvAsk = asksOnDisconnect({ preference, device, tv, controllers: 0 })
          expect(e14 && tvAsk, `${device} ${preference} ${tv}`).toBe(false)
        }
  })
})
