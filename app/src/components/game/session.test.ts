import { describe, expect, it } from 'vitest'
import type { GameInfo, Stats } from '@/emu'
import { FF_DEFAULT } from '@/lib/speeds'
import {
  crashReport,
  fileTitle,
  gameSettingsFrom,
  insetShare,
  pageBackground,
  speedShown,
} from './session'

describe('fileTitle', () => {
  it('drops the folder and the extension', () => {
    expect(fileTitle('C:\\Games\\Pocket Quest.gba')).toBe('Pocket Quest')
    expect(fileTitle('/home/me/games/tiny.blocks.gb')).toBe('tiny.blocks')
  })
  it('keeps a name without extension', () => {
    expect(fileTitle('rom')).toBe('rom')
  })
})

describe('pageBackground', () => {
  it('is transparent over the native video, the ground in the webview', () => {
    expect(pageBackground('native')).toBe('transparent')
    expect(pageBackground('webview')).toBe('var(--bg)')
  })
})

describe('insetShare', () => {
  it('is the covered share of the height, 0.9 at most', () => {
    expect(insetShare(0, 800)).toBe(0)
    expect(insetShare(200, 800)).toBe(0.25)
    expect(insetShare(900, 800)).toBe(0.9)
  })
})

describe('speedShown', () => {
  it('hides the chip at 1×', () => {
    expect(speedShown(false, { kind: 'factor', value: 1 })).toBe(false)
  })
  it('shows it at any other speed or while fast forward is held', () => {
    expect(speedShown(false, { kind: 'factor', value: 2 })).toBe(true)
    expect(speedShown(false, { kind: 'max' })).toBe(true)
    expect(speedShown(true, { kind: 'factor', value: 1 })).toBe(true)
  })
})

describe('gameSettingsFrom', () => {
  it('falls back to the defaults', () => {
    expect(gameSettingsFrom([])).toEqual({
      ffSpeed: FF_DEFAULT,
      undoAllowed: true,
      colourBlind: 'none',
    })
  })
  it('reads the rows', () => {
    const s = gameSettingsFrom([
      { key: 'speed.ffval', value: 'max' },
      { key: 'saves.undo', value: false },
      { key: 'a11y.cblind', value: 'deutan' },
    ])
    expect(s.ffSpeed).toEqual({ kind: 'max' })
    expect(s.undoAllowed).toBe(false)
    expect(s.colourBlind).toBe('deutan')
  })
})

describe('crashReport', () => {
  const game: GameInfo = {
    id: 'gba:0a',
    title: 'Pocket Quest',
    console: 'Game Boy Advance',
    consoleId: 'gba',
    core: 'mgba',
    resumable: false,
    autosaveOnExit: true,
  }
  const now = new Date(Date.UTC(2026, 10, 18, 12, 0, 0))
  it('names the game, the frames and the time', () => {
    const stats: Stats = {
      running: true,
      emuFps: 59.73,
      targetFps: 60,
      speed: null,
      paused: false,
      frames: 1200,
      autoSavedAt: 1000,
    }
    expect(crashReport(game, stats, now)).toEqual({
      console: 'gba',
      at: 1000,
      detail: 'Pocket Quest · mgba\nframes 1200 · 59.7 / 60 FPS\n2026-11-18T12:00:00.000Z',
    })
  })
  it('works before the first stats', () => {
    expect(crashReport(game, null, now).detail).toBe(
      'Pocket Quest · mgba\nframes 0 · 0 / 0 FPS\n2026-11-18T12:00:00.000Z',
    )
  })
})
