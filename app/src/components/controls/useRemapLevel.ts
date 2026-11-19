// C02's table at the level being edited: «Tutti i giochi», «Giochi <console>» or «Solo questo
// gioco» (the game opened from its details, else the one open, else the last played console),
// the rows resolved there (`emu.controls.bindings`), and `pad.remap` as stored or inherited
// there, the list every change starts from. Read again when the level or a `pad.*` setting
// changes.
import { computed, onMounted, ref, watch } from 'vue'
import { emu, type ConsoleId, type PadFamily, type RemapRow, type SettingsScope } from '../../emu'
import { remapList, remapWithout } from '../../controls'
import type { GameFocus } from '../../gameDetails'
import type { ErrorSink } from '../../i18n/labels'
import { library } from '../../library'
import { readSetting } from '../../settingsModel'
import { settingsUnder, useSettingsChanged } from '../../composables/useSettingsChanged'
import {
  RemapLevel,
  lastPlayedConsole,
  levelScope,
  remapLevels,
  type RemapLevel as Level,
} from './levels'
import type { BindingChanges } from './remapTable'

export function useRemapLevel(options: {
  family: () => PadFamily
  /** Opened from a game's details (L07): its «Solo questo gioco», chosen on arrival. */
  game: () => GameFocus | null | undefined
  onError: ErrorSink
}) {
  const level = ref<Level>(RemapLevel.Global)
  const contextConsole = ref<ConsoleId | null>(null)
  const gameScope = ref<SettingsScope | null>(null)

  async function findContext() {
    const game = options.game()
    if (game) {
      gameScope.value = game.scope
      contextConsole.value = game.scope.console
      level.value = RemapLevel.Game
      return
    }
    gameScope.value = await emu.settings.gameScope().catch(() => null)
    const open = gameScope.value
    contextConsole.value =
      open && open.level === 'game' ? open.console : lastPlayedConsole(library.games)
  }
  const levels = computed(() => remapLevels(contextConsole.value, gameScope.value))
  const scope = computed(() => levelScope(level.value, contextConsole.value, gameScope.value))

  const rows = ref<RemapRow[]>([])
  /** `pad.remap` as stored or inherited at this level. */
  const stored = ref<string[]>([])
  /** Whether this level has a `pad.remap` of its own. */
  const storedHere = ref(false)

  async function load() {
    try {
      const [b, row] = await Promise.all([
        emu.controls.bindings(options.family(), scope.value),
        readSetting('pad.remap', scope.value),
      ])
      rows.value = b.remap
      stored.value = Array.isArray(row?.value) ? row.value : []
      storedHere.value = row?.source === scope.value.level
    } catch (e) {
      options.onError(e)
    }
  }
  watch(scope, () => void load(), { deep: true })
  onMounted(async () => {
    await findContext()
    await load()
  })
  useSettingsChanged(settingsUnder('pad.'), () => void load())

  async function write(list: string[]) {
    try {
      if (list.length === 0 && level.value === RemapLevel.Global && storedHere.value)
        await emu.settings.reset('pad.remap', scope.value)
      else await emu.settings.set('pad.remap', scope.value, list)
    } catch (e) {
      options.onError(e)
    }
  }

  /** Writes these rows' new sources at this level. */
  const change = (changes: BindingChanges<RemapRow['button'], string>) =>
    write(remapList(stored.value, options.family(), rows.value, changes))
  /** «Ripristina i predefiniti»: this family's overrides gone from this level. */
  const restoreDefaults = () => write(remapWithout(stored.value, options.family()))

  return { level, contextConsole, levels, rows, change, restoreDefaults }
}
