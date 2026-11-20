import { describe, expect, it } from 'vitest'
import type { ClosingProgress } from '@/emu'
import { closingView } from './closing'

const progress = (
  phase: ClosingProgress['phase'],
  purpose: ClosingProgress['purpose'] = 'quit',
  error: ClosingProgress['error'] = null,
): ClosingProgress => ({ phase, purpose, error })

const diskFull = { code: 'saveFileFailed', params: { detail: 'disk full' } } as const

describe('closingView', () => {
  it('shows nothing before a close', () => {
    expect(closingView(null)).toEqual({ status: null, ask: null })
  })

  it('shows saving, then saved, when the app closes', () => {
    expect(closingView(progress('saving'))).toEqual({ status: 'saving', ask: null })
    expect(closingView(progress('saved'))).toEqual({ status: 'saved', ask: null })
  })

  it('keeps saving behind the question when the app closes', () => {
    expect(closingView(progress('timeout'))).toEqual({
      status: 'saving',
      ask: { kind: 'timeout' },
    })
    expect(closingView(progress('failed', 'quit', diskFull))).toEqual({
      status: 'saving',
      ask: { kind: 'failed', error: diskFull },
    })
  })

  it('leaves the scene of a game closing to the library to G14, but still asks', () => {
    expect(closingView(progress('saving', 'game'))).toEqual({ status: null, ask: null })
    expect(closingView(progress('saved', 'game'))).toEqual({ status: null, ask: null })
    expect(closingView(progress('timeout', 'game'))).toEqual({
      status: null,
      ask: { kind: 'timeout' },
    })
    expect(closingView(progress('failed', 'game', diskFull))).toEqual({
      status: null,
      ask: { kind: 'failed', error: diskFull },
    })
  })

  it('clears once the close is over', () => {
    expect(closingView(progress('closed'))).toEqual({ status: null, ask: null })
    expect(closingView(progress('closed', 'game'))).toEqual({ status: null, ask: null })
  })
})
