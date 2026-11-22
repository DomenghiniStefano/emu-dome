// How the shell lays out a section. Pure: the shell feeds it the current section.
import { SECTIONS, type Section } from './sections'

/** Sections that sit flush against the rail, each column scrolling on its own. */
const FLUSH: readonly Section[] = ['settings', 'controls', 'saves', 'cloud']
/** Sections whose screen focuses a control of its own first; the others leave it on the rail. */
const OWN_INITIAL: readonly Section[] = ['library', 'settings', 'controls', 'saves', 'cloud']

/**
 * The content has no padding and does not scroll as a whole (a game's details too, and the TV
 * library, whose shelves run to the screen's edge).
 */
export function isFlush(section: Section, showingDetails: boolean, tv = false): boolean {
  return showingDetails || FLUSH.includes(section) || (tv && section === 'library')
}

/**
 * TV mode (#30): whether the screen has a TV layout of its own (the library's shelves); the others
 * keep the desk's, scaled up (`zoom-tv`). Settings in TV is an open point on the card.
 */
export function isTvLaidOut(section: Section, showingDetails: boolean): boolean {
  return section === 'library' && !showingDetails
}

/** TV mode's L1 / R1 (#30): the section before or after `section`, round the ends. */
export function sectionStep(section: Section, dir: -1 | 1): Section {
  const at = SECTIONS.indexOf(section)
  const next = SECTIONS[(at + dir + SECTIONS.length) % SECTIONS.length]
  return next ?? section
}

export function hasOwnInitialFocus(section: Section): boolean {
  return OWN_INITIAL.includes(section)
}
