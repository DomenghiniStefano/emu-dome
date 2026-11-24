import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { GameInfo } from '@/emu'
import { bottomInset, insetShares } from './inset'
import { useGameSession } from './useGameSession'
import type { Toasts } from '@/composables/useToasts'

const bridge = vi.hoisted(() => ({
  setSlot: vi.fn(),
  setMenuOpen: vi.fn(),
  setPhone: vi.fn(),
  setVideoInset: vi.fn(),
}))
vi.mock('@/emu', () => ({
  AUTO_SLOT: 0,
  emu: {
    setMenuOpen: bridge.setMenuOpen,
    setVideoInset: bridge.setVideoInset,
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
  const moved = [{ id: 'main' as const, x: 0, y: 0.1, w: 1, h: 0.5 }]
  /** The shares Rust gets for the bottom `px` of jsdom's window. */
  const band = (px: number) => insetShares(bottomInset(px), window.innerWidth, window.innerHeight)
  /** What was queued in this tick is sent. */
  const sent = () => Promise.resolve()
  beforeEach(() => {
    bridge.setPhone.mockReset()
    bridge.setPhone.mockResolvedValue({})
    bridge.setVideoInset.mockReset()
    bridge.setVideoInset.mockResolvedValue(undefined)
    bridge.setMenuOpen.mockResolvedValue(undefined)
    vi.spyOn(console, 'error').mockImplementation(() => {})
  })

  it("a late one from the old game sends nothing of the old game's to the new one", async () => {
    const { session } = setup()
    await session.begin(game('gba:01', 1), null)
    session.setPlacement('layer', placed)
    session.setPlacement('editor', placed)
    await sent()
    expect(bridge.setPhone).toHaveBeenCalledTimes(1)
    await session.begin(game('gba:02', 1), null)
    // The old game's editor closing late: the new session holds no placement to fall back to.
    session.setPlacement('editor', null)
    await sent()
    expect(bridge.setPhone).toHaveBeenCalledTimes(1)
  })

  it('a placement and the inset of the same change go in one call', async () => {
    const { session } = setup()
    await session.begin(game('nds:01', 1), null)
    session.setPlacement('editor', placed)
    session.setInset(bottomInset(200))
    await sent()
    expect(bridge.setPhone).toHaveBeenCalledTimes(1)
    expect(bridge.setPhone).toHaveBeenCalledWith(placed, band(200))
    expect(bridge.setVideoInset).not.toHaveBeenCalled()
  })

  it('an inset alone (the quick menu) goes alone, and nothing is sent twice', async () => {
    const { session } = setup()
    await session.begin(game('nds:01', 1), null)
    session.setInset(bottomInset(120))
    await sent()
    session.setInset(bottomInset(120))
    await sent()
    expect(bridge.setVideoInset).toHaveBeenCalledTimes(1)
    expect(bridge.setVideoInset).toHaveBeenCalledWith(band(120))
    expect(bridge.setPhone).not.toHaveBeenCalled()
  })

  it('a refused call is sent again by the next same request', async () => {
    const { session } = setup()
    await session.begin(game('nds:01', 1), null)
    bridge.setPhone.mockRejectedValueOnce({ code: 'settingInvalidValue', params: {} })
    session.setPlacement('editor', placed)
    await sent()
    await sent()
    session.setPlacement('editor', moved)
    session.setPlacement('editor', placed)
    await sent()
    expect(bridge.setPhone).toHaveBeenCalledTimes(2)
    expect(bridge.setPhone).toHaveBeenLastCalledWith(placed, band(0))
  })

  it("a refusal of the old game's call leaves the new game's memo alone", async () => {
    const { session } = setup()
    await session.begin(game('nds:01', 1), null)
    let refuse!: (e: unknown) => void
    bridge.setPhone.mockReturnValueOnce(new Promise((_, r) => (refuse = r)))
    session.setPlacement('layer', placed)
    await sent()
    await session.begin(game('nds:02', 1), null)
    session.setPlacement('layer', moved)
    await sent()
    refuse({ code: 'noGame', params: {} })
    await sent()
    await sent()
    session.setPlacement('layer', moved)
    await sent()
    expect(bridge.setPhone).toHaveBeenCalledTimes(2)
  })
})
