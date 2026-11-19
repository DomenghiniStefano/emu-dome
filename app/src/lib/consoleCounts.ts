// Games counted per console, the way the interface shows them: one tile for each of GBA, NDS and
// 3DS, and one for Game Boy and Game Boy Color together. The first run's S02 counts the library,
// the library's L11 counts what a scan added: both through `consoleCounts`.
import type { ConsoleId, LibraryGame } from '@/emu'

/** The tiles, in order; `label` is the message key of the tile's name. */
const GROUPS: { id: string; consoles: ConsoleId[]; label: string }[] = [
  { id: 'gba', consoles: ['gba'], label: 'console.gba' },
  { id: 'nds', consoles: ['nds'], label: 'console.nds' },
  { id: '3ds', consoles: ['3ds'], label: 'console.3ds' },
  { id: 'gb', consoles: ['gb', 'gbc'], label: 'library.added.gbAndColor' },
]

export interface ConsoleCount {
  id: string
  /** The message key of its name. */
  label: string
  count: number
}

export type PerConsole = Partial<Record<ConsoleId, number>>

/** How many games of each console a list holds. */
export function countByConsole(games: Pick<LibraryGame, 'console'>[]): PerConsole {
  const per: PerConsole = {}
  for (const g of games) per[g.console] = (per[g.console] ?? 0) + 1
  return per
}

/** One count per tile; with `hideEmpty`, only the tiles that have games. */
export function consoleCounts(
  perConsole: PerConsole,
  { hideEmpty = false }: { hideEmpty?: boolean } = {},
): ConsoleCount[] {
  const counts = GROUPS.map(({ id, consoles, label }) => ({
    id,
    label,
    count: consoles.reduce((n, c) => n + (perConsole[c] ?? 0), 0),
  }))
  return hideEmpty ? counts.filter(c => c.count > 0) : counts
}
