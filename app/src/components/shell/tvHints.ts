// The TV button bar's hints (#30, mockup «TV-Libreria»: «A Gioca · X Dettagli del gioco · Y Cerca ·
// B Indietro · L1 R1 Cambia sezione»): what the controller's buttons do in each section. Pure: the
// bar draws each action's glyph (TvGlyph) and turns the key into text.
import type { PadAction } from '@/lib/padGlyphs'
import type { Section } from './sections'

export interface TvHint {
  /** The buttons, drawn side by side (L1 R1). */
  actions: PadAction[]
  /** The message key of what they do. */
  label: string
}

const SECTIONS_HINT: TvHint = { actions: ['prev', 'next'], label: 'tv.hint.sections' }
const BACK: TvHint = { actions: ['back'], label: 'tv.hint.back' }

/** The hints of `section`: the library's games, or choosing elsewhere (a game's details too). */
export function tvHints(section: Section, showingDetails = false): TvHint[] {
  if (section === 'library' && !showingDetails)
    return [
      { actions: ['confirm'], label: 'tv.hint.play' },
      { actions: ['more'], label: 'tv.hint.details' },
      { actions: ['search'], label: 'tv.hint.search' },
      BACK,
      SECTIONS_HINT,
    ]
  return [{ actions: ['confirm'], label: 'tv.hint.choose' }, BACK, SECTIONS_HINT]
}
