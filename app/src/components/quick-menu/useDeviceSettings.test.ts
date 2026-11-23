import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import type { SettingEntry, SettingRow, SettingsChanged, SettingsSchema } from '@/emu'
import { deviceEntries, useDeviceSettings } from './useDeviceSettings'
import { QuickMenuView } from './view'

const bridge = vi.hoisted(() => ({
  page: vi.fn(),
  set: vi.fn(),
  changed: [] as Array<(e: SettingsChanged) => void>,
}))
vi.mock('@/emu', () => ({
  emu: {
    settings: {
      page: bridge.page,
      set: bridge.set,
      onChanged: (cb: (e: SettingsChanged) => void) => {
        bridge.changed.push(cb)
        return Promise.resolve(() => {})
      },
    },
  },
}))

const vib = (over: Partial<SettingEntry> = {}) =>
  ({ key: 'touch.tvib', page: 'touch', phase: 1, platform: 'phone', ...over }) as SettingEntry
const schema = (device: 'phone' | 'desktop', entries: SettingEntry[]) =>
  ({ currentPhase: 1, device, entries }) as unknown as SettingsSchema
const row = (key: string, value: SettingRow['value']) =>
  ({ key, value, source: 'global', changed: false }) as SettingRow
const flush = () => new Promise(r => setTimeout(r, 0))

function setup() {
  const onError = vi.fn()
  let device!: ReturnType<typeof useDeviceSettings>
  mount(
    defineComponent({
      setup() {
        device = useDeviceSettings(onError)
        return () => h('div')
      },
    }),
  )
  return { device, onError }
}

describe('deviceEntries', () => {
  it('lists «Vibrazione» in Controlli on a phone only', () => {
    expect(deviceEntries(QuickMenuView.Controls, schema('phone', [vib()])).map(e => e.key)).toEqual(
      ['touch.tvib'],
    )
    expect(deviceEntries(QuickMenuView.Controls, schema('desktop', [vib()]))).toEqual([])
    expect(deviceEntries(QuickMenuView.Speed, schema('phone', [vib()]))).toEqual([])
    expect(deviceEntries(QuickMenuView.Controls, null)).toEqual([])
  })

  it('leaves out a row of a later phase', () => {
    expect(deviceEntries(QuickMenuView.Controls, schema('phone', [vib({ phase: 4 })]))).toEqual([])
  })
})

describe('useDeviceSettings', () => {
  beforeEach(() => {
    bridge.page.mockReset()
    bridge.set.mockReset()
    bridge.changed.length = 0
  })

  it('reads and sets the device rows at «Tutti i giochi», never «in prova»', async () => {
    bridge.page.mockResolvedValue([row('touch.tvib', false), row('touch.topac', 50)])
    const { device } = setup()
    await device.load()
    expect(bridge.page).toHaveBeenCalledWith('touch', { level: 'global' })
    expect(Object.keys(device.rows.value)).toEqual(['touch.tvib'])
    expect(device.rows.value['touch.tvib']?.value).toBe(false)
    await device.set('touch.tvib', true)
    expect(bridge.set).toHaveBeenCalledWith('touch.tvib', { level: 'global' }, true)
  })

  it('drops an older read that answers after a newer one', async () => {
    let first!: (rows: SettingRow[]) => void
    bridge.page
      .mockImplementationOnce(() => new Promise(r => (first = r)))
      .mockResolvedValueOnce([row('touch.tvib', true)])
    const { device } = setup()
    const older = device.load()
    await device.load()
    first([row('touch.tvib', false)])
    await older
    expect(device.rows.value['touch.tvib']?.value).toBe(true)
  })

  it('reads again when the vibration changes elsewhere, not for other settings', async () => {
    bridge.page.mockResolvedValue([row('touch.tvib', true)])
    setup()
    await flush()
    const calls = bridge.page.mock.calls.length
    bridge.changed.forEach(cb => cb({ key: 'video.filter', level: 'global' } as SettingsChanged))
    expect(bridge.page.mock.calls.length).toBe(calls)
    bridge.changed.forEach(cb => cb({ key: 'touch.tvib', level: 'global' } as SettingsChanged))
    expect(bridge.page.mock.calls.length).toBe(calls + 1)
  })

  it('reports a failed read or write', async () => {
    bridge.page.mockRejectedValue(new Error('read'))
    bridge.set.mockRejectedValue(new Error('write'))
    const { device, onError } = setup()
    await device.load()
    await device.set('touch.tvib', false)
    expect(onError).toHaveBeenCalledTimes(2)
  })
})
