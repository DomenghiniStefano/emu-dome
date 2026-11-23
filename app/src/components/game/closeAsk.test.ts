import { describe, expect, it } from 'vitest'
import type { Stats } from '@/emu'
import { asksBeforeClose, saveOutcome } from './closeAsk'

const stats = (over: Partial<Stats>) =>
  ({
    running: true,
    emuFps: 60,
    targetFps: 60,
    speed: null,
    paused: true,
    frames: 10,
    autoSavedAt: null,
    unsaved: false,
    ...over,
  }) as Stats

describe('asksBeforeClose', () => {
  it('asks only when the game moved since the last quick save or load', () => {
    expect(asksBeforeClose(stats({ unsaved: true }))).toBe(true)
    expect(asksBeforeClose(stats({ unsaved: false }))).toBe(false)
  })

  it('never asks with no game running', () => {
    expect(asksBeforeClose(stats({ running: false, unsaved: true }))).toBe(false)
  })

  it('asks when the stats could not be read', () => {
    expect(asksBeforeClose(null)).toBe(true)
  })
})

describe('saveOutcome', () => {
  it('reads the save of the slot waited for, and nothing else', () => {
    expect(saveOutcome({ type: 'stateSaved', slot: 2 }, 2)).toBe('saved')
    expect(saveOutcome({ type: 'stateSaved', slot: 3 }, 2)).toBeNull()
    expect(saveOutcome({ type: 'stateLoaded', slot: 2 }, 2)).toBeNull()
    const failed = (slot: number) =>
      ({
        type: 'error',
        error: { code: 'stateSaveFailed', params: { slot, cause: 'other' } },
      }) as const
    expect(saveOutcome(failed(2), 2)).toBe('failed')
    expect(saveOutcome(failed(0), 2)).toBeNull()
    expect(saveOutcome({ type: 'error', error: { code: 'noGame' } }, 2)).toBeNull()
  })
})
