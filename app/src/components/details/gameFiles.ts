// L10 File e informazioni: what the tab says about a game's cover and file. Pure: the
// translations come in as `t`.
import type { LibraryGame, SettingDevice } from '../../emu'
import type { Translator } from './translator'

/** One line of «File del gioco»: a label and its value; `mono` for an identity in hex. */
export interface GameFact {
  label: string
  value: string
  mono?: boolean
}

/** What the cover is made of, under «Copertina». */
export function coverDescription(game: LibraryGame, t: Translator['t']): string {
  switch (game.cover) {
    case 'generated':
      return t('library.cover.generatedDesc')
    case 'icon':
      return t('library.cover.icon')
    case 'user':
      return t('library.cover.user')
  }
}

/** Where the file is: on this phone or on this computer. */
export function deviceLine(device: SettingDevice | null | undefined, t: Translator['t']): string {
  return t(device === 'phone' ? 'details.files.onPhone' : 'details.files.onComputer')
}

/**
 * Region, version and game code when the header has them, then the identity (the SHA-1 of a
 * GB, GBC, GBA or NDS game; the title ID of a 3DS game) and the emulation engine.
 */
export function gameFacts(game: LibraryGame, t: Translator['t']): GameFact[] {
  const is3ds = game.console === '3ds'
  // The hex part of the id: a SHA-1, or the 3DS title ID and version.
  const idHex = game.id.slice(game.id.indexOf(':') + 1)
  const facts: GameFact[] = []
  if (game.region)
    facts.push({ label: t('details.files.region'), value: t(`details.region.${game.region}`) })
  if (game.version !== null)
    facts.push({
      label: t('details.files.version'),
      // Header revision: 0 is the first release (1.0). The 3DS remaster version is a number.
      value: is3ds ? String(game.version) : `1.${game.version}`,
    })
  if (game.code) facts.push({ label: t('details.files.code'), value: game.code })
  facts.push({
    label: t(is3ds ? 'details.files.titleId' : 'details.files.hash'),
    value: is3ds ? idHex.slice(0, 16).toUpperCase() : idHex,
    mono: true,
  })
  facts.push({ label: t('details.files.engine'), value: t(`details.files.core.${game.core}`) })
  return facts
}
