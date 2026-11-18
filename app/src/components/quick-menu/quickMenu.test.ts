import { describe, expect, it, vi } from 'vitest'
import type { SettingEntry, SettingRow } from '@/emu'
import { backKeyLabel } from './hints'
import { settingCard, sourceLine, stepper, videoPathCard } from './screenCards'
import {
  isEmptySlot,
  loadLabel,
  loadedLine,
  mainSlotCount,
  slotCount,
  slotDate,
  slotName,
} from './slots'
import { chosenSpeed, followedSpeed, modeSpeeds, speedChoices } from './speedChoices'
import { ToolId, quickTools, toolsNotYet } from './tools'
import { trialNames, trialTitle } from './trial'
import { QuickMenuView, isSubPage } from './view'

// The bridge reads Tauri's internals when it loads; these pure functions need only its constant.
vi.mock('@/emu', () => ({ AUTO_SLOT: 0, emu: {} }))

const x = (value: number) => ({ kind: 'factor', value }) as const
const MAX = { kind: 'max' } as const

const entry = (key: string, options: string[] | null, hidden: string[] = []): SettingEntry =>
  ({
    key,
    page: key.split('.')[0],
    kind: options
      ? { type: 'choice', style: 'segments', options, moreAtRuntime: false }
      : { type: 'toggle' },
    default: options ? options[0] : true,
    levels: ['default', 'global', 'console', 'game', 'session'],
    perDevice: false,
    platform: 'all',
    phase: 0,
    hiddenOptions: hidden,
    synonyms: [],
    mostUsed: false,
  }) as unknown as SettingEntry

const row = (key: string, value: unknown, source: SettingRow['source']): SettingRow =>
  ({ key, value, source, changed: source !== 'default' }) as SettingRow

describe('views', () => {
  it('has a title under the back button on every page but the main screen and Riavvia', () => {
    expect(isSubPage(QuickMenuView.Home)).toBe(false)
    expect(isSubPage(QuickMenuView.Restart)).toBe(false)
    expect(isSubPage(QuickMenuView.Speed)).toBe(true)
  })
})

describe('hints', () => {
  it('shows only the key of the input in use', () => {
    expect(backKeyLabel('keyboard')).toBe('flow.hintEsc')
    expect(backKeyLabel('controller')).toBe('controls.button.b')
    expect(backKeyLabel('touch')).toBeNull()
  })
})

describe('quick saves', () => {
  it('asks for the slots of saves.slots, ten when it is not a number', () => {
    expect(slotCount('5')).toBe(5)
    expect(slotCount(null)).toBe(10)
    expect(slotCount('many')).toBe(10)
  })
  it('shows Automatico and four slots on desktop, two on the phone', () => {
    expect(mainSlotCount(false)).toBe(5)
    expect(mainSlotCount(true)).toBe(3)
  })
  it('names the auto slot and the numbered ones', () => {
    expect(slotName(0)).toEqual({ key: 'qm.auto' })
    expect(slotName(3)).toEqual({ key: 'qm.slot', params: { slot: 3 } })
  })
  it('labels Carica by the chosen slot and whether it is empty', () => {
    expect(loadLabel(0, true)).toEqual({ key: 'qm.loadAuto' })
    expect(loadLabel(2, true)).toEqual({ key: 'qm.emptySlot', params: { slot: 2 } })
    expect(loadLabel(2, false)).toEqual({ key: 'qm.loadSlot', params: { slot: 2 } })
    expect(loadedLine(0)).toEqual({ key: 'toast.autoLoaded' })
    expect(loadedLine(4)).toEqual({ key: 'toast.stateLoaded', params: { slot: 4 } })
  })
  it('tells an empty slot from a saved one, and dates the saved one', () => {
    const empty = { slot: 1, savedAt: null, thumbUrl: null }
    const saved = { slot: 1, savedAt: Date.UTC(2026, 10, 18, 9, 30), thumbUrl: null }
    expect(isEmptySlot(undefined)).toBe(true)
    expect(isEmptySlot(empty)).toBe(true)
    expect(isEmptySlot(saved)).toBe(false)
    expect(slotDate(empty, 'en')).toBeNull()
    expect(slotDate(saved, 'en')).toMatch(/11\/18\/26/)
  })
})

describe('speeds', () => {
  it('reads the two modes from their settings, with the defaults', () => {
    expect(modeSpeeds('2', 'max')).toEqual({ ff: x(2), slow: MAX })
    expect(modeSpeeds(null, '')).toEqual({ ff: x(4), slow: x(0.5) })
  })
  it('finds the choice the game runs at, none for a speed set elsewhere', () => {
    const choices = speedChoices(x(4), x(0.5))
    expect(choices.map(c => c.id)).toEqual(['normal', 'ff', 'slow'])
    expect(chosenSpeed(choices, x(1))).toBe('normal')
    expect(chosenSpeed(choices, x(0.5))).toBe('slow')
    expect(chosenSpeed(choices, x(3))).toBe('')
  })
  it('moves the game to a mode its new setting changed, only when that mode is in use', () => {
    const before = { ff: x(4), slow: x(0.5) }
    expect(followedSpeed(x(4), { ff: x(2), slow: x(0.5) }, before)).toEqual(x(2))
    expect(followedSpeed(x(0.5), { ff: x(4), slow: x(0.25) }, before)).toEqual(x(0.25))
    expect(followedSpeed(x(1), { ff: x(2), slow: x(0.25) }, before)).toBeNull()
    expect(followedSpeed(x(4), before, before)).toBeNull()
  })
})

describe('tools', () => {
  const base = { phone: false, dualScreen: false, muted: false, fullscreen: true }
  it('gives every game screenshot, mute, full screen and restart on desktop', () => {
    expect(quickTools(base).map(t => t.id)).toEqual([
      ToolId.Screenshot,
      ToolId.Mute,
      ToolId.Fullscreen,
      ToolId.Restart,
    ])
    expect(quickTools(base).find(t => t.id === ToolId.Fullscreen)?.pressed).toBe(true)
  })
  it('has no full screen on the phone', () => {
    expect(quickTools({ ...base, phone: true }).map(t => t.id)).not.toContain(ToolId.Fullscreen)
  })
  it('adds the two dual-screen tools, not available yet, for NDS and 3DS', () => {
    const tools = quickTools({ ...base, dualScreen: true })
    expect(tools).toHaveLength(6)
    expect(toolsNotYet(tools)).toEqual(['qm.swapScreens', 'qm.closeLid'])
    expect(toolsNotYet(quickTools(base))).toEqual([])
  })
})

describe('changes «in prova»', () => {
  const entries = [entry('video.filter', ['none', 'light', 'soft']), entry('saves.undo', null)]
  const find = (key: string) => entries.find(e => e.key === key)
  it('names a choice by its value and anything else by the setting', () => {
    const names = trialNames(
      [
        row('video.filter', 'soft', 'session'),
        row('saves.undo', false, 'session'),
        row('video.scale', 'fit', 'global'),
      ],
      find,
    )
    expect(names).toEqual(['settings.video.filter.option.soft', 'settings.saves.undo.name'])
  })
  it('titles one change and several', () => {
    expect(trialTitle(['A'])).toEqual({ key: 'qm.trialOne', params: { names: 'A' } })
    expect(trialTitle(['A', 'B'])).toEqual({ key: 'qm.trial', params: { n: 2, names: 'A, B' } })
  })
})

describe('G09 cards', () => {
  it('steps through the options and stops at the ends', () => {
    const set = vi.fn()
    const middle = stepper(['a', 'b', 'c'], 'b', set)
    middle.prev?.()
    middle.next?.()
    expect(set.mock.calls).toEqual([['a'], ['c']])
    expect(stepper(['a', 'b'], 'a', set).prev).toBeNull()
    expect(stepper(['a', 'b'], 'b', set).next).toBeNull()
    expect(stepper(['a', 'b'], 'z', set)).toEqual({ prev: null, next: null })
  })
  it('says where a value comes from', () => {
    expect(sourceLine(row('k', 1, 'session'), 'GBA')).toEqual({ key: 'qm.source.session' })
    expect(sourceLine(row('k', 1, 'game'), 'GBA')).toEqual({ key: 'qm.source.game' })
    expect(sourceLine(row('k', 1, 'console'), 'GBA')).toEqual({
      key: 'settingsUi.sameAs',
      level: { key: 'settingsUi.scope.console', params: { console: 'GBA' } },
    })
    expect(sourceLine(undefined, 'GBA')).toEqual({
      key: 'settingsUi.sameAs',
      level: { key: 'settingsUi.scope.global' },
    })
  })
  it('builds a setting card without hidden options, and none to step when locked', () => {
    const set = vi.fn()
    const e = entry('video.scale', ['fit', 'integer', 'fill'], ['integer'])
    const card = settingCard({
      entry: e,
      row: row('video.scale', 'fit', 'session'),
      value: 'fit',
      locked: false,
      consoleName: 'GBA',
      set,
    })
    expect(card.value).toEqual({ key: 'settings.video.scale.option.fit' })
    expect(card.trial).toBe(true)
    card.next?.()
    expect(set).toHaveBeenCalledWith('fill')
    const locked = settingCard({
      entry: e,
      row: undefined,
      value: null,
      locked: true,
      consoleName: 'GBA',
      set,
    })
    expect(locked).toMatchObject({ value: null, prev: null, next: null, trial: false })
  })
  it('offers the accelerated method only where it exists', () => {
    const set = vi.fn()
    const only = videoPathCard({ mode: 'webview', nativeAvailable: false, phone: false, set })
    expect(only).toMatchObject({ prev: null, next: null, sub: { key: 'qm.noNative' } })
    const both = videoPathCard({ mode: 'native', nativeAvailable: true, phone: true, set })
    expect(both.value).toEqual({ key: 'settings.video.path.option.accelerated' })
    expect(both.sub).toEqual({ key: 'settingsUi.onlyHere.phone' })
    both.next?.()
    expect(set).toHaveBeenCalledWith('webview')
  })
})
