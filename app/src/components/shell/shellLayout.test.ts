import { describe, expect, it } from 'vitest'
import { hasOwnInitialFocus, isFlush } from './shellLayout'
import { hazeStyle } from './useCoverHaze'

describe('isFlush', () => {
  it('is flush for Settings, Controls and Saves, padded for Library and Cloud', () => {
    expect(isFlush('settings', false)).toBe(true)
    expect(isFlush('controls', false)).toBe(true)
    expect(isFlush('saves', false)).toBe(true)
    expect(isFlush('library', false)).toBe(false)
    expect(isFlush('cloud', false)).toBe(false)
  })

  it('is flush while a game’s details are shown', () => {
    expect(isFlush('library', true)).toBe(true)
  })
})

describe('hasOwnInitialFocus', () => {
  it('leaves the first focus on the rail for a placeholder section only', () => {
    expect(hasOwnInitialFocus('library')).toBe(true)
    expect(hasOwnInitialFocus('saves')).toBe(true)
    expect(hasOwnInitialFocus('cloud')).toBe(false)
  })
})

describe('hazeStyle', () => {
  it('sets the three cover colours', () => {
    expect(hazeStyle(['#111111', '#222222', '#333333'])).toEqual({
      '--cover-1': '#111111',
      '--cover-2': '#222222',
      '--cover-3': '#333333',
    })
  })

  it('sets nothing without colours: the fallback spots show', () => {
    expect(hazeStyle(null)).toEqual({})
  })
})
