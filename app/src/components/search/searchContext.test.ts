import { describe, expect, it } from 'vitest'
import type { HotkeyId } from '@/emu'
import type { KeyAction } from '@/lib/keyboard'
import {
  SHELL,
  actionReason,
  gameReason,
  settingReason,
  type GameContext,
  type SearchContext,
} from './searchContext'

const ALL: HotkeyId[] = [
  'menu',
  'save',
  'load',
  'next_slot',
  'prev_slot',
  'ff_hold',
  'ff_toggle',
  'slow',
  'rewind',
  'pause',
  'screenshot',
  'fps',
  'swap',
  'layout',
  'lid',
]

const game = (over: Partial<GameContext> = {}): SearchContext => ({
  game: {
    playing: 'gba:00ff',
    dualScreen: false,
    lidSupported: false,
    loadBlock: null,
    slot: { chosen: 2, empty: false, auto: false },
    settings: ['speed.ffval', 'video.scale'],
    ...over,
  },
})

const reasonKey = (id: KeyAction, c: SearchContext) => actionReason(id, c)?.key ?? null

describe('actionReason', () => {
  it('outside a game lets only fullscreen run', () => {
    expect(reasonKey('fullscreen', SHELL)).toBeNull()
    for (const id of ALL) expect(reasonKey(id, SHELL)).toBe('search.needsGame')
  })

  it('in the quick menu runs what the menu can do with one press', () => {
    const c = game()
    const run: KeyAction[] = [
      'menu',
      'save',
      'load',
      'next_slot',
      'prev_slot',
      'ff_toggle',
      'slow',
      'screenshot',
      'fps',
      'fullscreen',
    ]
    for (const id of run) expect(reasonKey(id, c)).toBeNull()
  })

  it('refuses the held shortcuts and the pause, with their reasons', () => {
    const c = game()
    expect(reasonKey('ff_hold', c)).toBe('search.heldInGame')
    expect(reasonKey('rewind', c)).toBe('search.heldInGame')
    expect(reasonKey('pause', c)).toBe('search.pausedInMenu')
  })

  it('needs two screens for the screen shortcuts, and a lid for the lid', () => {
    for (const id of ['swap', 'layout', 'lid'] as const)
      expect(reasonKey(id, game())).toBe('search.twoScreensOnly')
    const melon = game({ dualScreen: true, lidSupported: true })
    const azahar = game({ dualScreen: true, lidSupported: false })
    for (const id of ['swap', 'layout', 'lid'] as const) expect(reasonKey(id, melon)).toBeNull()
    expect(reasonKey('swap', azahar)).toBeNull()
    expect(reasonKey('lid', azahar)).toBe('qm.lidUnsupported')
  })

  it('saves and loads as the menu does on its chosen slot', () => {
    expect(reasonKey('save', game({ slot: { chosen: 0, empty: false, auto: true } }))).toBe(
      'qm.autoUpdates',
    )
    expect(actionReason('load', game({ slot: { chosen: 3, empty: true, auto: false } }))).toEqual({
      key: 'qm.emptySlot',
      params: { slot: 3 },
    })
    // A game whose core cannot load states says so first, empty slot or not.
    const blocked = game({
      loadBlock: 'vulkanNoTitleId',
      slot: { chosen: 3, empty: true, auto: false },
    })
    expect(reasonKey('load', blocked)).toBe('flow.loadBlock.vulkanNoTitleId')
    expect(reasonKey('save', blocked)).toBeNull()
  })
})

describe('settingReason', () => {
  it('opens every setting in the shell', () => {
    expect(settingReason('audio.vol', SHELL)).toBeNull()
  })

  it('in the quick menu opens only the rows it shows «in prova»', () => {
    expect(settingReason('speed.ffval', game())).toBeNull()
    expect(settingReason('audio.vol', game())?.key).toBe('search.settingInLibrary')
  })
})

describe('gameReason', () => {
  it('refuses only the game being played', () => {
    expect(gameReason('gba:00ff', SHELL)).toBeNull()
    expect(gameReason('gba:00ff', game())?.key).toBe('search.playingNow')
    expect(gameReason('nds:1234', game())).toBeNull()
  })
})
