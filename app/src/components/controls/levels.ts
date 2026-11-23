// C02's «Si applica a»: the levels a remap table is edited at, «Tutti i giochi», «Giochi
// <console>» and «Solo questo gioco», and the settings scope of each.
import type { ConsoleId, LibraryGame, SettingsScope } from '@/emu'
import { t } from '@/i18n'

export const RemapLevel = {
  Global: 'global',
  Console: 'console',
  Game: 'game',
} as const
export type RemapLevel = (typeof RemapLevel)[keyof typeof RemapLevel]

/** The levels on offer: the console's with a console in context, the game's with a game. */
export function remapLevels(
  console_: ConsoleId | null,
  gameScope: SettingsScope | null,
): RemapLevel[] {
  const out: RemapLevel[] = [RemapLevel.Global]
  if (console_) out.push(RemapLevel.Console)
  if (gameScope) out.push(RemapLevel.Game)
  return out
}

/** The settings scope a level reads and writes; «Tutti i giochi» when its context is missing. */
export function levelScope(
  level: RemapLevel,
  console_: ConsoleId | null,
  gameScope: SettingsScope | null,
): SettingsScope {
  if (level === RemapLevel.Game && gameScope) return gameScope
  if (level === RemapLevel.Console && console_) return { level: 'console', console: console_ }
  return { level: 'global' }
}

/** As the Settings section does: the console of the last game played (TODO design, #41). */
export function lastPlayedConsole(games: LibraryGame[]): ConsoleId | null {
  const last = [...games]
    .filter(g => g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0]
  return last?.console ?? null
}

export function levelName(level: RemapLevel, console_: ConsoleId | null): string {
  if (level === RemapLevel.Console && console_)
    return t('settingsUi.scope.console', { console: t(`console.${console_}`) })
  if (level === RemapLevel.Game) return t('controls.levelGame')
  return t('settingsUi.scope.global')
}
