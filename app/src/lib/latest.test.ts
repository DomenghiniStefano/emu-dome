import { describe, expect, it } from 'vitest'
import { latest, latestBy } from './latest'

describe('latest', () => {
  it('a request stops being fresh once a newer one starts', () => {
    const loads = latest()
    const first = loads.next()
    expect(first()).toBe(true)
    const second = loads.next()
    expect(first()).toBe(false)
    expect(second()).toBe(true)
  })

  it('`now` follows the request started last without starting one', () => {
    const loads = latest()
    loads.next()
    const part = loads.now()
    expect(part()).toBe(true)
    loads.now()
    expect(part()).toBe(true)
    loads.next()
    expect(part()).toBe(false)
  })

  it('an answer that arrives after a newer request is dropped, in whatever order they end', async () => {
    const loads = latest()
    const shown: string[] = []
    const load = async (value: string, ms: number) => {
      const fresh = loads.next()
      await new Promise(r => setTimeout(r, ms))
      if (fresh()) shown.push(value)
    }
    await Promise.all([load('global', 20), load('game', 0)])
    expect(shown).toEqual(['game'])
  })

  it('by key: requests for different keys stay fresh, the same key does not', () => {
    const pages = latestBy<string>()
    const video = pages.next('video')
    const audio = pages.next('audio')
    expect([video(), audio()]).toEqual([true, true])
    pages.next('video')
    expect([video(), audio()]).toEqual([false, true])
  })
})
