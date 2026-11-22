import { describe, expect, it, vi } from 'vitest'
import { distance, match, nearMiss, rank, search, toQuery, type Candidate } from './search'

// `fold` lives in settingsModel, which reaches the bridge.
vi.mock('@/emu', () => ({ emu: {} }))

const c = (name: string, words = '', synonyms: string[] = []): Candidate => ({
  name,
  words,
  synonyms,
})

describe('match', () => {
  const q = toQuery

  it('ignores case, accents and extra spaces', () => {
    expect(match(q('  VELOCITÀ  '), c('Velocita'))).not.toBeNull()
    expect(match(q('velocita'), c('Velocità massima'))).not.toBeNull()
  })

  it('ranks exact, then prefix, then a word prefix, then every word, then a substring', () => {
    const scores = [
      match(q('rotazione'), c('Rotazione')),
      match(q('rotaz'), c('Rotazione')),
      match(q('spazio'), c('Lo spazio tra gli schermi')),
      match(q('spa sch'), c('Lo spazio tra gli schermi')),
      match(q('azio'), c('Rotazione')),
    ].map(m => m?.score ?? -1)
    expect([...scores].sort((a, b) => b - a)).toEqual(scores)
    expect(scores.every(s => s > 0)).toBe(true)
  })

  it('finds by the other words, below any name hit', () => {
    const byWords = match(q('ds e 3ds'), c('Rotazione', 'Impostazioni · Schermi di DS e 3DS'))
    const byName = match(q('rot'), c('Rotazione'))
    expect(byWords).not.toBeNull()
    expect(byWords!.score).toBeLessThan(byName!.score)
    expect(byWords!.bySynonym).toBe(false)
  })

  it('finds by a synonym, marked as such, below everything else', () => {
    const m = match(q('tearing'), c('Sincronizzazione verticale (V-Sync)', '', ['tearing']))
    expect(m?.bySynonym).toBe(true)
    expect(match(q('input lag please'), c('Run-ahead', '', ['lag']))?.bySynonym).toBe(true)
    // A synonym too short to be read inside the query does not match it.
    expect(match(q('xyz ab'), c('Nome', '', ['ab']))).toBeNull()
  })

  it('a shorter name wins a tie, an empty query matches nothing', () => {
    expect(match(q('vol'), c('Volume'))!.score).toBeGreaterThan(
      match(q('vol'), c('Volumetto'))!.score,
    )
    expect(match(q('   '), c('Volume'))).toBeNull()
  })
})

describe('rank', () => {
  it('keeps the items that match, best first, then in their own order', () => {
    const items = ['Spazio tra gli schermi', 'Schermo intero', 'Audio', 'Schermi']
    const hits = rank(toQuery('schermi'), items, n => c(n))
    expect(hits.map(h => h.item)).toEqual(['Schermi', 'Spazio tra gli schermi'])
    expect(rank(toQuery('scherm'), items, n => c(n)).map(h => h.item)).toEqual([
      'Schermi',
      'Schermo intero',
      'Spazio tra gli schermi',
    ])
  })
})

describe('distance and nearMiss', () => {
  it('counts the letters to change, stopping past the limit', () => {
    expect(distance('tearing', 'tearing', 2)).toBe(0)
    expect(distance('tearng', 'tearing', 2)).toBe(1)
    expect(distance('vsnc', 'vsync', 1)).toBe(1)
    expect(distance('audio', 'video', 1)).toBe(2)
    expect(distance('a', 'abcdef', 2)).toBe(3)
  })

  it('a typo of a name word or a synonym is a near miss; short words never are', () => {
    expect(nearMiss(toQuery('tearng'), c('V-Sync', '', ['tearing']))).toBe(true)
    expect(nearMiss(toQuery('volme'), c('Volume'))).toBe(true)
    expect(nearMiss(toQuery('vol'), c('Val'))).toBe(false)
    expect(nearMiss(toQuery('zzzzzz'), c('Volume'))).toBe(false)
  })
})

describe('search', () => {
  const src = {
    actions: ['Scambia gli schermi', 'Salvataggio rapido'],
    action: (n: string) => c(n),
    settings: [
      { name: 'Spazio tra gli schermi', page: 'Schermi di DS e 3DS', syn: [] as string[] },
      { name: 'Sincronizzazione verticale (V-Sync)', page: 'Video', syn: ['tearing', 'strappo'] },
      { name: 'Metodo di visualizzazione', page: 'Video', syn: ['schermo nero'] },
    ],
    setting: (s: { name: string; page: string; syn: string[] }) => c(s.name, s.page, s.syn),
    games: ['Cronache di Brina'],
    game: (n: string) => c(n, 'Nintendo DS'),
  }

  it('answers the three groups, each in its order', () => {
    const r = search('schermi', src)
    expect(r.actions).toEqual(['Scambia gli schermi'])
    expect(r.settings.map(s => s.name)).toEqual(['Spazio tra gli schermi'])
    expect(r.games).toEqual([])
    expect(r.suggestions).toEqual([])
  })

  it('keeps synonym hits after the others when something else matched', () => {
    const r = search('scherm', src)
    expect(r.actions).toEqual(['Scambia gli schermi'])
    expect(r.settings.map(s => s.name)).toEqual([
      'Spazio tra gli schermi',
      'Metodo di visualizzazione',
    ])
  })

  it('X02: only synonyms match, so they are the suggestions and the groups are empty', () => {
    const r = search('tearing', src)
    expect(r.actions).toEqual([])
    expect(r.settings).toEqual([])
    expect(r.suggestions.map(s => s.name)).toEqual(['Sincronizzazione verticale (V-Sync)'])
  })

  it('X02: nothing at all matches, so the typos are the suggestions', () => {
    expect(search('tearign', src).suggestions.map(s => s.name)).toEqual([
      'Sincronizzazione verticale (V-Sync)',
    ])
    expect(search('qwertyuiop', src)).toEqual({
      actions: [],
      settings: [],
      games: [],
      suggestions: [],
    })
  })

  it('finds a game by its console and caps every group', () => {
    expect(search('nintendo', src).games).toEqual(['Cronache di Brina'])
    const many = { ...src, games: Array.from({ length: 20 }, (_, i) => `Gioco ${i}`) }
    expect(search('gioco', many).games).toHaveLength(5)
  })

  it('an empty query finds nothing', () => {
    expect(search('  ', src)).toEqual({ actions: [], settings: [], games: [], suggestions: [] })
  })
})
