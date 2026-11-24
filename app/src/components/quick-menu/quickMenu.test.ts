import { describe, expect, it, vi } from 'vitest'
import type { ScreenStatus, SettingEntry, SettingRow } from '@/emu'
import { backKeyLabel } from './hints'
import {
  cycleStartingAt,
  layoutCard,
  settingCard,
  sourceLine,
  stepper,
  unavailableOptions,
  videoPathCard,
} from './screenCards'
import { ScreenPickId, pickChanges, screenPicks } from './screenPicks'
import { isEmptySlot, loadLabel, loadedLine, mainSlotCount, slotName } from './slots'
import { chosenSpeed, followedSpeed, modeSpeeds, speedChoices } from './speedChoices'
import { ToolId, quickTools, toolReasons, toolsNotYet } from './tools'
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
    needsNativeVideo: false,
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
  it('tells an empty slot from a saved one', () => {
    const empty = { slot: 1, savedAt: null, thumbUrl: null }
    const saved = { slot: 1, savedAt: Date.UTC(2026, 10, 18, 9, 30), thumbUrl: null }
    expect(isEmptySlot(undefined)).toBe(true)
    expect(isEmptySlot(empty)).toBe(true)
    expect(isEmptySlot(saved)).toBe(false)
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
  const base = { phone: false, dualScreen: false, muted: false, fullscreen: true, screens: null }
  const screens = (over: Partial<ScreenStatus>): ScreenStatus => ({
    dualScreen: true,
    layout: 'vertical',
    cycle: ['vertical', 'side'],
    swapped: false,
    swapHold: false,
    hasCustom: false,
    lidSupported: true,
    lidClosed: false,
    phonePlacement: false,
    ...over,
  })
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
  it('adds swap and the lid for NDS and 3DS, pressed while they are on', () => {
    const tools = quickTools({ ...base, dualScreen: true })
    expect(tools).toHaveLength(6)
    expect(toolsNotYet(tools)).toEqual([])
    expect(toolReasons(tools)).toEqual([])
    const on = quickTools({
      ...base,
      dualScreen: true,
      screens: screens({ swapped: true, lidClosed: true }),
    })
    expect(on.find(t => t.id === ToolId.SwapScreens)?.pressed).toBe(true)
    expect(on.find(t => t.id === ToolId.CloseLid)?.pressed).toBe(true)
    expect(toolsNotYet(quickTools(base))).toEqual([])
  })
  it('disables the lid with the reason where the emulator has none (3DS)', () => {
    const tools = quickTools({
      ...base,
      dualScreen: true,
      screens: screens({ lidSupported: false }),
    })
    const lid = tools.find(t => t.id === ToolId.CloseLid)
    expect(lid?.available).toBe(false)
    expect(toolReasons(tools)).toEqual(['qm.lidUnsupported'])
    // A reason, not «arrives in a future update».
    expect(toolsNotYet(tools)).toEqual([])
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
  it('names the layout card’s change by the layout the game opens with', () => {
    const names = trialNames([row('screens.cycle', ['side', 'vertical'], 'session')], find)
    expect(names).toEqual(['settings.screens.cycle.option.side'])
  })
  it('titles one change and several', () => {
    expect(trialTitle(['A'])).toEqual({ key: 'qm.trialOne', params: { names: 'A' } })
    expect(trialTitle(['A', 'B'])).toEqual({ key: 'qm.trial', params: { n: 2, names: 'A, B' } })
  })
})

describe('G09 cards', () => {
  it('shows the layout card disabled with its reason while the touch layout places the screens', () => {
    const status = {
      dualScreen: true,
      layout: 'side',
      cycle: ['vertical', 'side'],
      swapped: false,
      swapHold: false,
      hasCustom: false,
      lidSupported: true,
      lidClosed: false,
      phonePlacement: true,
    } satisfies ScreenStatus
    const card = layoutCard({ status, row: undefined, consoleName: 'DS', set: vi.fn() })
    expect(card.value).toEqual({ key: 'qm.layoutPhone.value' })
    expect(card.sub).toEqual({ key: 'qm.layoutPhone.why' })
    expect(card.trial).toBe(false)
    expect(card.prev).toBe(null)
    expect(card.next).toBe(null)
  })

  it('steps the layout card through the cycle, from the layout shown', () => {
    const set = vi.fn()
    const status = {
      dualScreen: true,
      layout: 'side',
      cycle: ['vertical', 'side', 'big_small'],
      swapped: false,
      swapHold: false,
      hasCustom: false,
      lidSupported: true,
      lidClosed: false,
      phonePlacement: false,
    } satisfies ScreenStatus
    const card = layoutCard({ status, row: undefined, consoleName: 'Nintendo DS', set })
    expect(card.name).toEqual({ key: 'qm.layout' })
    expect(card.value).toEqual({ key: 'settings.screens.cycle.option.side' })
    expect(card.trial).toBe(false)
    card.next?.()
    card.prev?.()
    expect(set.mock.calls).toEqual([['big_small'], ['vertical']])
    // «In prova» once the cycle was changed here.
    const trial = layoutCard({
      status,
      row: row('screens.cycle', ['side', 'big_small', 'vertical'], 'session'),
      consoleName: 'Nintendo DS',
      set,
    })
    expect(trial.trial).toBe(true)
    expect(trial.sub).toEqual({ key: 'qm.source.session' })
    // A layout shown from outside the cycle (the editor's custom) steps into it.
    const outside = layoutCard({
      status: { ...status, layout: 'custom' },
      row: undefined,
      consoleName: 'Nintendo DS',
      set,
    })
    expect(outside.prev).toBeNull()
    outside.next?.()
    expect(set).toHaveBeenLastCalledWith('vertical')
  })
  it('turns the cycle to start at the chosen layout, keeping its order', () => {
    expect(cycleStartingAt(['vertical', 'side', 'big_small', 'top_only'], 'big_small')).toEqual([
      'big_small',
      'top_only',
      'vertical',
      'side',
    ])
    expect(cycleStartingAt(['vertical', 'side'], 'custom')).toEqual(['custom', 'vertical', 'side'])
  })
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
      reason: null,
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
      reason: 'settingsUi.lockedNeedsNative',
      consoleName: 'GBA',
      set,
    })
    expect(locked).toMatchObject({ value: null, prev: null, next: null, trial: false })
    // A locked card says why instead of where its value comes from (X05).
    expect(locked.sub).toEqual({ key: 'settingsUi.lockedNeedsNative' })
  })
  it('offers «Effetto da file» only once a file was chosen', () => {
    const none = (key: string) => (key === 'fx.fxfile' ? '' : null)
    const chosen = (key: string) => (key === 'fx.fxfile' ? 'C:/fx/crt.slangp' : null)
    expect(unavailableOptions('fx.fxpick', none)).toEqual(['file'])
    expect(unavailableOptions('fx.fxpick', chosen)).toEqual([])
    expect(unavailableOptions('video.scale', none)).toEqual([])
    const set = vi.fn()
    const card = settingCard({
      entry: entry('fx.fxpick', ['none', 'crt', 'file']),
      row: undefined,
      value: 'crt',
      reason: null,
      unavailable: ['file'],
      consoleName: 'GBA',
      set,
    })
    expect(card.next).toBeNull()
    card.prev?.()
    expect(set).toHaveBeenCalledWith('none')
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

describe('screen picks', () => {
  const values: Record<string, string> = {
    'video.scale': 'integer',
    'video.filter': 'none',
    'fx.fxpick': 'none',
  }
  const value = (key: string) => values[key] ?? null

  it('marks the pick whose every value is the current one', () => {
    const picks = screenPicks({ value, locked: () => false })
    expect(picks.map(p => [p.id, p.on])).toEqual([
      [ScreenPickId.Sharp, true],
      [ScreenPickId.Soft, false],
      [ScreenPickId.Lcd, false],
      [ScreenPickId.Crt, false],
    ])
    expect(picks.every(p => p.available)).toBe(true)
    expect(picks[0]?.label).toBe('qm.picks.sharp')
  })

  it('disables a pick that would change a locked setting (the effects on Compatible)', () => {
    const picks = screenPicks({ value, locked: key => key === 'fx.fxpick' })
    expect(picks.filter(p => p.available).map(p => p.id)).toEqual([ScreenPickId.Sharp])
  })

  it('changes only the values that differ', () => {
    expect(pickChanges(ScreenPickId.Lcd, value)).toEqual([['fx.fxpick', 'lcd']])
    expect(pickChanges(ScreenPickId.Sharp, value)).toEqual([])
    expect(pickChanges(ScreenPickId.Crt, value)).toEqual([
      ['video.scale', 'fit'],
      ['fx.fxpick', 'crt'],
    ])
  })
})
