// X01 / X02: the matching of the search everywhere, pure. The palette (SearchPalette.vue) hands
// in its candidates already in words (a setting's name in the current language, its page, its
// synonyms from the catalogue; an action's label; a game's title and console) and gets back the
// three groups in order, or, when nothing matches, the suggestions «Forse cercavi:».
//
// How a candidate matches, best first (case and accents aside, `fold`):
// - its name is the query, starts with it, has a word that starts with it, has every word of the
//   query at the start of one of its words, or contains it;
// - its other words (a setting's description and page, a game's console) contain the query, or
//   every word of the query;
// - a synonym (`SYNONYMS` in catalog.rs) starts with the query, contains it or is contained in it.
// Synonym hits rank after every other hit of their group. When no candidate matches by its name or
// its words, the synonym hits are the suggestions (X02: «tearing» suggests V-Sync), and failing
// those, the names and synonyms one or two letters away from a word of the query (a typo).
import { fold } from '@/lib/settingsModel'

/** What a candidate is found by, in the current language. */
export interface Candidate {
  name: string
  /** Its other words: a setting's description and page, a game's console. */
  words?: string
  /** Extra words in any language (a setting's synonyms from the catalogue). */
  synonyms?: readonly string[]
}

/** A query, folded once. */
export interface Query {
  text: string
  tokens: string[]
}

export function toQuery(raw: string): Query {
  const text = fold(raw).trim().replace(/\s+/g, ' ')
  return { text, tokens: text ? text.split(' ') : [] }
}

/** How well a candidate matches; `bySynonym`: through a synonym only. */
export interface Match {
  score: number
  bySynonym: boolean
}

const wordsOf = (s: string) => s.split(/[^\p{L}\p{N}]+/u).filter(Boolean)

/** Every token of the query starts a word of `words`. */
const everyTokenStarts = (tokens: string[], words: string[]) =>
  tokens.every(t => words.some(w => w.startsWith(t)))

/** The candidate's match for `q`, or null. A shorter name wins a tie. */
export function match(q: Query, c: Candidate): Match | null {
  if (!q.text) return null
  const name = fold(c.name)
  const nameWords = wordsOf(name)
  const tie = Math.min(name.length, 99) / 100
  const scored = (score: number, bySynonym = false): Match => ({ score: score - tie, bySynonym })
  if (name === q.text) return scored(100)
  if (name.startsWith(q.text)) return scored(80)
  if (nameWords.some(w => w.startsWith(q.text))) return scored(60)
  if (everyTokenStarts(q.tokens, nameWords)) return scored(50)
  if (name.includes(q.text)) return scored(40)
  const other = fold(c.words ?? '')
  if (other.includes(q.text)) return scored(20)
  if (q.tokens.length > 1 && q.tokens.every(t => name.includes(t) || other.includes(t)))
    return scored(15)
  for (const raw of c.synonyms ?? []) {
    const s = fold(raw)
    if (s.startsWith(q.text) || s.includes(q.text) || (s.length > 2 && q.text.includes(s)))
      return scored(10, true)
  }
  return null
}

/** One candidate found, with its match. */
export interface Hit<T> {
  item: T
  score: number
  bySynonym: boolean
}

/** `items` that match `q`, best first, then in their own order. */
export function rank<T>(q: Query, items: readonly T[], candidate: (t: T) => Candidate): Hit<T>[] {
  const hits: (Hit<T> & { at: number })[] = []
  items.forEach((item, at) => {
    const m = match(q, candidate(item))
    if (m) hits.push({ item, ...m, at })
  })
  hits.sort((a, b) => b.score - a.score || a.at - b.at)
  return hits.map(({ item, score, bySynonym }) => ({ item, score, bySynonym }))
}

/** Edit distance of `a` and `b`, or `limit + 1` once it is over `limit`. */
export function distance(a: string, b: string, limit: number): number {
  if (Math.abs(a.length - b.length) > limit) return limit + 1
  let prev = Array.from({ length: b.length + 1 }, (_, i) => i)
  for (let i = 1; i <= a.length; i++) {
    const row = [i]
    let best = i
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1
      const v = Math.min((prev[j] ?? 0) + 1, (row[j - 1] ?? 0) + 1, (prev[j - 1] ?? 0) + cost)
      row.push(v)
      best = Math.min(best, v)
    }
    if (best > limit) return limit + 1
    prev = row
  }
  return prev[b.length] ?? limit + 1
}

/** How many letters a word of `length` may be off by: none under 4, one up to 6, else two. */
const typoLimit = (length: number) => (length < 4 ? 0 : length <= 6 ? 1 : 2)

/** Whether a word of the query is a typo of a word of the candidate's name or synonyms. */
export function nearMiss(q: Query, c: Candidate): boolean {
  const words = [...wordsOf(fold(c.name)), ...(c.synonyms ?? []).flatMap(s => wordsOf(fold(s)))]
  return q.tokens.some(t => {
    const limit = typoLimit(t.length)
    return limit > 0 && words.some(w => distance(t, w, limit) <= limit)
  })
}

/** What the palette searches, each group with how its items are found. */
export interface Sources<A, S, G> {
  actions: readonly A[]
  action: (a: A) => Candidate
  settings: readonly S[]
  setting: (s: S) => Candidate
  games: readonly G[]
  game: (g: G) => Candidate
}

/** How many results each group shows at most. */
export const LIMITS = { actions: 4, settings: 6, games: 5, suggestions: 3 } as const

/** The palette's answer: the three groups, or the suggestions when they are all empty. */
export interface Results<A, S, G> {
  actions: A[]
  settings: S[]
  games: G[]
  /** X02 «Forse cercavi:»: settings, only when no group has a result. */
  suggestions: S[]
}

/** X01 / X02: the results for `raw` (an empty query finds nothing). */
export function search<A, S, G>(raw: string, src: Sources<A, S, G>): Results<A, S, G> {
  const q = toQuery(raw)
  const none: Results<A, S, G> = { actions: [], settings: [], games: [], suggestions: [] }
  if (!q.text) return none
  const actions = rank(q, src.actions, src.action)
  const settings = rank(q, src.settings, src.setting)
  const games = rank(q, src.games, src.game)
  const direct = <T>(hits: Hit<T>[]) => hits.filter(h => !h.bySynonym)
  const found = direct(actions).length + direct(settings).length + direct(games).length > 0
  const items = <T>(hits: Hit<T>[], limit: number) => hits.slice(0, limit).map(h => h.item)
  if (found)
    return {
      actions: items(actions, LIMITS.actions),
      // Synonym hits stay, after the others (`rank` puts them last).
      settings: items(settings, LIMITS.settings),
      games: items(games, LIMITS.games),
      suggestions: [],
    }
  const bySynonym = items(settings, LIMITS.suggestions)
  const suggestions = bySynonym.length
    ? bySynonym
    : src.settings.filter(s => nearMiss(q, src.setting(s))).slice(0, LIMITS.suggestions)
  return { ...none, suggestions }
}
