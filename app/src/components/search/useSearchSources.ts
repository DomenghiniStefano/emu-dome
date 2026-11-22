// What the search everywhere (X01) searches, from what the app already knows, never listed here:
// - actions: the controller's shortcuts (`emu.controls.bindings`, in `Hotkey::ALL` order, the
//   reserved ones left out) with their `controls.hotkey.*` names and their combo, plus fullscreen
//   (F11, the keyboard's own); a shortcut acts on the open game, so outside a game it is shown
//   disabled with the reason (X05);
// - settings: the schema (`settings_schema`), the rows a page shows (phase, `showIf` with the
//   values of «Tutti i giochi»), their names and pages in the current language and the
//   catalogue's synonyms, their value at «Tutti i giochi» on the right;
// - games: the library the interface already holds (hidden games left out).
// Loaded when the palette opens (`load`); the matching itself is search.ts's.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type ComboRow,
  type LibraryGame,
  type Pos,
  type SettingEntry,
  type SettingsSchema,
} from '@/emu'
import { settingValueText } from '@/components/details/gameOverrides'
import { rowVisible, shownOn } from '@/components/settings/settingsPage'
import { useSettingsValues } from '@/components/settings/useSettingsValues'
import { useErrorSink } from '@/composables/useToasts'
import { playTimeLabel } from '@/i18n/labels'
import { comboText, hotkeyName, keyName } from '@/lib/controls'
import type { KeyAction } from '@/lib/keyboard'
import { KEYBOARD_SHORTCUTS } from '@/lib/keyboard'
import { library } from '@/lib/library'
import { GLOBAL, loadSchema, msg } from '@/lib/settingsModel'
import { search, type Candidate, type Results } from './search'

/** An action: its name, its shortcut, and why it cannot run here (null: it can). */
export interface ActionItem {
  kind: 'action'
  id: KeyAction
  name: string
  shortcut: string | null
  reason: string | null
}

/** A setting: its name, its page («Impostazioni · Video»), its value at «Tutti i giochi». */
export interface SettingItem {
  kind: 'setting'
  entry: SettingEntry
  name: string
  where: string
  value: string
}

/** A game of the library: its title, console and play time. */
export interface GameItem {
  kind: 'game'
  game: LibraryGame
  name: string
  where: string
}

export type SearchItem = ActionItem | SettingItem | GameItem
export type SearchResults = Results<ActionItem, SettingItem, GameItem>

/** The names of a generic pad: the shortcuts' combos read the same whatever is connected. */
const FAMILY = 'generic'
/** The only action that works without a game: the keyboard's fullscreen. */
const FULLSCREEN: KeyAction = 'fullscreen'

export function useSearchSources() {
  const { t, te } = useI18n()
  const failWith = useErrorSink()

  const schema = ref<SettingsSchema | null>(null)
  const byKey = computed(() => new Map((schema.value?.entries ?? []).map(e => [e.key, e])))
  const store = useSettingsValues(schema, byKey, ref(GLOBAL), failWith)
  const combos = ref<ComboRow[]>([])
  const modifier = ref<Pos>('l3')

  /** Reads what may have changed since the last opening: the values and the shortcuts. */
  async function load() {
    try {
      schema.value = await loadSchema()
      const [bindings] = await Promise.all([emu.controls.bindings(FAMILY, GLOBAL), store.loadAll()])
      combos.value = bindings.combos
      modifier.value = bindings.modifier
    } catch (e) {
      failWith(e)
    }
  }

  const actions = computed<ActionItem[]>(() => {
    const needsGame = t('search.needsGame')
    const fromPad: ActionItem[] = combos.value
      .filter(c => c.status !== 'later')
      .map(c => ({
        kind: 'action',
        id: c.hotkey,
        name: hotkeyName(c.hotkey),
        shortcut: c.pos ? comboText(FAMILY, modifier.value, c.pos) : null,
        reason: needsGame,
      }))
    const f11 = KEYBOARD_SHORTCUTS.find(k => k.action === FULLSCREEN)
    return [
      ...fromPad,
      {
        kind: 'action',
        id: FULLSCREEN,
        name: hotkeyName(FULLSCREEN),
        shortcut: f11 ? keyName(f11.code) : null,
        reason: null,
      },
    ]
  })

  const settings = computed<SettingItem[]>(() => {
    const s = schema.value
    if (!s) return []
    return s.pages.flatMap(page => {
      const rows = store.values[page]
      return shownOn(s, page)
        .filter(e => rowVisible(e, rows, byKey.value))
        .map(e => {
          const value = rows?.[e.key]?.value ?? e.default
          return {
            kind: 'setting' as const,
            entry: e,
            name: t(msg.name(e.key)),
            where: t('search.settingWhere', { page: t(msg.page(page)) }),
            value: value === null ? '' : settingValueText(e, value, { t, te }),
          }
        })
    })
  })

  const games = computed<GameItem[]>(() =>
    library.games
      .filter(g => !g.hidden)
      .map(g => ({
        kind: 'game',
        game: g,
        name: g.title,
        where:
          g.playTimeSecs > 0
            ? t('search.gameWhere', {
                console: t(`console.${g.console}`),
                time: playTimeLabel(g.playTimeSecs),
              })
            : t(`console.${g.console}`),
      })),
  )

  /** The games played last, newest first: what the palette shows before anything is typed. */
  const recentGames = computed(() =>
    games.value
      .filter(g => g.game.lastPlayed !== null)
      .sort((a, b) => (b.game.lastPlayed ?? 0) - (a.game.lastPlayed ?? 0))
      .slice(0, 5),
  )

  const settingCandidate = (s: SettingItem): Candidate => ({
    name: s.name,
    words: `${s.where} ${t(msg.desc(s.entry.key))} ${s.entry.key}`,
    synonyms: s.entry.synonyms,
  })

  /** The results for `query` (search.ts). */
  const resultsFor = (query: string): SearchResults =>
    search(query, {
      actions: actions.value,
      action: a => ({ name: a.name, words: a.shortcut ?? '' }),
      settings: settings.value,
      setting: settingCandidate,
      games: games.value,
      game: g => ({ name: g.name, words: g.where }),
    })

  return { load, resultsFor, recentGames }
}
