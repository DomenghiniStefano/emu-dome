import { beforeEach, describe, expect, it, vi } from 'vitest'

const schema = vi.fn()
vi.mock('@/emu', () => ({ emu: { settings: { schema: () => schema() } } }))

describe('loadSchema', () => {
  beforeEach(() => {
    vi.resetModules()
    schema.mockReset()
  })

  it('asks the bridge once and shares the answer', async () => {
    schema.mockResolvedValue({ entries: [] })
    const { loadSchema } = await import('./settingsModel')
    await loadSchema()
    await loadSchema()
    expect(schema).toHaveBeenCalledTimes(1)
  })

  it('asks again after a read that failed', async () => {
    schema.mockRejectedValueOnce(new Error('busy')).mockResolvedValue({ entries: [] })
    const { loadSchema } = await import('./settingsModel')
    await expect(loadSchema()).rejects.toThrow('busy')
    await expect(loadSchema()).resolves.toEqual({ entries: [] })
    expect(schema).toHaveBeenCalledTimes(2)
  })
})

describe('showIfHolds', () => {
  const strength = (values: string[], not = false) =>
    ({ key: 'touch.thapt', showIf: { key: 'touch.tvib', values, not } }) as never

  it('reads a switch by its value as text, like ShowIf::holds', async () => {
    const { showIfHolds } = await import('./settingsModel')
    expect(showIfHolds(strength(['true']), () => true)).toBe(true)
    expect(showIfHolds(strength(['true']), () => false)).toBe(false)
    expect(showIfHolds(strength(['true'], true), () => false)).toBe(true)
    // No value is none of the listed ones.
    expect(showIfHolds(strength(['true']), () => null)).toBe(false)
  })
})
