// Pure decisions of the game session that App.vue's composables act on (no Vue, no bridge).
import type { ConsoleId, GameInfo, SettingValue, Speed, Stats, VideoMode } from '../../emu'
import { FF_DEFAULT, speedFromOption } from '../../speeds'

/** The file name without folder and extension, as the title of a game outside the library. */
export function fileTitle(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path
  return name.replace(/\.[^.]+$/, '')
}

/** With no native video underneath, the webview paints the background itself. */
export function pageBackground(mode: VideoMode): string {
  return mode === 'native' ? 'transparent' : 'var(--bg)'
}

/** G09: the share of the window's height the quick menu's bar covers (0.9 at most). */
export function insetShare(px: number, height: number): number {
  return px > 0 ? Math.min(0.9, px / height) : 0
}

/** The HUD's speed chip: shown while not at 1× (fast forward held, max, or another factor). */
export function speedShown(turboHeld: boolean, speed: Speed): boolean {
  return turboHeld || speed.kind === 'max' || (speed.kind === 'factor' && speed.value !== 1)
}

/**
 * Backspace: whether rewind is on after this press or release. Held by default; with
 * `a11y.toggle` each press switches it and a release changes nothing.
 */
export function rewindAfter(toggle: boolean, pressed: boolean, rewinding: boolean): boolean {
  if (!toggle) return pressed
  return pressed ? !rewinding : rewinding
}

/** The open game's settings the shell itself applies (session level, «in prova» included). */
export interface GameSettings {
  /** `speed.ffval`: fast forward while Tab is held and on the HUD. */
  ffSpeed: Speed
  /** `saves.undo`: «Annulla il caricamento». */
  undoAllowed: boolean
  /** `a11y.toggle`: Backspace switches rewind instead of holding it. */
  rewindToggle: boolean
  /** `a11y.cblind`: the colour correction on the game. */
  colourBlind: string
}

export const GAME_SETTING_PAGES = ['speed', 'saves', 'a11y'] as const

export function gameSettingsFrom(
  rows: { key: string; value: SettingValue | null }[],
): GameSettings {
  const of = (key: string) => rows.find(r => r.key === key)?.value
  return {
    ffSpeed: speedFromOption(of('speed.ffval')) ?? FF_DEFAULT,
    undoAllowed: of('saves.undo') !== false,
    rewindToggle: of('a11y.toggle') === true,
    colourBlind: String(of('a11y.cblind') ?? 'none'),
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
