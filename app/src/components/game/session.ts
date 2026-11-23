// Pure decisions of the game session that App.vue's composables act on (no Vue, no bridge).
import type { ConsoleId, GameInfo, SettingValue, Speed, Stats, VideoMode } from '@/emu'
import { modeSpeeds } from '@/components/quick-menu/speedChoices'

/** With no native video underneath, the webview paints the background itself. */
export function pageBackground(mode: VideoMode): string {
  return mode === 'native' ? 'transparent' : 'var(--bg)'
}

/** The HUD's speed chip: shown while not at 1× (fast forward held, max, or another factor). */
export function speedShown(turboHeld: boolean, speed: Speed): boolean {
  return turboHeld || speed.kind === 'max' || (speed.kind === 'factor' && speed.value !== 1)
}

/** The open game's settings the shell itself shows or applies (session level, «in prova»
 * included). Fast forward and rewind themselves run in Rust (the keys and the controllers). */
export interface GameSettings {
  /** `speed.ffval`, shown on the HUD while fast forward is held. */
  ffSpeed: Speed
  /** `saves.undo`: «Annulla il caricamento». */
  undoAllowed: boolean
  /** `a11y.cblind`: the colour correction on the game. */
  colourBlind: string
  /** `video.filter`: how the webview canvas smooths the game (the native renderer reads it in
   * Rust). */
  smoothing: string
}

export const GAME_SETTING_PAGES = ['speed', 'saves', 'a11y', 'video'] as const

export function gameSettingsFrom(
  rows: { key: string; value: SettingValue | null }[],
): GameSettings {
  const of = (key: string) => rows.find(r => r.key === key)?.value
  return {
    // The quick menu's reading of the speed options, the same one its Avanti veloce uses.
    ffSpeed: modeSpeeds(of('speed.ffval'), of('speed.slow')).ff,
    undoAllowed: of('saves.undo') !== false,
    colourBlind: String(of('a11y.cblind') ?? 'none'),
    smoothing: String(of('video.filter') ?? 'light'),
  }
}

/** E11: what the crash dialog says and «Copia i dettagli» copies. */
export function crashReport(
  game: GameInfo,
  stats: Stats | null,
  now: Date,
): { console: ConsoleId; at: number | null; detail: string } {
  return {
    console: game.consoleId,
    at: stats?.autoSavedAt ?? null,
    detail: [
      `${game.title} · ${game.core}`,
      `frames ${stats?.frames ?? 0} · ${stats ? stats.emuFps.toFixed(1) : '0'} / ${stats?.targetFps ?? 0} FPS`,
      now.toISOString(),
    ].join('\n'),
  }
}
