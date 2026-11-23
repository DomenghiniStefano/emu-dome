// The whole screen for a layout editor opened outside a game (C06 and C09 from Settings, #131):
// Android's system bars hidden while it is open, shown again when it closes, so the editor places
// the controls and the screens as a game is placed (a game holds the bars hidden by itself,
// `game_hooks` in Rust). Nothing on the desktop. `on` is read once: over a game (`live`) the
// editors leave the bars to the game.
import { onBeforeUnmount, onMounted } from 'vue'
import { emu } from '@/emu'
import { ignore } from '@/lib/errors'

export function useImmersive(on: boolean) {
  if (!on) return
  // A failure leaves the bars as they were: the editor still works, over or under them.
  onMounted(() => ignore(emu.setImmersive(true), 'system bars not hidden'))
  onBeforeUnmount(() => ignore(emu.setImmersive(false), 'system bars not shown again'))
}
