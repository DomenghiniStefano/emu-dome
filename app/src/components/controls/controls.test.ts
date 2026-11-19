import { beforeAll, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import type { ComboRow, LibraryGame, PadInfo, PadState, RemapRow } from '@/emu'
import { i18n } from '@/i18n'
import {
  captureDeviceOf,
  connectionText,
  deviceInUse,
  familyOf,
  isSameDevice,
  menuBackPos,
} from './devices'
import { RemapLevel, lastPlayedConsole, levelScope, remapLevels } from './levels'
import { RemapRowStatus, remapRowStatus, sourceUses } from './remapTable'
import { keyboardRememberLines, padRememberLines } from './shortcutLines'
import { axisUnit, deadZoneDiameter, stickDot } from './stick'
import { ControlsTab, tabOrRemap, tabsFor } from './tabs'
import { buttonsUsing, newlyHeld } from './tester'
import { useButtonTester } from './useButtonTester'

// emu.ts (through i18n) builds its protocol URLs from Tauri's bridge when it loads: a stand-in.
vi.hoisted(() => {
  Object.assign(window, {
    __TAURI_INTERNALS__: {
      convertFileSrc: (path: string, protocol: string) => `${protocol}://${path}`,
    },
  })
})

beforeAll(() => {
  i18n.global.locale.value = 'en'
})

const pad = (id: number, battery: number | null = null): PadInfo => ({
  id,
  name: `Pad ${id}`,
  family: 'xbox',
  vendor: 0,
  product: 0,
  battery,
})
const row = (button: RemapRow['button'], source: string | null, changed = false): RemapRow => ({
  button,
  source,
  default: 'south',
  changed,
})

describe('devices', () => {
  it('uses the chosen controller, else the first, else none', () => {
    const pads = [pad(1), pad(2)]
    expect(deviceInUse(pads, 2)).toEqual(pad(2))
    expect(deviceInUse(pads, null)).toEqual(pad(1))
    expect(deviceInUse(pads, 9)).toEqual(pad(1))
    expect(deviceInUse([], null)).toBeNull()
    expect(deviceInUse([], 'keyboard')).toBe('keyboard')
  })

  it('names the family, and the keyboard for no device', () => {
    expect(familyOf(pad(1))).toBe('xbox')
    expect(familyOf('keyboard')).toBe('keyboard')
    expect(familyOf(null)).toBe('keyboard')
  })

  it('tells the device in use by id, the keyboard by itself', () => {
    expect(isSameDevice(pad(1), pad(1, 50))).toBe(true)
    expect(isSameDevice(pad(1), pad(2))).toBe(false)
    expect(isSameDevice('keyboard', 'keyboard')).toBe(true)
    expect(isSameDevice(pad(1), 'keyboard')).toBe(false)
  })

  it('says the battery only when known', () => {
    expect(connectionText(pad(1))).toBe('Connected')
    expect(connectionText(pad(1, 80))).toBe('Connected · battery 80%')
  })

  it('cancels a capture with the back button of the menus', () => {
    expect(menuBackPos('bottom')).toBe('east')
    expect(menuBackPos('right')).toBe('south')
    expect(menuBackPos(undefined)).toBe('south')
    expect(captureDeviceOf(pad(3), 'east')).toEqual({ kind: 'pad', id: 3, back: 'east' })
    expect(captureDeviceOf('keyboard', 'east')).toEqual({ kind: 'keyboard' })
  })
})

describe('tabs', () => {
  it('drops sticks for the keyboard', () => {
    expect(tabsFor('keyboard')).toEqual([ControlsTab.Remap, ControlsTab.Shortcuts])
    expect(tabsFor('generic')).toHaveLength(3)
  })

  it('falls back to remap for a tab the device has not', () => {
    expect(tabOrRemap(tabsFor('keyboard'), ControlsTab.Sticks)).toBe(ControlsTab.Remap)
    expect(tabOrRemap(tabsFor('xbox'), ControlsTab.Sticks)).toBe(ControlsTab.Sticks)
  })
})

describe('tester', () => {
  it('finds what was just pressed', () => {
    expect(newlyHeld(new Set(['a']), ['a', 'b', 'c'])).toEqual(['b', 'c'])
    expect(newlyHeld(new Set(['a']), [])).toEqual([])
  })

  it('lists the game buttons a source gives', () => {
    const rows = [row('a', 'east'), row('b', 'south'), row('x', 'east')]
    expect(buttonsUsing(rows, 'east')).toEqual(['a', 'x'])
    expect(buttonsUsing(rows, 'north')).toEqual([])
  })
})

describe('remap levels', () => {
  const game = { level: 'game', console: 'gba', game: 'gba:01' } as const

  it('offers the console and the game only with their context', () => {
    expect(remapLevels(null, null)).toEqual([RemapLevel.Global])
    expect(remapLevels('gba', null)).toEqual([RemapLevel.Global, RemapLevel.Console])
    expect(remapLevels('gba', game)).toEqual([
      RemapLevel.Global,
      RemapLevel.Console,
      RemapLevel.Game,
    ])
  })

  it('maps a level to its scope, all games without context', () => {
    expect(levelScope(RemapLevel.Game, 'gba', game)).toBe(game)
    expect(levelScope(RemapLevel.Console, 'nds', null)).toEqual({
      level: 'console',
      console: 'nds',
    })
    expect(levelScope(RemapLevel.Console, null, null)).toEqual({ level: 'global' })
    expect(levelScope(RemapLevel.Game, null, null)).toEqual({ level: 'global' })
  })

  it('takes the console of the last game played', () => {
    const g = (console_: string, lastPlayed: number | null) =>
      ({ console: console_, lastPlayed }) as LibraryGame
    expect(lastPlayedConsole([g('gba', 1), g('nds', 5), g('gb', null)])).toBe('nds')
    expect(lastPlayedConsole([g('gb', null)])).toBeNull()
  })
})

describe('remap table', () => {
  it('flags a source used twice, unless the row listens', () => {
    const rows = [row('a', 'east', true), row('b', 'south'), row('x', 'south'), row('y', null)]
    const uses = sourceUses(rows)
    expect(uses.get('south')).toBe(2)
    expect(remapRowStatus(rows[1]!, false, uses)).toBe(RemapRowStatus.UsedTwice)
    expect(remapRowStatus(rows[1]!, true, uses)).toBe(RemapRowStatus.None)
    expect(remapRowStatus(rows[0]!, false, uses)).toBe(RemapRowStatus.Changed)
    expect(remapRowStatus(rows[3]!, false, uses)).toBe(RemapRowStatus.None)
  })
})

describe('shortcuts to remember', () => {
  const combo = (
    hotkey: ComboRow['hotkey'],
    pos: ComboRow['pos'],
    toRemember = true,
  ): ComboRow => ({ hotkey, pos, default: pos, status: 'active', toRemember })

  it('opens the menu with Home and pairs the slots on one line', () => {
    const lines = padRememberLines('xbox', 'l3', [
      combo('menu', null),
      combo('save', 'r1'),
      combo('prev_slot', 'left'),
      combo('next_slot', 'right'),
      combo('fps', 'north', false),
      combo('load', null),
    ])
    expect(lines.map(l => l.key)).toEqual(['menu', 'save', 'slot'])
    expect(lines[1]!.chip).toBe('L3 + RB')
  })

  it('lists every key of the keyboard table', () => {
    const lines = keyboardRememberLines()
    expect(lines.find(l => l.key === 'save')?.chip).toBe('F1')
    expect(lines.find(l => l.key === 'slot')?.chip).toBe('F2 / F3')
  })
})

describe('live stick', () => {
  it('centres the dot at rest and reaches the edge at full tilt', () => {
    expect(stickDot(0, 0, 120, 12)).toEqual({ left: 54, top: 54 })
    expect(stickDot(1, -1, 120, 12)).toEqual({ left: 108, top: 0 })
    expect(axisUnit(32767)).toBe(1)
    expect(axisUnit(undefined)).toBe(0)
    expect(deadZoneDiameter(120, 15)).toBe(18)
  })
})

describe('useButtonTester', () => {
  function harness(padId: number | null, paused = () => false) {
    let api: ReturnType<typeof useButtonTester> | undefined
    const wrapper = mount(
      defineComponent({
        setup() {
          api = useButtonTester({ padId: () => padId, describe: s => `pressed ${s}`, paused })
          return () => h('div')
        },
      }),
    )
    if (!api) throw new Error('not set up')
    return { api, wrapper }
  }
  const state = (held: string[]) => [{ id: 1, held, axes: [0, 0, 0, 0, 0, 0] }] as PadState[]

  it('lights what a controller holds and describes the newest press', () => {
    const { api, wrapper } = harness(1)
    api.onPads(state(['south']))
    api.onPads(state(['south', 'r1']))
    expect([...api.held.value]).toEqual(['south', 'r1'])
    expect(api.last.value).toBe('pressed r1')
    api.reset()
    expect(api.held.value.size).toBe(0)
    expect(api.last.value).toBe('')
    wrapper.unmount()
  })

  it('keeps quiet while paused, and reads the keyboard only without a controller', () => {
    let paused = true
    const pads = harness(1, () => paused)
    pads.api.onPads(state(['east']))
    expect(pads.api.last.value).toBe('')
    paused = false
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyX' }))
    expect(pads.api.held.value.has('KeyX')).toBe(false)
    pads.wrapper.unmount()

    const keys = harness(null)
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyX' }))
    expect(keys.api.held.value.has('KeyX')).toBe(true)
    expect(keys.api.last.value).toBe('pressed KeyX')
    window.dispatchEvent(new KeyboardEvent('keyup', { code: 'KeyX' }))
    expect(keys.api.held.value.size).toBe(0)
    keys.wrapper.unmount()
  })
})
