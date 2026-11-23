import { describe, expect, it } from 'vitest'
import type { GameId, LibraryGame } from '@/emu'
import {
  START,
  TRAIL_MAX,
  arrivalFor,
  arrived,
  backFrom,
  closeDetails,
  openDetails,
  moved,
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
    expect(START).toEqual({ section: 'library', details: null, arrival: null, trail: [] })
    expect(showingDetails(START)).toBe(false)
  })

  it('back from the details is the grid, its cover focused once', () => {
    const d = openDetails(START, id)
    expect(showingDetails(d)).toBe(true)
    const back = closeDetails(d)
    expect(showingDetails(back)).toBe(false)
    expect(arrivalFor(back, 'library')).toEqual({ section: 'library', cover: id })
    expect(arrivalFor(arrived(back), 'library')).toBeNull()
  })

  it('keeps the details while another section is open; the rail goes to the grid (#127)', () => {
    const away = pickSection(openDetails(START, id), 'saves')
    expect(away.section).toBe('saves')
    expect(showingDetails(away)).toBe(false)
    // The rail's (or the tab bar's) Library: the grid, on the game's cover.
    const again = pickSection(away, 'library')
    expect(showingDetails(again)).toBe(false)
    expect(arrivalFor(again, 'library')).toEqual({ section: 'library', cover: id })
    // The rail's Library on the details themselves: back to the grid.
    expect(showingDetails(pickSection(openDetails(START, id), 'library'))).toBe(false)
    // «Apri la libreria» from Saves, and Android's back, are the library as it is.
    expect(showingDetails(openSection(away, 'library'))).toBe(true)
  })

  it('the jumps from a game carry it to the section they open, typed per section', () => {
    const d = openDetails(START, id)
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
    const r = settingRowFor(openDetails(START, id), 'video.vsync')
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

  it('remembers the sections left, the latest last, each once (#29)', () => {
    const saves = moved(START, pickSection(START, 'saves'))
    expect(saves.trail).toEqual(['library'])
    const settings = moved(saves, pickSection(saves, 'settings'))
    expect(settings.trail).toEqual(['library', 'saves'])
    // Back to a section already on the trail: it leaves its older place.
    const again = moved(settings, pickSection(settings, 'saves'))
    expect(again.trail).toEqual(['library', 'settings'])
    // A move inside the section (a game's details in the Library) adds nothing.
    expect(moved(START, openDetails(START, id)).trail).toEqual([])
  })

  it('remembers at most TRAIL_MAX sections', () => {
    let r = START
    for (let i = 0; i < 3 * TRAIL_MAX; i++)
      r = moved(r, pickSection(r, i % 2 ? 'library' : i % 3 ? 'saves' : 'cloud'))
    expect(r.trail.length).toBeLessThanOrEqual(TRAIL_MAX)
  })

  it('back on the phone goes along the trail, then to the Library, then leaves (#29)', () => {
    const saves = moved(START, pickSection(START, 'saves'))
    const cloud = moved(saves, pickSection(saves, 'cloud'))
    const one = backFrom(cloud)
    expect(one?.section).toBe('saves')
    expect(one?.trail).toEqual(['library'])
    const two = one && backFrom(one)
    expect(two?.section).toBe('library')
    expect(two && backFrom(two)).toBeNull()
    // No trail left (after a restart of the shell, say): the Library first.
    const lone = { ...cloud, trail: [] }
    expect(backFrom(lone)?.section).toBe('library')
    // The details the Library kept come back with it.
    const away = moved(openDetails(START, id), pickSection(openDetails(START, id), 'saves'))
    expect(showingDetails(backFrom(away)!)).toBe(true)
  })
})
