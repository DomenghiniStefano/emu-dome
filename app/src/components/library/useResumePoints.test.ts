import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref, type Ref } from 'vue'
import type { GameId, LibraryGame } from '@/emu'
import type { ResumePoint } from '@/lib/gameDetails'
import { useResumePoints } from './useResumePoints'

const saves = vi.hoisted(() => ({
  slotCount: vi.fn<() => Promise<number>>(),
  point: vi.fn<(g: { id: string }, slots: number) => Promise<unknown>>(),
}))
vi.mock('@/components/details/gameSaves', () => ({
  DEFAULT_SLOT_COUNT: 10,
  NOTHING_TO_RESUME: { slot: null, thumbUrl: null, none: true },
  loadSlotCount: saves.slotCount,
  loadResumePoint: saves.point,
}))

const flush = () => new Promise(r => setTimeout(r, 0))
const game = (id: string, lastPlayed = 1) => ({ id: id as GameId, lastPlayed }) as LibraryGame
const point = (slot: number): ResumePoint => ({ slot, thumbUrl: null, none: false })

function setup(recent: Ref<LibraryGame[]>) {
  const onError = vi.fn()
  let resume!: Ref<Record<string, ResumePoint>>
  mount(
    defineComponent({
      setup() {
        resume = useResumePoints(recent, onError)
        return () => h('div')
      },
    }),
  )
  return { resume, onError }
}

beforeEach(() => {
  vi.clearAllMocks()
  saves.slotCount.mockResolvedValue(4)
})

describe('useResumePoints', () => {
  it('reads each recent game at the slot count of the setting', async () => {
    saves.point.mockImplementation(async g => (g.id === 'a' ? point(2) : point(3)))
    const { resume } = setup(ref([game('a'), game('b')]))
    await flush()
    expect(saves.point.mock.calls.map(c => c[1])).toEqual([4, 4])
    expect(resume.value).toEqual({ a: point(2), b: point(3) })
  })

  it('an unreadable game says «Gioca»; a failed slot count is reported and the default used', async () => {
    saves.slotCount.mockRejectedValue({ code: 'io' })
    saves.point.mockResolvedValue(null)
    const { resume, onError } = setup(ref([game('a')]))
    await flush()
    expect(onError).toHaveBeenCalledWith({ code: 'io' })
    expect(saves.point).toHaveBeenCalledWith(expect.objectContaining({ id: 'a' }), 10)
    expect(resume.value.a?.none).toBe(true)
  })

  it('reads again when a game is played, and drops an answer overtaken by a newer read', async () => {
    const answers: Array<(p: ResumePoint) => void> = []
    saves.point.mockImplementation(() => new Promise(r => answers.push(r)))
    const recent = ref([game('a', 1)])
    const { resume } = setup(recent)
    await flush()
    recent.value = [game('a', 2)]
    await flush()
    expect(answers).toHaveLength(2)
    // The newer read answers first; the older one, answering late, is dropped.
    answers[1]?.(point(5))
    await flush()
    answers[0]?.(point(1))
    await flush()
    expect(resume.value.a).toEqual(point(5))
  })

  it('reads nothing without recent games', async () => {
    setup(ref([]))
    await flush()
    expect(saves.slotCount).not.toHaveBeenCalled()
  })
})
