import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { Button, GameInfo } from '@/emu'
import { keymap } from '@/lib/keymap'
import { keyRoute, type KeyContext } from './keyRoute'
import { useGameKeys } from './useGameKeys'
import type { GameSession } from './useGameSession'

const bridge = vi.hoisted(() => ({
  key: vi.fn(async () => {}),
  releaseKeys: vi.fn(async () => {}),
  bindings: vi.fn(),
  dispatchNav: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    key: bridge.key,
    releaseKeys: bridge.releaseKeys,
    controls: { bindings: bridge.bindings },
  },
}))
vi.mock('@/nav', async importOriginal => ({
  ...(await importOriginal<typeof import('@/nav')>()),
  dispatchNav: bridge.dispatchNav,
}))

const flush = () => new Promise(r => setTimeout(r, 0))
const A = 'a' as Button

describe('keyRoute', () => {
  const playing: KeyContext = { inGame: true, menuOpen: false }
  const map = { KeyX: A }
  const route = (code: string, pressed = true, repeat = false, ctx: Partial<KeyContext> = {}) =>
    keyRoute(code, pressed, repeat, { ...playing, ...ctx }, map)

  it('Esc opens the quick menu, and in the menu goes one level up; its release does nothing', () => {
    expect(route('Escape')).toEqual({ kind: 'openMenu' })
    expect(route('Escape', true, false, { inGame: false })).toEqual({ kind: 'openMenu' })
    expect(route('Escape', true, false, { menuOpen: true })).toEqual({ kind: 'back' })
    expect(route('Escape', false, false, { menuOpen: true })).toEqual({ kind: 'none' })
    expect(route('Escape', false)).toEqual({ kind: 'none' })
  })

  it('F11 is always taken; it toggles on a first press only', () => {
    expect(route('F11')).toEqual({ kind: 'fullscreen', toggle: true })
    expect(route('F11', true, true)).toEqual({ kind: 'fullscreen', toggle: false })
    expect(route('F11', false)).toEqual({ kind: 'fullscreen', toggle: false })
    expect(route('F11', true, false, { menuOpen: true })).toEqual({
      kind: 'fullscreen',
      toggle: true,
    })
  })

  it('outside the game and in the menu, a pressed key is focus navigation, never Rust’s', () => {
    expect(route('ArrowUp', true, false, { inGame: false })).toEqual({ kind: 'nav' })
    expect(route('KeyX', true, false, { menuOpen: true })).toEqual({ kind: 'nav' })
    expect(route('F1', true, false, { menuOpen: true })).toEqual({ kind: 'nav' })
    expect(route('ArrowUp', false, false, { inGame: false })).toEqual({ kind: 'none' })
    expect(route('KeyX', false, false, { inGame: false })).toEqual({ kind: 'none' })
  })

  it('a key Rust takes, let go in the menu, still reaches it (#74); others do not', () => {
    expect(route('KeyX', false, false, { menuOpen: true })).toEqual({
      kind: 'rust',
      prevent: false,
    })
    expect(route('Backspace', false, false, { menuOpen: true })).toEqual({
      kind: 'rust',
      prevent: false,
    })
    expect(route('KeyQ', false, false, { menuOpen: true })).toEqual({ kind: 'none' })
  })

  it('in game the shortcuts and the game’s keys go to Rust with their repeats; others nowhere', () => {
    expect(route('F1')).toEqual({ kind: 'rust', prevent: true })
    expect(route('Tab', false)).toEqual({ kind: 'rust', prevent: true })
    expect(route('KeyX')).toEqual({ kind: 'rust', prevent: true })
    expect(route('KeyX', true, true)).toEqual({ kind: 'rust', prevent: true })
    expect(route('KeyQ')).toEqual({ kind: 'none' })
  })
})

function setup(opts: { inGame?: boolean } = {}) {
  const session = {
    game: ref<GameInfo | null>(opts.inGame === false ? null : ({ title: 'MINISH' } as GameInfo)),
    menuOpen: ref(false),
    setMenu: vi.fn(async () => {}),
    toggleFullscreen: vi.fn(async () => {}),
  }
  const wrapper = mount(
    defineComponent({
      setup() {
        useGameKeys(session as unknown as GameSession).listen()
        return () => h('div')
      },
    }),
  )
  return { session, wrapper }
}

function key(type: 'keydown' | 'keyup', code: string, repeat = false) {
  const e = new KeyboardEvent(type, { code, repeat, cancelable: true })
  window.dispatchEvent(e)
  return e
}
const press = (code: string, repeat = false) => key('keydown', code, repeat)
const release = (code: string) => key('keyup', code)

beforeEach(() => {
  vi.clearAllMocks()
  keymap.value = {}
  bridge.bindings.mockResolvedValue({ remap: [{ button: 'a', source: 'KeyX' }] })
})

describe('useGameKeys', () => {
  it('loads the keys of «Tutti i giochi» and sends a game key raw to Rust, repeats included', async () => {
    const { wrapper } = setup()
    await flush()
    expect(bridge.bindings).toHaveBeenCalledWith('keyboard', { level: 'global' })
    const down = press('KeyX')
    press('KeyX', true)
    release('KeyX')
    expect(down.defaultPrevented).toBe(true)
    expect(bridge.key.mock.calls).toEqual([
      ['KeyX', true, false],
      ['KeyX', true, true],
      ['KeyX', false, false],
    ])
    wrapper.unmount()
  })

  it('the keyboard’s shortcuts go to Rust, which runs them; the browser’s Tab does nothing', () => {
    const { wrapper } = setup()
    const tab = press('Tab')
    release('Tab')
    press('F1')
    press('Backspace')
    expect(tab.defaultPrevented).toBe(true)
    expect(bridge.key.mock.calls).toEqual([
      ['Tab', true, false],
      ['Tab', false, false],
      ['F1', true, false],
      ['Backspace', true, false],
    ])
    wrapper.unmount()
  })

  it('a key held when the menu opens and let go there still reaches Rust (#74)', async () => {
    const { session, wrapper } = setup()
    await flush()
    press('Backspace')
    press('Escape')
    expect(session.setMenu).toHaveBeenCalledWith(true)
    session.menuOpen.value = true
    release('Backspace')
    expect(bridge.key.mock.calls).toEqual([
      ['Backspace', true, false],
      ['Backspace', false, false],
    ])
    wrapper.unmount()
  })

  it('the window losing the focus has Rust let go of every key, while a game is open', () => {
    const { wrapper } = setup()
    window.dispatchEvent(new Event('blur'))
    expect(bridge.releaseKeys).toHaveBeenCalledOnce()
    wrapper.unmount()

    const shell = setup({ inGame: false })
    window.dispatchEvent(new Event('blur'))
    expect(bridge.releaseKeys).toHaveBeenCalledOnce()
    shell.wrapper.unmount()
  })

  it('in the quick menu the game gets nothing: Esc is «back», arrows and Enter move the focus', () => {
    const { session, wrapper } = setup()
    session.menuOpen.value = true
    press('Escape')
    const arrow = press('ArrowDown')
    press('Enter')
    press('KeyX')
    press('F1')
    expect(bridge.dispatchNav.mock.calls).toEqual([['back'], ['down'], ['confirm']])
    expect(arrow.defaultPrevented).toBe(true)
    expect(bridge.key).not.toHaveBeenCalled()
    expect(session.setMenu).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('outside a game the arrows move the focus and the game’s shortcuts go nowhere', () => {
    const { wrapper } = setup({ inGame: false })
    press('ArrowLeft')
    release('ArrowLeft')
    expect(bridge.dispatchNav.mock.calls).toEqual([['left']])
    press('F9')
    expect(bridge.key).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('F11 toggles full screen on a first press only and never reaches the browser', () => {
    const { session, wrapper } = setup()
    const first = press('F11')
    const again = press('F11', true)
    const up = release('F11')
    expect(session.toggleFullscreen).toHaveBeenCalledOnce()
    expect([first, again, up].every(e => e.defaultPrevented)).toBe(true)
    expect(bridge.key).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('stops listening when the component goes', () => {
    const { session, wrapper } = setup()
    wrapper.unmount()
    press('F9')
    press('Escape')
    expect(bridge.key).not.toHaveBeenCalled()
    expect(session.setMenu).not.toHaveBeenCalled()
  })
})
