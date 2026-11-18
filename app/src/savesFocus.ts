// The game the Saves section should show first, when another screen sends you there for one game
// (L01 «Continua a giocare» → «Salvataggi», L05 «Mostra i backup»), or import a save for
// (L05 «Importa…»). Set before switching section; the Saves section takes it and clears it.
import { ref } from 'vue'

/** ROM path of the game to show in the Saves section; null = the section's own choice. */
export const savesFocus = ref<string | null>(null)

/** ROM path of the game to import a save for (V05 with the game already chosen). */
export const savesImportFor = ref<string | null>(null)
