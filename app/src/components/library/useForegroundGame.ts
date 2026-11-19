// The cover in the foreground of the library, whose colours tint the shell's haze (ambient.ts):
// the game under the pointer or the focus; until one, the first «Continua a giocare» (or the
// first of the grid). When the library changes (a scan, a new cover) the same game stays, with
// its fresh record.
import { watch, type Ref } from 'vue'
import { ambientGame } from '../../ambient'
import type { LibraryGame } from '../../emu'
import { library } from '../../library'

export function useForegroundGame(recent: Ref<LibraryGame[]>, games: Ref<LibraryGame[]>) {
  watch(
    [() => recent.value[0] ?? games.value[0] ?? null, () => library.games],
    ([first]) => {
      const current = ambientGame.value
      ambientGame.value = (current && library.games.find(g => g.id === current.id)) || first
    },
    { immediate: true },
  )
  /** The pointer or the focus reached this game. */
  return (g: LibraryGame) => {
    ambientGame.value = g
  }
}
