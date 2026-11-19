import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameSaves, SettingRow, SettingsSchema } from '../../emu'
import {
  DEFAULT_SLOT_COUNT,
  NOTHING_TO_RESUME,
  loadGameSaves,
  loadResumePoint,
  loadSlotCount,
  slotCount,
} from './gameSaves'

// A bridge with the schema, the `saves` page and the saves of one game.
const bridge = vi.hoisted(() => ({
  schema: vi.fn(),
  page: vi.fn(),
  game: vi.fn(),
}))
vi.mock('../../emu', () => ({
  emu: {
    settings: { schema: bridge.schema, page: bridge.page },
    saves: { game: bridge.game },
  },
}))

const schema = {
  entries: [{ key: 'saves.slots', page: 'saves' }],
} as unknown as SettingsSchema
const row = (key: string, value: SettingRow['value']): SettingRow => ({
  key,
  value,
  source: 'global',
  changed: false,
})
const saves = (quick: GameSaves['quick'], inGame: GameSaves['inGame'] = null) =>
  ({ quick, inGame }) as GameSaves

beforeEach(() => {
  vi.clearAllMocks()
  bridge.schema.mockResolvedValue(schema)
  bridge.page.mockResolvedValue([row('saves.other', 3), row('saves.slots', 4)])
  bridge.game.mockResolvedValue(saves([]))
})

describe('slotCount', () => {
  it('reads a number, as text too', () => {
    expect(slotCount(4)).toBe(4)
    expect(slotCount('5')).toBe(5)
  })

  it('falls back to 10 without a value, with a non-number or with zero', () => {
    expect(DEFAULT_SLOT_COUNT).toBe(10)
    expect(slotCount(undefined)).toBe(10)
    expect(slotCount(null)).toBe(10)
    expect(slotCount('many')).toBe(10)
    expect(slotCount(0)).toBe(10)
  })
})

describe('loadSlotCount', () => {
  it('reads saves.slots at «Tutti i giochi», on the page the schema gives it', async () => {
    expect(await loadSlotCount()).toBe(4)
    expect(bridge.page).toHaveBeenCalledWith('saves', { level: 'global' })
  })

  it('throws what the bridge throws: the caller reports it', async () => {
    bridge.page.mockRejectedValueOnce({ code: 'io' })
    await expect(loadSlotCount()).rejects.toEqual({ code: 'io' })
  })
})

describe('loadGameSaves', () => {
  it('asks for the setting’s slots, or the ones given', async () => {
    await loadGameSaves({ path: '/g.gba' })
    expect(bridge.game).toHaveBeenLastCalledWith('/g.gba', 4)
    await loadGameSaves({ path: '/g.gba' }, 3)
    expect(bridge.game).toHaveBeenLastCalledWith('/g.gba', 3)
  })
})

describe('loadResumePoint', () => {
  it('picks up from the newest quick save', async () => {
    bridge.game.mockResolvedValueOnce(
      saves([
        { slot: 1, savedAt: 10, thumbUrl: 'a' },
        { slot: 2, savedAt: 30, thumbUrl: 'b' },
        { slot: 3, savedAt: null, thumbUrl: null },
      ] as GameSaves['quick']),
    )
    expect(await loadResumePoint({ path: '/g.gba' }, 4)).toEqual({
      slot: 2,
      thumbUrl: 'b',
      none: false,
    })
  })

  it('is null when the saves cannot be read, and never throws', async () => {
    bridge.game.mockRejectedValueOnce({ code: 'io' })
    expect(await loadResumePoint({ path: '/g.gba' }, 4)).toBeNull()
    expect(NOTHING_TO_RESUME.none).toBe(true)
  })
})
