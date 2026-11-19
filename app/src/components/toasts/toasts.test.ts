import { describe, expect, it } from 'vitest'
import { ToastKind, toastDuration } from './toasts'

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
