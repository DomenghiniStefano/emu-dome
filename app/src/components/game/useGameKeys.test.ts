import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import type { Button, GameInfo } from '@/emu'
import { keymap } from '@/lib/keymap'
import { keyRoute, type KeyContext } from './keyRoute'
import { useGameKeys } from './useGameKeys'
import type { GameSession } from './useGameSession'

const bridge = vi.hoisted(() => ({
  input: vi.fn(async () => {}),
  releaseKeys: vi.fn(async () => {}),
  saveState: vi.fn(async () => {}),
  loadState: vi.fn(async () => {}),
  bindings: vi.fn(),
  dispatchNav: vi.fn(),
}))
vi.mock('@/emu', () => ({
  emu: {
    input: bridge.input,
    releaseKeys: bridge.releaseKeys,
    saveState: bridge.saveState,
    loadState: bridge.loadState,
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
  const playing: KeyContext = {
    inGame: true,
    menuOpen: false,
    rewindToggle: false,
    rewinding: false,
  }
  const map = { KeyX: A }
  const route = (code: string, pressed = true, repeat = false, ctx: Partial<KeyContext> = {}) =>
    keyRoute(code, pressed, repeat, { ...playing, ...ctx }, map)

  it('Esc opens the quick menu, and in the menu goes one level up; its release does nothing there', () => {
    expect(route('Escape')).toEqual({ kind: 'openMenu' })
    expect(route('Escape', true, false, { inGame: false })).toEqual({ kind: 'openMenu' })
    expect(route('Escape', true, false, { menuOpen: true })).toEqual({ kind: 'back' })
    expect(route('Escape', false, false, { menuOpen: true })).toEqual({ kind: 'none' })
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

  it('outside the game and in the menu, a pressed key is focus navigation, never a game key', () => {
    expect(route('ArrowUp', true, false, { inGame: false })).toEqual({ kind: 'nav' })
    expect(route('KeyX', true, false, { menuOpen: true })).toEqual({ kind: 'nav' })
    expect(route('F1', true, false, { menuOpen: true })).toEqual({ kind: 'nav' })
    expect(route('ArrowUp', false, false, { inGame: false })).toEqual({ kind: 'none' })
  })

  it('a game key let go in the menu still reaches the game; outside a game it goes nowhere', () => {
    expect(route('KeyX', false, false, { menuOpen: true })).toEqual({
      kind: 'button',
      button: A,
      send: true,
    })
    expect(route('KeyX', false, false, { inGame: false })).toEqual({ kind: 'none' })
    expect(route('KeyQ', false, false, { menuOpen: true })).toEqual({ kind: 'none' })
  })

  it('in game the keyboard table wins over the game’s keys', () => {
    expect(route('F1')).toEqual({ kind: 'shortcut', action: 'save' })
    expect(route('Tab', false)).toEqual({ kind: 'shortcut', action: 'ff_hold' })
    expect(keyRoute('F1', true, false, playing, { F1: A })).toEqual({
      kind: 'shortcut',
      action: 'save',
    })
  })

  it('a game key goes to the core, its repeats do not; an unmapped key goes nowhere', () => {
    expect(route('KeyX')).toEqual({ kind: 'button', button: A, send: true })
    expect(route('KeyX', true, true)).toEqual({ kind: 'button', button: A, send: false })
    expect(route('KeyX', false)).toEqual({ kind: 'button', button: A, send: true })
    expect(route('KeyQ')).toEqual({ kind: 'none' })
  })

  it('Backspace let go in the menu ends a held rewind, not a toggled one', () => {
    const inMenu = { menuOpen: true, rewinding: true }
    expect(route('Backspace', false, false, inMenu)).toEqual({ kind: 'endRewind' })
    expect(route('Backspace', false, false, { ...inMenu, rewindToggle: true })).toEqual({
      kind: 'none',
    })
    expect(route('Backspace', false, false, { menuOpen: true })).toEqual({ kind: 'none' })
  })
})

function setup(opts: { inGame?: boolean; rewindToggle?: boolean } = {}) {
  const session = {
    game: ref<GameInfo | null>(opts.inGame === false ? null : ({ title: 'MINISH' } as GameInfo)),
    menuOpen: ref(false),
    comboSlot: ref(2),
    showFps: ref(false),
    rewinding: ref(false),
    settings: ref({ rewindToggle: opts.rewindToggle ?? false }),
    setRewinding: vi.fn((on: boolean) => (session.rewinding.value = on)),
    setTurbo: vi.fn(),
    stepSlot: vi.fn(),
    screenshot: vi.fn(),
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
  it('loads the keys of «Tutti i giochi» and sends a mapped key to the game, once per press', async () => {
    const { wrapper } = setup()
    await flush()
    expect(bridge.bindings).toHaveBeenCalledWith('keyboard', { level: 'global' })
    const down = press('KeyX')
    press('KeyX', true)
    release('KeyX')
    expect(down.defaultPrevented).toBe(true)
    expect(bridge.input.mock.calls).toEqual([
      ['a', true],
      ['a', false],
    ])
    wrapper.unmount()
  })

  it('F1 / F4 save and load the current slot; F2 / F3 step it; F9 shows the FPS; F12 captures', () => {
    const { session, wrapper } = setup()
    press('F1')
    press('F4')
    press('F2')
    press('F3')
    press('F9')
    press('F12')
    expect(bridge.saveState).toHaveBeenCalledWith(2)
    expect(bridge.loadState).toHaveBeenCalledWith(2)
    expect(session.stepSlot.mock.calls).toEqual([[false], [true]])
    expect(session.showFps.value).toBe(true)
    expect(session.screenshot).toHaveBeenCalledOnce()
    wrapper.unmount()
  })

  it('a shortcut acts on the press only: neither its repeats nor its release act again', () => {
    const { session, wrapper } = setup()
    press('F9')
    press('F9', true)
    release('F9')
    press('F1', true)
    expect(session.showFps.value).toBe(true)
    expect(bridge.saveState).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('Tab holds fast forward while held, ignoring its repeats', () => {
    const { session, wrapper } = setup()
    press('Tab')
    press('Tab', true)
    release('Tab')
    expect(session.setTurbo.mock.calls).toEqual([[true], [false]])
    wrapper.unmount()
  })

  it('Backspace rewinds while held; with `a11y.toggle` each press turns it on or off', () => {
    const held = setup()
    press('Backspace')
    expect(held.session.rewinding.value).toBe(true)
    release('Backspace')
    expect(held.session.rewinding.value).toBe(false)
    held.wrapper.unmount()

    const toggled = setup({ rewindToggle: true })
    press('Backspace')
    release('Backspace')
    expect(toggled.session.rewinding.value).toBe(true)
    press('Backspace')
    expect(toggled.session.rewinding.value).toBe(false)
    toggled.wrapper.unmount()
  })

  it('Backspace let go after the menu opened still ends the rewind it held', () => {
    const { session, wrapper } = setup()
    press('Backspace')
    press('Escape')
    expect(session.setMenu).toHaveBeenCalledWith(true)
    session.menuOpen.value = true
    release('Backspace')
    expect(session.rewinding.value).toBe(false)
    wrapper.unmount()
  })

  it('a key held when the menu opens and let go there is released in the game (#74)', async () => {
    const { session, wrapper } = setup()
    await flush()
    press('KeyX')
    press('Escape')
    session.menuOpen.value = true
    release('KeyX')
    expect(bridge.input.mock.calls).toEqual([
      ['a', true],
      ['a', false],
    ])
    wrapper.unmount()
  })

  it('the menu opening ends the fast forward and the rewind the keys hold', async () => {
    const { session, wrapper } = setup()
    press('Tab')
    press('Backspace')
    session.menuOpen.value = true
    await flush()
    expect(session.setTurbo.mock.calls).toEqual([[true], [false]])
    expect(session.rewinding.value).toBe(false)
    release('Tab')
    expect(session.setTurbo).toHaveBeenCalledTimes(2)
    wrapper.unmount()
  })

  it('the window losing the focus lets go of every key; a toggled rewind stays on', () => {
    const { session, wrapper } = setup({ rewindToggle: true })
    press('Tab')
    press('Backspace')
    window.dispatchEvent(new Event('blur'))
    expect(bridge.releaseKeys).toHaveBeenCalledOnce()
    expect(session.setTurbo.mock.calls).toEqual([[true], [false]])
    expect(session.rewinding.value).toBe(true)
    wrapper.unmount()

    const held = setup()
    press('Backspace')
    window.dispatchEvent(new Event('blur'))
    expect(held.session.rewinding.value).toBe(false)
    held.wrapper.unmount()
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
    expect(bridge.input).not.toHaveBeenCalled()
    expect(bridge.saveState).not.toHaveBeenCalled()
    expect(session.setMenu).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('outside a game the arrows move the focus and the game’s shortcuts do nothing', () => {
    const { session, wrapper } = setup({ inGame: false })
    press('ArrowLeft')
    release('ArrowLeft')
    expect(bridge.dispatchNav.mock.calls).toEqual([['left']])
    press('F9')
    expect(session.showFps.value).toBe(false)
    wrapper.unmount()
  })

  it('F11 toggles full screen on a first press only and never reaches the browser', () => {
    const { session, wrapper } = setup()
    const first = press('F11')
    const again = press('F11', true)
    const up = release('F11')
    expect(session.toggleFullscreen).toHaveBeenCalledOnce()
    expect([first, again, up].every(e => e.defaultPrevented)).toBe(true)
    wrapper.unmount()
  })

  it('stops listening when the component goes', () => {
    const { session, wrapper } = setup()
    wrapper.unmount()
    press('F9')
    press('Escape')
    expect(session.showFps.value).toBe(false)
    expect(session.setMenu).not.toHaveBeenCalled()
  })
})
