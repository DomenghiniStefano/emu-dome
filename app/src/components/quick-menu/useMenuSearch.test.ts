import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { LibraryGame, Speed } from '@/emu'
import type { KeyAction } from '@/lib/keyboard'
import { searchHost } from '@/components/search/useSearch'
import { searchKeyLabels } from './hints'
import { steppedSlot } from './slots'
import { toggledSpeed } from './speedChoices'
import { useMenuSearch, type MenuSearchOptions } from './useMenuSearch'
import { QuickMenuView, sessionViewOf } from './view'

vi.mock('@/emu', () => ({ AUTO_SLOT: 0, emu: {} }))
vi.mock('@/nav', () => ({ activeScopeRoot: () => null }))

const x = (value: number): Speed => ({ kind: 'factor', value })
const NORMAL = x(1)

describe('the search’s pure parts in the quick menu', () => {
  it('steps the chosen slot through the numbered slots, wrapping', () => {
    const slots = [0, 1, 2, 3, 4]
    expect(steppedSlot(slots, 2, true)).toBe(3)
    expect(steppedSlot(slots, 4, true)).toBe(1)
    expect(steppedSlot(slots, 1, false)).toBe(4)
    // From the auto slot: the first or the last numbered one.
    expect(steppedSlot(slots, 0, true)).toBe(1)
    expect(steppedSlot(slots, 0, false)).toBe(4)
    expect(steppedSlot([0], 0, true)).toBe(0)
  })

  it('toggles fast forward and slow motion as the shortcuts do', () => {
    expect(toggledSpeed(NORMAL, x(4))).toEqual(x(4))
    expect(toggledSpeed(x(4), x(4))).toEqual(NORMAL)
    expect(toggledSpeed(x(4), x(0.5))).toEqual(x(0.5))
  })

  it('finds the page that shows a setting «in prova»', () => {
    expect(sessionViewOf('speed.ffval')).toBe(QuickMenuView.Speed)
    expect(sessionViewOf('video.scale')).toBe(QuickMenuView.Screens)
    expect(sessionViewOf('touch.topac')).toBe(QuickMenuView.Controls)
    expect(sessionViewOf('audio.vol')).toBeNull()
  })

  it('names the keys that open the search for the input in use only', () => {
    expect(searchKeyLabels('keyboard')).toEqual(['search.key.ctrl', 'search.key.k'])
    expect(searchKeyLabels('controller')).toEqual(['controls.button.y'])
    expect(searchKeyLabels('touch')).toEqual([])
  })
})

function setup() {
  const run = {
    resume: vi.fn(),
    speed: vi.fn(),
    screenshot: vi.fn(),
    fps: vi.fn(),
    swapScreens: vi.fn(),
    nextLayout: vi.fn(),
    toggleLid: vi.fn(),
    fullscreen: vi.fn(),
    openGame: vi.fn(),
  }
  const slots = {
    numbers: () => [0, 1, 2, 3],
    chosen: () => 3,
    pick: vi.fn(),
    save: vi.fn(),
    loadChosen: vi.fn(),
  }
  const openPage = vi.fn()
  const options: MenuSearchOptions = {
    root: ref<HTMLElement | null>(null),
    context: () => ({
      playing: 'gba:01',
      dualScreen: true,
      lidSupported: true,
      loadBlock: null,
      slot: { chosen: 3, empty: false, auto: false },
      settings: ['speed.ffval'],
    }),
    slots,
    speed: () => x(4),
    modes: () => ({ ff: x(4), slow: x(0.5) }),
    run,
    openPage,
  }
  const wrapper = mount(
    defineComponent({
      setup() {
        useMenuSearch(options)
        return () => h('div')
      },
    }),
  )
  const host = searchHost.value
  if (!host) throw new Error('the quick menu did not register as the search’s host')
  return { wrapper, host, run, slots, openPage }
}

let unmount: (() => void) | null = null
afterEach(() => {
  unmount?.()
  unmount = null
})

describe('useMenuSearch', () => {
  it('is the search’s host while mounted, with the open game as its context', () => {
    const { wrapper, host } = setup()
    unmount = () => wrapper.unmount()
    expect(host.context().game?.playing).toBe('gba:01')
  })

  it('runs each shortcut through what the menu already does', () => {
    const { wrapper, host, run, slots } = setup()
    unmount = () => wrapper.unmount()
    const calls: [KeyAction, () => unknown][] = [
      ['menu', () => run.resume],
      ['save', () => slots.save],
      ['load', () => slots.loadChosen],
      ['screenshot', () => run.screenshot],
      ['fps', () => run.fps],
      ['swap', () => run.swapScreens],
      ['layout', () => run.nextLayout],
      ['lid', () => run.toggleLid],
      ['fullscreen', () => run.fullscreen],
    ]
    for (const [id, fn] of calls) {
      host.action(id)
      expect(fn()).toHaveBeenCalledOnce()
    }
    host.action('next_slot')
    host.action('prev_slot')
    expect(slots.pick.mock.calls).toEqual([[1], [2]])
    // Running at fast forward: its toggle goes back to normal, slow motion goes slow.
    host.action('ff_toggle')
    host.action('slow')
    expect(run.speed.mock.calls).toEqual([[NORMAL], [x(0.5)]])
  })

  it('opens the page of a setting it shows, and another game through the menu', () => {
    const { wrapper, host, run, openPage } = setup()
    unmount = () => wrapper.unmount()
    host.setting('speed.ffval')
    expect(openPage).toHaveBeenCalledWith(QuickMenuView.Speed)
    host.setting('audio.vol')
    expect(openPage).toHaveBeenCalledOnce()
    const other = { id: 'nds:02' } as LibraryGame
    host.game(other, false)
    expect(run.openGame).toHaveBeenCalledWith(other)
  })
})
