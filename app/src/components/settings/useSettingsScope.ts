// The level the Settings section edits (the scope bar): «Tutti i giochi», «Giochi <console> ▾»
// (the consoles with games in the library, starting on the open game's console, else the last
// one played: design, #41) and, when the section opened from a game's details (L06), «Solo questo
// gioco» for that game, chosen on arrival.
import { computed, ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type ConsoleId,
  type SettingEntry,
  type SettingLevel,
  type SettingsScope,
} from '@/emu'
import type { GameFocus } from '@/lib/gameDetails'
import { library } from '@/lib/library'
import { pageLevels } from '@/lib/settingsModel'
import {
  effectiveLevel,
  offersGameLevel,
  pageConsole,
  pageConsoles,
  scopeAt,
  scopeConsoles,
  scopeLevels,
  stepLevel,
  type EditLevel,
} from './settingsPage'

export function useSettingsScope(pageEntries: Ref<SettingEntry[]>, gameFocus: GameFocus | null) {
  const { t } = useI18n()

  const level = ref<EditLevel>(gameFocus ? 'game' : 'global')
  /** The console chosen (the game's, the open game's, or picked in the menu); null: none yet. */
  const chosenConsole = ref<ConsoleId | null>(null)
  /** The open game's console, when a game is open. */
  const openConsole = ref<ConsoleId | null>(null)

  /** The consoles of this page's menu: with games, and among those its rows are for (#176). */
  const consoles = computed(() =>
    scopeConsoles(
      library.games,
      [openConsole.value, gameFocus?.scope.console],
      pageConsoles(pageEntries.value),
    ),
  )
  const consoleOptions = computed(() =>
    consoles.value.map(c => ({
      id: c,
      text: t('settingsUi.scope.console', { console: t(`console.${c}`) }),
    })),
  )
  /** The console the scope bar shows here. It follows the library too: a late answer picks
   * one, losing the chosen console's last game moves off it. */
  const contextConsole = computed(() =>
    pageConsole(chosenConsole.value, consoles.value, library.games),
  )

  async function findContextConsole() {
    const open = await emu.settings.gameScope().catch(() => null)
    openConsole.value = open && open.level === 'game' ? open.console : null
    chosenConsole.value = gameFocus?.scope.console ?? openConsole.value ?? null
  }

  /** A console picked in the scope menu: it also selects the console level. */
  function pickConsole(id: string) {
    const c = consoles.value.find(x => x === id)
    if (!c) return
    chosenConsole.value = c
    level.value = 'console'
  }

  const forGame = !!gameFocus
  /** The page has settings per console or per game: the scope bar is shown. */
  const scoped = computed(
    () =>
      pageLevels(pageEntries.value).has('console') || offersGameLevel(pageEntries.value, forGame),
  )
  const levels = computed(() => scopeLevels(pageEntries.value, contextConsole.value, forGame))
  /** The level in effect: the one chosen, when this page has it. */
  const effective = computed(() => effectiveLevel(levels.value, level.value))
  const scope = computed<SettingsScope>(() =>
    scopeAt(effective.value, contextConsole.value, gameFocus?.scope ?? null),
  )

  /** The words of the scope bar for a level. */
  function levelName(l: SettingLevel): string {
    if (l === 'console' && contextConsole.value)
      return t('settingsUi.scope.console', { console: t(`console.${contextConsole.value}`) })
    if (l === 'game') return t('controls.levelGame')
    return t('settingsUi.scope.global')
  }

  /** L1 / R1: the previous / next level of the scope bar; `false` on a page with one level, so
   * the step goes on to the shell (in TV mode it switches section). */
  function tab(dir: -1 | 1): boolean | void {
    const next = stepLevel(levels.value, effective.value, dir)
    if (!next) return false
    level.value = next
  }

  /** A recent change opens at the level it was made, when its console is offered. */
  function goTo(l: SettingLevel, console: ConsoleId | null) {
    if (l === 'console' && console && consoles.value.includes(console)) {
      chosenConsole.value = console
      level.value = 'console'
    } else level.value = 'global'
  }

  return {
    level,
    contextConsole,
    consoleOptions,
    findContextConsole,
    pickConsole,
    scoped,
    levels,
    effective,
    scope,
    levelName,
    tab,
    goTo,
  }
}
