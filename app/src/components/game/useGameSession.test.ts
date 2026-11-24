import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { GameInfo } from '@/emu'
import { useGameSession } from './useGameSession'
import type { Toasts } from '@/composables/useToasts'

const bridge = vi.hoisted(() => ({
  setSlot: vi.fn(),
  setMenuOpen: vi.fn(),
  setPhone: vi.fn(),
}))
vi.mock('@/emu', () => ({
  AUTO_SLOT: 0,
  emu: {
    setMenuOpen: bridge.setMenuOpen,
    controls: { setSlot: bridge.setSlot },
    screens: { setPhone: bridge.setPhone },
  },
}))
vi.mock('@/lib/keymap', () => ({ loadKeymap: () => Promise.resolve() }))
// The parts this file only puts together.
vi.mock('./useGameSettings', () => ({
  useGameSettings: () => ({
    settings: ref({}),
    load: () => Promise.resolve(),
    concerns: ref(null),
  }),
}))
vi.mock('./useGameEvents', () => ({ useGameEvents: () => ({ reset: () => {} }) }))
vi.mock('./useGameEnding', () => ({ useGameEnding: () => ({}) }))

const game = (id: string, quickSlot: number) => ({ id, title: id, quickSlot }) as GameInfo

function setup() {
  const toasts = { notify: vi.fn(), fail: vi.fn(), failWith: vi.fn() } as unknown as Toasts & {
    failWith: ReturnType<typeof vi.fn>
  }
  return { session: useGameSession(toasts), toasts }
}

describe("the game's quick slot (#110)", () => {
  beforeEach(() => {
    bridge.setSlot.mockReset()
    bridge.setMenuOpen.mockResolvedValue(undefined)
  })

  it('is the one the game opens with', async () => {
    const { session } = setup()
    await session.begin(game('gba:01', 6), null)
    expect(session.comboSlot.value).toBe(6)
    await session.begin(game('gba:02', 1), null)
    expect(session.comboSlot.value).toBe(1)
  })

  it('«Rendi rapido» sets it at once, as Rust answers it', async () => {
    const { session } = setup()
    await session.begin(game('gba:01', 1), null)
    bridge.setSlot.mockResolvedValue(4)
    await session.setQuickSlot(4)
    expect(bridge.setSlot).toHaveBeenCalledTimes(1)
    expect(bridge.setSlot).toHaveBeenCalledWith(4)
    expect(session.comboSlot.value).toBe(4)
  })

  it('an answer for a game no longer open is dropped, a failure is said', async () => {
    const { session, toasts } = setup()
    await session.begin(game('gba:01', 1), null)
    let answer!: (slot: number) => void
    bridge.setSlot.mockReturnValue(new Promise<number>(r => (answer = r)))
    const pending = session.setQuickSlot(5)
    await session.begin(game('gba:02', 3), null)
    answer(5)
    await pending
    expect(session.comboSlot.value).toBe(3)

    bridge.setSlot.mockRejectedValue({ code: 'noGame', params: {} })
    await session.setQuickSlot(2)
    expect(toasts.failWith).toHaveBeenCalledTimes(1)
    expect(session.comboSlot.value).toBe(3)
  })
})

describe("the phone's placement of the screens (#134)", () => {
  const placed = [{ id: 'main' as const, x: 0, y: 0, w: 1, h: 0.5 }]
  beforeEach(() => {
    bridge.setPhone.mockReset()
    bridge.setPhone.mockResolvedValue({})
    bridge.setMenuOpen.mockResolvedValue(undefined)
  })

  it("a late one from the old game sends nothing of the old game's to the new one", async () => {
    const { session } = setup()
    await session.begin(game('gba:01', 1), null)
    session.setPlacement('layer', placed)
    session.setPlacement('editor', placed)
    expect(bridge.setPhone).toHaveBeenCalledTimes(1)
    await session.begin(game('gba:02', 1), null)
    // The old game's editor closing late: the new session holds no placement to fall back to.
    session.setPlacement('editor', null)
    expect(bridge.setPhone).toHaveBeenCalledTimes(1)
  })
})
