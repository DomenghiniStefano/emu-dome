// Settings rows of a feature that lives in another section (Trello #112): the row's button leads
// there through the shell's route (shell/route.ts `Jump`, `jumpTo`), on the desk and in TV alike
// (useRowChoose). Each is marked carried out in the catalogue (`.carried_out()`, catalog.rs).
import type { SettingKey } from '@/emu'
import type { Jump } from '@/components/shell/route'

export const ROW_JUMPS: Partial<Record<SettingKey, Jump>> = {
  // Cloud › «Giochi su Google Drive», the section's first tab.
  'cloud.sgames': 'cloudGames',
  // The library with L03's «Nascosti» on.
  'library.hidden': 'hiddenGames',
  // Saves › import (V05), the game found from the file's name.
  'saves.import': 'importSave',
  // Controls › C02 on the first controller, or on the keyboard.
  'pad.dev1': 'remapPad',
  'pad.dev2': 'remapKeyboard',
}
