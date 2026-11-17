// Keyboard → game buttons: the keyboard's row of `pad.remap`, resolved by Rust with its
// defaults (`controls_bindings`, family `keyboard`), so no key table is written here.
// In game it follows the open game's settings (its changes «in prova» included).
import { ref } from 'vue'
import { emu, type Button, type SettingsScope } from './emu'

/** `KeyboardEvent.code` → game button. */
export const keymap = ref<Record<string, Button>>({})

export async function loadKeymap(inGame: boolean) {
  const scope: SettingsScope = inGame ? { level: 'session' } : { level: 'global' }
  try {
    const { remap } = await emu.controls.bindings('keyboard', scope)
    const map: Record<string, Button> = {}
    for (const row of remap) if (row.source) map[row.source] = row.button
    keymap.value = map
  } catch {
    // The settings are not readable: keep the keys in use.
  }
}
