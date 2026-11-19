import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h, nextTick } from 'vue'
import type { SettingsChanged } from '../emu'
import { listenerSet } from './useListeners'
import {
  EVERY_SETTING,
  settingMatches,
  settingsUnder,
  useSettingsChanged,
  type SettingsFilter,
} from './useSettingsChanged'

// The bridge's `settings://changed`: each subscription answers when the test says so, as Tauri
// answers `listen` later; `emit` plays a change to every live callback.
const bridge = vi.hoisted(() => {
  const subscriptions: Array<{
    cb: (e: SettingsChanged) => void
    unlisten: ReturnType<typeof vi.fn>
    resolve: () => void
  }> = []
  return {
    subscriptions,
    onChanged: (cb: (e: SettingsChanged) => void) =>
      new Promise<() => void>(resolve => {
        const unlisten = vi.fn()
        subscriptions.push({ cb, unlisten, resolve: () => resolve(unlisten) })
      }),
    emit: (e: SettingsChanged) => subscriptions.forEach(s => s.cb(e)),
  }
})
vi.mock('../emu', () => ({ emu: { settings: { onChanged: bridge.onChanged } } }))

const change = (key: string, level: SettingsChanged['level'] = 'global'): SettingsChanged => ({
  key,
  level,
})
const flush = () => new Promise(r => setTimeout(r, 0))

function follower(filter: SettingsFilter) {
  const cb = vi.fn()
  const C = defineComponent({
    setup() {
      useSettingsChanged(filter, cb)
      return () => h('div')
    },
  })
  return { cb, wrapper: mount(C) }
}

beforeEach(() => {
  bridge.subscriptions.length = 0
})

describe('settingMatches', () => {
  it('takes one key, a list of keys, or a test on the change', () => {
    expect(settingMatches('pad.ab', change('pad.ab'))).toBe(true)
    expect(settingMatches('pad.ab', change('pad.abc'))).toBe(false)
    expect(settingMatches(['pad.ab', 'pad.remap'], change('pad.remap'))).toBe(true)
    expect(settingMatches(['pad.ab'], change('pad.mod'))).toBe(false)
    expect(settingMatches(settingsUnder('pad.'), change('pad.mod'))).toBe(true)
    expect(settingMatches(settingsUnder('pad.'), change('video.scale'))).toBe(false)
    expect(settingMatches(e => e.level === 'game', change('x', 'game'))).toBe(true)
    expect(settingMatches(EVERY_SETTING, change('anything'))).toBe(true)
  })
})

describe('useSettingsChanged', () => {
  it('calls back on the changes its filter takes, while mounted', async () => {
    const { cb, wrapper } = follower(['saves.slots', 'saves.autoq'])
    await nextTick()
    bridge.subscriptions[0]?.resolve()
    await flush()
    bridge.emit(change('saves.slots'))
    bridge.emit(change('general.theme'))
    expect(cb).toHaveBeenCalledOnce()
    expect(cb).toHaveBeenCalledWith(change('saves.slots'))
    wrapper.unmount()
    expect(bridge.subscriptions[0]?.unlisten).toHaveBeenCalledOnce()
  })

  it('stops a listener that arrives after the component unmounted (the leak)', async () => {
    const { wrapper } = follower('pad.ab')
    await nextTick()
    expect(bridge.subscriptions).toHaveLength(1)
    // Unmounted while the bridge has not answered yet.
    wrapper.unmount()
    const sub = bridge.subscriptions[0]!
    expect(sub.unlisten).not.toHaveBeenCalled()
    sub.resolve()
    await flush()
    expect(sub.unlisten).toHaveBeenCalledOnce()
  })
})

describe('listenerSet', () => {
  it('stops every listener once, and each late one as it arrives', async () => {
    const set = listenerSet()
    const early = vi.fn()
    const sync = vi.fn()
    let lateResolve!: (off: () => void) => void
    const late = vi.fn()
    const added = set.add(
      Promise.resolve(early),
      sync,
      new Promise<() => void>(r => (lateResolve = r)),
    )
    await flush()
    set.stop()
    expect(early).toHaveBeenCalledOnce()
    expect(sync).toHaveBeenCalledOnce()
    lateResolve(late)
    await added
    expect(late).toHaveBeenCalledOnce()
    set.stop()
    expect(early).toHaveBeenCalledOnce()
  })
})
