// L12 «Scegli la copertina»'s choices (Trello #85): the cover kinds the library knows, with «Da
// uno screenshot» before «Immagine tua» (the owner's mockup). Both of those make a user cover
// through «Adatta la copertina»; generated and icon apply at once.
import type { CoverKind, LibraryGame } from '@/emu'
import { coverKinds } from '@/components/library/libraryView'

export type CoverChoice = CoverKind | 'screenshot'

export function coverChoices(game: Pick<LibraryGame, 'hasIcon'>): CoverChoice[] {
  const kinds = coverKinds(game).filter(k => k !== 'user')
  return [...kinds, 'screenshot', 'user']
}

/** Where a choice goes when confirmed. */
export type ChoiceNext = 'apply' | 'screenshots' | 'pick'

export function choiceNext(choice: CoverChoice): ChoiceNext {
  switch (choice) {
    case 'screenshot':
      return 'screenshots'
    case 'user':
      return 'pick'
    case 'generated':
    case 'icon':
      return 'apply'
  }
}
