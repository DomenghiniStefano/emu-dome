import { describe, expect, it } from 'vitest'
import { ToastKind, toastAreaStyle, toastDuration } from './toasts'

describe('toastAreaStyle', () => {
  it('spans the sections room and keeps 12 above its bottom edge (#142)', () => {
    // Upright: the whole width, above a 60 px tab bar.
    expect(toastAreaStyle({ left: 0, right: 0, bottom: 60 })).toEqual({
      left: '0px',
      right: '0px',
      bottom: 'calc(60px + var(--spacing-12))',
    })
    // On its side, the column on the camera's left: the stack starts past it.
    expect(toastAreaStyle({ left: 98, right: 24, bottom: 0 }).left).toBe('98px')
  })
})

describe('toastDuration', () => {
  it('keeps a notice 2 s, in game or not', () => {
    expect(toastDuration(ToastKind.Info, false)).toBe(2000)
    expect(toastDuration(ToastKind.Info, true)).toBe(2000)
  })
  it('keeps an error 5 s outside the game, 2 s in it', () => {
    expect(toastDuration(ToastKind.Error, false)).toBe(5000)
    expect(toastDuration(ToastKind.Error, true)).toBe(2000)
  })
  it('takes the duration asked', () => {
    expect(toastDuration(ToastKind.Info, true, 5000)).toBe(5000)
    expect(toastDuration(ToastKind.Error, false, 0)).toBe(0)
  })
})
