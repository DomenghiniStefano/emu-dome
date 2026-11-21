// The game flow's watchdogs (E11 hang, E12 slow): module-level state, reset by `gameStarted`.
import { beforeEach, describe, expect, it } from 'vitest'
import type { Stats } from '@/emu'
import { HANG_MS, SLOW_MS, flow, gameStarted, sample } from './gameFlow'

const stats = (frames: number, over: Partial<Stats> = {}): Stats => ({
  running: true,
  emuFps: 60,
  targetFps: 60,
  speed: { kind: 'factor', value: 1 },
  paused: false,
  frames,
  autoSavedAt: null,
  ...over,
})
const slow = (frames: number) => stats(frames, { emuFps: 40 })

beforeEach(() => gameStarted())

describe('E11: the core stops answering', () => {
  it('frames standing still for HANG_MS while running is a hang, not a moment before', () => {
    expect(sample(stats(10), true, 0)).toBeNull()
    expect(sample(stats(10), true, HANG_MS - 1)).toBeNull()
    expect(sample(stats(10), true, HANG_MS)).toBe('hang')
  })

  it('frames moving reset the clock', () => {
    sample(stats(10), true, 0)
    sample(stats(11), true, HANG_MS - 1)
    expect(sample(stats(11), true, HANG_MS + 100)).toBeNull()
    expect(sample(stats(11), true, 2 * HANG_MS - 1)).toBe('hang')
  })

  it('paused, a menu open or the core not running is never a hang, and restarts the clock', () => {
    sample(stats(10), true, 0)
    expect(sample(stats(10), false, HANG_MS * 3)).toBeNull()
    expect(sample(stats(10, { running: false }), true, HANG_MS * 4)).toBeNull()
    // Running again: the still frames count from the last paused sample.
    expect(sample(stats(10), true, HANG_MS * 4 + HANG_MS - 1)).toBeNull()
    expect(sample(stats(10), true, HANG_MS * 5)).toBe('hang')
  })
})

describe('E12: slower than normal', () => {
  it('under 90% of the target for SLOW_MS at 1× is slow, once per game', () => {
    let f = 0
    expect(sample(slow(f++), true, 0)).toBeNull()
    expect(sample(slow(f++), true, SLOW_MS - 1)).toBeNull()
    expect(sample(slow(f++), true, SLOW_MS)).toBe('slow')
    expect(sample(slow(f++), true, SLOW_MS * 3)).toBeNull()
    // A new game may say it again.
    gameStarted()
    expect(sample(slow(f++), true, SLOW_MS * 4)).toBeNull()
    expect(sample(slow(f), true, SLOW_MS * 5)).toBe('slow')
  })

  it('a moment back at speed starts the count again', () => {
    let f = 0
    sample(slow(f++), true, 0)
    sample(stats(f++), true, SLOW_MS - 1)
    expect(sample(slow(f++), true, SLOW_MS)).toBeNull()
    expect(sample(slow(f), true, 2 * SLOW_MS)).toBe('slow')
  })

  it('fast forward, slow motion and Max are never slow', () => {
    let f = 0
    for (const speed of [
      { kind: 'factor', value: 2 },
      { kind: 'factor', value: 0.5 },
      { kind: 'max' },
    ] as const) {
      gameStarted()
      sample(stats(f++, { emuFps: 10, speed }), true, 0)
      expect(sample(stats(f++, { emuFps: 10, speed }), true, SLOW_MS * 2)).toBeNull()
    }
  })

  it('exactly 90% is not slow, and an unknown target never is', () => {
    let f = 0
    sample(stats(f++, { emuFps: 54 }), true, 0)
    expect(sample(stats(f++, { emuFps: 54 }), true, SLOW_MS * 2)).toBeNull()
    gameStarted()
    sample(stats(f++, { emuFps: 10, targetFps: 0 }), true, 0)
    expect(sample(stats(f, { emuFps: 10, targetFps: 0 }), true, SLOW_MS * 2)).toBeNull()
  })
})

describe('gameStarted', () => {
  it('clears the previous game’s E11 / E12 dialogs', () => {
    flow.crashed = { console: 'gba', at: null, detail: '' }
    flow.slow = { fps: 40, target: 60 }
    gameStarted()
    expect(flow.crashed).toBeNull()
    expect(flow.slow).toBeNull()
  })
})
