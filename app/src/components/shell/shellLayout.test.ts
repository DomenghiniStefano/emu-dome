import { describe, expect, it } from 'vitest'
import { hasOwnInitialFocus, isFlush, isTvLaidOut, sectionStep } from './shellLayout'
import { tvHints } from './tvHints'
import { hazeStyle } from './useCoverHaze'

describe('isFlush', () => {
  it('is flush for Settings, Controls, Saves and Cloud, padded for Library', () => {
    expect(isFlush('settings', false)).toBe(true)
    expect(isFlush('controls', false)).toBe(true)
    expect(isFlush('saves', false)).toBe(true)
    expect(isFlush('cloud', false)).toBe(true)
    expect(isFlush('library', false)).toBe(false)
  })

  it('is flush while a game’s details are shown', () => {
    expect(isFlush('library', true)).toBe(true)
  })

  it('is flush for the TV library, whose shelves reach the edge', () => {
    expect(isFlush('library', false, true)).toBe(true)
  })
})

describe('isTvLaidOut', () => {
  it('has a TV layout for the library only, not for a game’s details or the others', () => {
    expect(isTvLaidOut('library', false)).toBe(true)
    expect(isTvLaidOut('library', true)).toBe(false)
    expect(isTvLaidOut('settings', false)).toBe(false)
  })
})

describe('sectionStep', () => {
  it('steps through the sections in the rail order, round the ends', () => {
    expect(sectionStep('library', 1)).toBe('saves')
    expect(sectionStep('saves', -1)).toBe('library')
    expect(sectionStep('library', -1)).toBe('settings')
    expect(sectionStep('settings', 1)).toBe('library')
  })
})

describe('tvHints', () => {
  it('names play, details, search, back and the sections in the library', () => {
    expect(tvHints('library').map(h => h.actions.join(' '))).toEqual([
      'confirm',
      'more',
      'search',
      'back',
      'prev next',
    ])
  })

  it('names choose, back and the sections elsewhere and in a game’s details', () => {
    const elsewhere = ['tv.hint.choose', 'tv.hint.back', 'tv.hint.sections']
    expect(tvHints('settings').map(h => h.label)).toEqual(elsewhere)
    expect(tvHints('library', true).map(h => h.label)).toEqual(elsewhere)
  })
})

describe('hasOwnInitialFocus', () => {
  it('gives every section its own first focus', () => {
    expect(hasOwnInitialFocus('library')).toBe(true)
    expect(hasOwnInitialFocus('saves')).toBe(true)
    expect(hasOwnInitialFocus('cloud')).toBe(true)
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
