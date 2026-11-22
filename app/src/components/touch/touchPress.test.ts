import { describe, expect, it } from 'vitest'
import type { Button } from '@/emu'
import { createPresser, TURBO_HALF_MS, type Every } from './touchPress'

/** A fake clock: `tick()` runs every timer once, as if half a turbo cycle passed. */
function harness() {
  const sent: [Button, boolean][] = []
  const timers = new Set<() => void>()
  const every: Every = (ms, tick) => {
    expect(ms).toBe(TURBO_HALF_MS)
    timers.add(tick)
    return () => timers.delete(tick)
  }
  const presser = createPresser((b, on) => sent.push([b, on]), every)
  return { sent, presser, tick: () => [...timers].forEach(t => t()), timers }
}

describe('touch button behaviours', () => {
  it('tap: pressed while touched', () => {
    const { sent, presser } = harness()
    expect(presser.down('a', 'tap')).toBe(true)
    presser.up('a', 'tap')
    expect(sent).toEqual([
      ['a', true],
      ['a', false],
    ])
  })

  it('hold: a tap holds it, the next lets it go; lifting the finger does nothing', () => {
    const { sent, presser } = harness()
    presser.down('b', 'hold')
    presser.up('b', 'hold')
    expect(presser.latched('b')).toBe(true)
    expect(sent).toEqual([['b', true]])
    presser.down('b', 'hold')
    presser.up('b', 'hold')
    expect(presser.latched('b')).toBe(false)
    expect(sent).toEqual([
      ['b', true],
      ['b', false],
    ])
  })

  it('turbo: repeats while touched, and ends released', () => {
    const { sent, presser, tick, timers } = harness()
    presser.down('a', 'turbo')
    tick()
    tick()
    tick()
    expect(sent).toEqual([
      ['a', true],
      ['a', false],
      ['a', true],
      ['a', false],
    ])
    // A second finger on it starts nothing more.
    expect(presser.down('a', 'turbo')).toBe(false)
    presser.up('a', 'turbo')
    expect(timers.size).toBe(0)
    expect(sent.at(-1)).toEqual(['a', false])
  })

  it('releaseAll lets go of every held, latched and repeating button', () => {
    const { sent, presser, timers } = harness()
    presser.down('a', 'tap')
    presser.down('b', 'hold')
    presser.down('x', 'turbo')
    sent.length = 0
    presser.releaseAll()
    expect(timers.size).toBe(0)
    expect(presser.latched('b')).toBe(false)
    expect(sent.sort()).toEqual([
      ['a', false],
      ['b', false],
      ['x', false],
    ])
    presser.releaseAll()
    expect(sent).toHaveLength(3)
  })
})
