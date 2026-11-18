// How the shell lays out a section. Pure: the shell feeds it the current section.
import type { Section } from '../../sections'

/** Sections that sit flush against the rail, each column scrolling on its own. */
const FLUSH: readonly Section[] = ['settings', 'controls', 'saves']
/** Sections whose screen focuses a control of its own first; the others leave it on the rail. */
const OWN_INITIAL: readonly Section[] = ['library', 'settings', 'controls', 'saves']

/** The content has no padding and does not scroll as a whole (a game's details too). */
export function isFlush(section: Section, showingDetails: boolean): boolean {
  return showingDetails || FLUSH.includes(section)
}

export function hasOwnInitialFocus(section: Section): boolean {
  return OWN_INITIAL.includes(section)
}
