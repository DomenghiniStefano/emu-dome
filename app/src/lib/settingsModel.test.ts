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
