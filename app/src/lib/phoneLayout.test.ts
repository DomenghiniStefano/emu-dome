import { describe, expect, it } from 'vitest'
import { cameraSide, shellNav, ShellNav } from './phoneLayout'

describe('shellNav', () => {
  it('keeps the rail on the desktop, upright or not, and while the device is not known', () => {
    expect(shellNav('desktop', false)).toBe(ShellNav.Rail)
    expect(shellNav('desktop', true)).toBe(ShellNav.Rail)
    expect(shellNav(null, true)).toBe(ShellNav.Rail)
  })

  it('puts the phone upright on a tab bar and on its side on a column', () => {
    expect(shellNav('phone', true)).toBe(ShellNav.Tabs)
    expect(shellNav('phone', false)).toBe(ShellNav.Column)
  })
})

describe('cameraSide', () => {
  it('follows the way the phone is turned', () => {
    expect(cameraSide('landscape-primary')).toBe('left')
    expect(cameraSide('landscape-secondary')).toBe('right')
  })

  it('falls back to the left when upright or not reported', () => {
    expect(cameraSide('portrait-primary')).toBe('left')
    expect(cameraSide(undefined)).toBe('left')
  })
})
