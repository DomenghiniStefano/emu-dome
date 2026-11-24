// What the search everywhere (X01) searches, from what the app already knows, never listed here:
// - actions: the controller's shortcuts (`emu.controls.bindings`, in `Hotkey::ALL` order, the
//   reserved ones left out) with their `controls.hotkey.*` names and their combo, plus fullscreen
//   (F11, the keyboard's own); whether each can run here is searchContext.ts' (outside a game
//   only fullscreen, X05), the reason shown on the disabled ones;
// - settings: the schema (`settings_schema`), the rows a page shows (phase, `showIf` with the
//   values of «Tutti i giochi»), their names and pages in the current language and the
//   catalogue's synonyms, their value at «Tutti i giochi» on the right;
// - games: the library the interface already holds (hidden games left out).
// Where the search is (the shell, or the quick menu over an open game) comes in as `context`: a
// setting the quick menu does not show, or the game being played, is disabled with the reason.
// Loaded when the palette opens (`load`); the matching itself is search.ts's.
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type ComboRow,
  type LibraryGame,
  type PadFamily,
  type Pos,
  type SettingEntry,
  type SettingsSchema,
} from '@/emu'
import { settingValueText } from '@/components/details/gameOverrides'
import { rowVisible, shownOn } from '@/components/settings/settingsPage'
import { useSettingsValues } from '@/components/settings/useSettingsValues'
import { useKeyboardKeys } from '@/composables/useKeyboardKeys'
import { useErrorSink } from '@/composables/useToasts'
import { playTimeLabel } from '@/i18n/labels'
import { useSay, type Message } from '@/i18n/message'
import { comboText, hotkeyName, keyName } from '@/lib/controls'
import type { KeyAction } from '@/lib/keyboard'
import { KEYBOARD_SHORTCUTS } from '@/lib/keyboard'
import { library } from '@/lib/library'
import { GLOBAL, keyWords, loadSchema, msg } from '@/lib/settingsModel'
import { search, type Candidate, type Results } from './search'
import { actionReason, gameReason, settingReason, type SearchContext } from './searchContext'

/** An action: its name, its shortcut, and why it cannot run here (null: it can). */
export interface ActionItem {
  kind: 'action'
  id: KeyAction
  name: string
  shortcut: string | null
  reason: string | null
}

/** A setting: its name, its page («Impostazioni · Video»), its value at «Tutti i giochi», and
 * why it cannot be opened here (null: it can). */
export interface SettingItem {
  kind: 'setting'
  entry: SettingEntry
  name: string
  where: string
  value: string
  reason: string | null
}

/** A game of the library: its title, console and play time, and why it cannot be chosen here. */
export interface GameItem {
  kind: 'game'
  game: LibraryGame
  name: string
  where: string
  reason: string | null
}

export type SearchItem = ActionItem | SettingItem | GameItem
export type SearchResults = Results<ActionItem, SettingItem, GameItem>

/** The keyboard's own action, listed after the controller's shortcuts. */
const FULLSCREEN: KeyAction = 'fullscreen'

/** `context`: where the search is now, read reactively (the shell, or the quick menu). */
export function useSearchSources(context: () => SearchContext) {
  const { t, te } = useI18n()
  const say = useSay()
  const failWith = useErrorSink()
  const said = (m: Message | null) => (m ? say(m) : null)
  /** F11 beside fullscreen: on the phone only once a physical keyboard was used (#118). */
  const keyboardKeys = useKeyboardKeys()

  const schema = ref<SettingsSchema | null>(null)
  const byKey = computed(() => new Map((schema.value?.entries ?? []).map(e => [e.key, e])))
  const store = useSettingsValues(schema, byKey, ref(GLOBAL), failWith)
  const combos = ref<ComboRow[]>([])
  const modifier = ref<Pos>('l3')
  /** The combos are named as on the controller connected first («L3 + Y» on an Xbox pad), else
   * by position on a generic one. */
  const family = ref<PadFamily>('generic')

  /** Reads what may have changed since the last opening: the values and the shortcuts. */
  async function load() {
    try {
      schema.value = await loadSchema()
      const [pads] = await Promise.all([emu.controls.pads(), store.loadAll()])
      family.value = pads[0]?.family ?? 'generic'
      const bindings = await emu.controls.bindings(family.value, GLOBAL)
      combos.value = bindings.combos
      modifier.value = bindings.modifier
    } catch (e) {
      failWith(e)
    }
  }

  const actions = computed<ActionItem[]>(() => {
    const here = context()
    const fromPad: ActionItem[] = combos.value
      .filter(c => c.status !== 'later')
      .map(c => ({
        kind: 'action',
        id: c.hotkey,
        name: hotkeyName(c.hotkey),
        shortcut: c.pos ? comboText(family.value, modifier.value, c.pos) : null,
        reason: said(actionReason(c.hotkey, here)),
      }))
    const f11 = KEYBOARD_SHORTCUTS.find(k => k.action === FULLSCREEN)
    return [
      ...fromPad,
      {
        kind: 'action',
        id: FULLSCREEN,
        name: hotkeyName(FULLSCREEN),
        shortcut: f11 && keyboardKeys.value ? keyName(f11.code) : null,
        reason: said(actionReason(FULLSCREEN, here)),
      },
    ]
  })

  const settings = computed<SettingItem[]>(() => {
    const s = schema.value
    if (!s) return []
    const here = context()
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
            reason: said(settingReason(e.key, here)),
          }
        })
    })
  })

  const games = computed<GameItem[]>(() => {
    const here = context()
    return library.games
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
        reason: said(gameReason(g.id, here)),
      }))
  })

  /** The games played last, newest first: what the palette shows before anything is typed. */
  const recentGames = computed(() =>
    games.value
      .filter(g => g.game.lastPlayed !== null)
      .sort((a, b) => (b.game.lastPlayed ?? 0) - (a.game.lastPlayed ?? 0))
      .slice(0, 5),
  )

  const settingCandidate = (s: SettingItem): Candidate => ({
    name: s.name,
    words: `${s.where} ${t(msg.desc(s.entry.key))} ${keyWords(
      s.entry.key,
      k => te(k),
      k => t(k),
    )} ${s.entry.key}`,
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
