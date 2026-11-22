import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { hideSplash, SPLASH_REMOVE_AFTER_MS } from '@/lib/splash'

function addSplash() {
  const splash = document.createElement('div')
  splash.id = 'splash'
  document.body.append(splash)
  return splash
}

describe('hideSplash', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
    document.body.replaceChildren()
  })

  it('fades the splash out, then removes it when the fade ends', () => {
    const splash = addSplash()
    hideSplash()
    expect(splash.dataset.state).toBe('out')
    expect(splash.isConnected).toBe(true)
    splash.dispatchEvent(new Event('transitionend'))
    expect(splash.isConnected).toBe(false)
  })

  it('removes it anyway when no fade ends', () => {
    const splash = addSplash()
    hideSplash()
    vi.advanceTimersByTime(SPLASH_REMOVE_AFTER_MS - 1)
    expect(splash.isConnected).toBe(true)
    vi.advanceTimersByTime(1)
    expect(splash.isConnected).toBe(false)
  })

  it('does nothing without a splash, or a second time', () => {
    expect(() => hideSplash()).not.toThrow()
    const splash = addSplash()
    hideSplash()
    hideSplash()
    vi.runAllTimers()
    expect(splash.isConnected).toBe(false)
  })
})
