import { describe, expect, it } from 'vitest'
import type { GameId, LibraryGame } from '@/emu'
import {
  START,
  arrivalFor,
  arrived,
  closeDetails,
  openDetails,
  openSection,
  pickSection,
  remapFor,
  savesFor,
  settingRowFor,
  settingsFor,
  showingDetails,
} from './route'

const id = 'gba:00ff' as GameId
const game = { id, console: 'gba', title: 'Test' } as Pick<LibraryGame, 'id' | 'console' | 'title'>

describe('the shell route', () => {
  it('starts on the library grid with nothing to arrive at', () => {
    expect(START).toEqual({ section: 'library', details: null, arrival: null })
    expect(showingDetails(START)).toBe(false)
  })

  it('back from the details is the grid, its cover focused once', () => {
    const d = openDetails(id)
    expect(showingDetails(d)).toBe(true)
    const back = closeDetails(d)
    expect(showingDetails(back)).toBe(false)
    expect(arrivalFor(back, 'library')).toEqual({ section: 'library', cover: id })
    expect(arrivalFor(arrived(back), 'library')).toBeNull()
  })

  it('keeps the details while another section is open, and the rail comes back to them', () => {
    const away = pickSection(openDetails(id), 'saves')
    expect(away.section).toBe('saves')
    expect(showingDetails(away)).toBe(false)
    const again = pickSection(away, 'library')
    expect(showingDetails(again)).toBe(true)
    // The rail's Library on the details themselves: back to the grid.
    expect(showingDetails(pickSection(again, 'library'))).toBe(false)
    // «Apri la libreria» from Saves is the library as it is.
    expect(showingDetails(openSection(away, 'library'))).toBe(true)
  })

  it('the jumps from a game carry it to the section they open, typed per section', () => {
    const d = openDetails(id)
    expect(arrivalFor(savesFor(d, id), 'saves')).toEqual({
      section: 'saves',
      game: id,
      importing: false,
    })
    expect(arrivalFor(savesFor(d, id, true), 'saves')?.importing).toBe(true)
    const settings = settingsFor(d, game)
    expect(settings.section).toBe('settings')
    expect(arrivalFor(settings, 'settings')?.game).toEqual({
      scope: { level: 'game', console: 'gba', game: id },
      title: 'Test',
    })
    expect(arrivalFor(settings, 'settings')?.row).toBeNull()
    expect(arrivalFor(settings, 'controls')).toBeNull()
    expect(arrivalFor(remapFor(d, game), 'controls')?.game.title).toBe('Test')
    // The details stay remembered behind every jump.
    expect(remapFor(d, game).details).toBe(id)
  })

  it('the search opens one setting on its page, for every game, the details kept', () => {
    const r = settingRowFor(openDetails(id), 'video.vsync')
    expect(r.section).toBe('settings')
    expect(arrivalFor(r, 'settings')).toEqual({
      section: 'settings',
      game: null,
      row: 'video.vsync',
    })
    expect(r.details).toBe(id)
  })

  it('an arrival is dropped once read; a route without one is left as it is', () => {
    const r = savesFor(START, id)
    expect(arrived(r).arrival).toBeNull()
    expect(arrived(START)).toBe(START)
    // Opening a section drops what was meant for another.
    expect(openSection(r, 'controls').arrival).toBeNull()
  })
})
