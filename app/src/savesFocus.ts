// The game the Saves section should show first, when another screen sends you there for one game
// (L01 «Continua a giocare» → «Salvataggi»). Set before switching section; the Saves section
// takes it and clears it.
//
// TODO(#56 part B → part A): SavesView.vue does not read it yet (it belongs to the quick menu
// stream): on mount it should pick `savesFocus.value` as `selectedPath` and set it back to null.
import { ref } from 'vue'

/** ROM path of the game to show in the Saves section; null = the section's own choice. */
export const savesFocus = ref<string | null>(null)
