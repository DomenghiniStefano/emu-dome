// The state of the game being played, shared by the session's parts: useGameSession (what the
// quick menu and the keys do), useGameEvents (controller combos and the core's events) and
// useGameEnding (closing, crashes).
import { ref } from 'vue'
import type { GameInfo, LibraryGame, Speed, Stats } from '@/emu'
import { NORMAL } from '@/lib/speeds'
import { NO_INSET, type InsetPx } from './inset'

export function createGameState() {
  return {
    game: ref<GameInfo | null>(null),
    /** The library game being played («Colore della copertina»); null for a file. */
    playingGame: ref<LibraryGame | null>(null),
    /** The game's frame size (the core's `started` event): where the native video draws it, so
     * the side background (GameSurround) leaves that rectangle clear. */
    gameSize: ref<{ w: number; h: number } | null>(null),
    menuOpen: ref(false),
    /** Paused with the controller combo (L3 + A), menu closed. */
    paused: ref(false),
    showFps: ref(false),
    stats: ref<Stats | null>(null),
    speed: ref<Speed>({ ...NORMAL }),
    /** Fast forward held (Tab or L3 + R2, run in Rust; `emu://hotkey` says so). */
    turboHeld: ref(false),
    /** The controller in use (quick menu). */
    controller: ref<string | null>(null),
    /** The current slot (L3 + ← →, F2 / F3 move it; L3 + R1 / L1, F1 / F4 use it): the quick
     * menu opens on it. Rust holds it (`current_slot`), shared by the controller and the keyboard. */
    comboSlot: ref(1),
    /** The slot of the last load, until the next save or load: «Annulla il caricamento»
     * (`saves.undo`). */
    undoLoadSlot: ref<number | null>(null),
    /** «Muto» and «A schermo intero» (quick menu tools). */
    muted: ref(false),
    fullscreen: ref(false),
    /** The edges kept clear of the game, CSS px: the quick menu's bar, sheet or side panel, or
     * the touch controls (#29); the game is fitted in the rest. */
    videoInset: ref<InsetPx>(NO_INSET),
  }
}
export type GameState = ReturnType<typeof createGameState>
