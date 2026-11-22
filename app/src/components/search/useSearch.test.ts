import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, ref } from 'vue'
import { SHELL } from './searchContext'
import { searchHost, searchState, useSearchHost, type SearchHost } from './useSearch'

// Ctrl + K asks nav.ts which scope has the steps.
const nav = vi.hoisted(() => ({ root: null as HTMLElement | null }))
vi.mock('@/nav', () => ({ activeScopeRoot: () => nav.root }))

function hostOf(): SearchHost {
  return {
    root: ref<HTMLElement | null>(null),
    context: () => SHELL,
    action: vi.fn(),
    setting: vi.fn(),
    game: vi.fn(),
  }
}

function mountHost(host: SearchHost) {
  let open!: () => void
  const wrapper = mount(
    defineComponent({
      setup() {
        open = useSearchHost(host).openSearch
        return () => h('div')
      },
    }),
  )
  return { wrapper, open }
}

const ctrlK = () =>
  window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyK', ctrlKey: true }))

afterEach(() => {
  searchState.open = false
  nav.root = null
})

describe('useSearchHost', () => {
  it('makes the newest host the one in use, and the older one again when it goes', () => {
    const shell = hostOf()
    const menu = hostOf()
    const a = mountHost(shell)
    const b = mountHost(menu)
    expect(searchHost.value).toBe(menu)
    b.wrapper.unmount()
    expect(searchHost.value).toBe(shell)
    a.wrapper.unmount()
    expect(searchHost.value).toBeNull()
  })

  it('opens only for the host in use', () => {
    const a = mountHost(hostOf())
    const b = mountHost(hostOf())
    a.open()
    expect(searchState.open).toBe(false)
    b.open()
    expect(searchState.open).toBe(true)
    b.wrapper.unmount()
    a.wrapper.unmount()
  })

  it('opens with Ctrl + K only while the host’s scope has the steps', () => {
    const host = hostOf()
    const el = document.createElement('div')
    host.root.value = el
    const { wrapper } = mountHost(host)
    ctrlK()
    expect(searchState.open).toBe(false)
    nav.root = el
    ctrlK()
    expect(searchState.open).toBe(true)
    wrapper.unmount()
  })

  it('closes the search when the host in use goes (the game closed under the menu)', () => {
    const { wrapper, open } = mountHost(hostOf())
    open()
    expect(searchState.open).toBe(true)
    wrapper.unmount()
    expect(searchState.open).toBe(false)
  })
})
