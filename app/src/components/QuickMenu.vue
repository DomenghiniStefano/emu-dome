<script setup lang="ts">
// The quick menu (boards «G07–G11 · Desktop · Menu rapido» and «G07–G11 · Telefono · Menu
// rapido», 17/11 «Prima ciò che usi più spesso»): no tabs. The main screen shows, top to bottom,
// the quick saves (G07), the speed and three rows that open a sub-page each: «Schermi e
// immagine» (G09), «Controlli» (G10), «Strumenti» (G11); «Altri slot e salvataggi» (G08) opens
// the slots that do not fit. Desktop: a full-height glass panel on the right. Phone in portrait:
// a bottom sheet (phone landscape and TV mode come with their own boards, #14 / #30).
// The game is paused. Settings changed here are «in prova» (Session level): live, listed in
// the bar at the bottom, saved for this game or its console, or undone; closing the game
// drops them.
//
// Controller / keyboard: opening the menu focuses the selected slot; B / Esc go back one level
// (App.vue sends Esc here as a «back» step), and from the main screen resume the game.
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { I18nT, useI18n } from 'vue-i18n'
import {
  ChevronLeft,
  ChevronRight,
  Clock,
  FastForward,
  Gamepad2,
  Play,
  RotateCcw,
  Rows2,
  Save,
  Wrench,
} from '@lucide/vue'
import {
  emu,
  type ComboRow,
  type GameInfo,
  type HotkeyId,
  type Pos,
  type SettingEntry,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
  type SlotInfo,
  type Speed,
  type VideoMode,
} from '../emu'
import { errorText, playTimeLabel, speedLabel } from '../i18n/labels'
import { comboText, hotkeyName, posName } from '../controls'
import { useNavScope } from '../nav'
import { isShown, loadSchema, msg } from '../settingsModel'
import SettingRowView from './SettingRow.vue'

const props = defineProps<{
  game: GameInfo
  speed: Speed
  videoMode: VideoMode
  nativeAvailable: boolean
  /** Unix ms the game started at («stai giocando da …»). */
  startedAt: number
  /** Name of the controller in use, null with none. */
  controller: string | null
  /** The slot the controller shortcuts save to and load from: selected when the menu opens. */
  currentSlot: number
  /** A quick save was loaded in this game: «Annulla il caricamento» can go back (`saves.undo`). */
  canUndoLoad?: boolean
}>()
const emit = defineEmits<{
  videoMode: [VideoMode]
  resume: []
  speed: [Speed]
  save: [number]
  load: [number]
  undoLoad: []
  reset: []
  close: []
  error: [string]
}>()

const { t, locale } = useI18n()

// ---- Main screen and sub-pages ----
type View = 'home' | 'slots' | 'screens' | 'controls' | 'shortcuts' | 'tools'
const view = ref<View>('home')
/** The row a sub-page was opened from, focused again on the way back. */
let opener: HTMLElement | null = null

const root = ref<HTMLElement | null>(null)
const body = ref<HTMLElement | null>(null)

function open(v: View, e: Event) {
  if (view.value === 'home')
    opener = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
  view.value = v
  void nextTick(() => {
    // The page's first control, else its back button.
    const first =
      body.value?.querySelector<HTMLElement>('[data-sub-initial]:not(:disabled)') ??
      body.value?.querySelector<HTMLElement>('.back')
    first?.focus()
  })
}
function back() {
  if (view.value === 'home') return emit('resume')
  if (view.value === 'shortcuts') {
    view.value = 'controls'
    void nextTick(() => body.value?.querySelector<HTMLElement>('[data-shortcuts]')?.focus())
    return
  }
  view.value = 'home'
  void loadSlots()
  void nextTick(() => opener?.focus())
}

useNavScope(root, { back, autofocus: true })

const playing = computed(() =>
  t('qm.playing', {
    console: t(`console.${props.game.consoleId}`),
    time: playTimeLabel(Math.max(60, (Date.now() - props.startedAt) / 1000)),
  }),
)

const SUB_TITLES: Record<Exclude<View, 'home'>, string> = {
  slots: 'qm.more',
  screens: 'qm.row.screens.name',
  controls: 'qm.row.controls.name',
  shortcuts: 'qm.shortcuts',
  tools: 'qm.row.tools.name',
}
const subTitle = computed(() => (view.value === 'home' ? '' : t(SUB_TITLES[view.value])))

// ---- Session settings («in prova») ----
const schema = ref<SettingsSchema | null>(null)
const session = ref<Record<string, SettingRow>>({})
const SESSION = { level: 'session' } as const
const PAGES = ['video', 'fx', 'speed', 'saves', 'touch']

async function loadSession() {
  try {
    const rows = (await Promise.all(PAGES.map(p => emu.settings.page(p, SESSION)))).flat()
    session.value = Object.fromEntries(rows.map(r => [r.key, r]))
  } catch (e) {
    emit('error', errorText(e))
  }
}
const entry = (key: string): SettingEntry | undefined =>
  schema.value?.entries.find(e => e.key === key)
const value = (key: string): SettingValue | null =>
  session.value[key]?.value ?? entry(key)?.default ?? null

async function setSession(key: string, v: SettingValue) {
  try {
    await emu.settings.set(key, SESSION, v)
  } catch (e) {
    emit('error', errorText(e))
  }
}

/** The changes «in prova», by value or name («Affiancati, Schermo LCD»). */
const trial = computed(() =>
  Object.values(session.value)
    .filter(r => r.source === 'session')
    .map(r => {
      const e = entry(r.key)
      if (e?.kind.type === 'choice' && typeof r.value === 'string')
        return t(msg.option(r.key, r.value))
      return t(msg.name(r.key))
    }),
)
async function commit(target: 'game' | 'console') {
  try {
    await emu.settings.commitSession(target)
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function discard() {
  try {
    await emu.settings.discardSession()
  } catch (e) {
    emit('error', errorText(e))
  }
}

// Setting rows each sub-page shows at the Session level (those the schema shows here:
// `touch.topac` is phone only and arrives in Phase 4).
const ROWS: Partial<Record<View, string[]>> = {
  screens: ['video.scale', 'video.filter', 'fx.fxpick'],
  controls: ['touch.topac'],
}
const rowsOf = (v: View) =>
  (ROWS[v] ?? [])
    .map(key => entry(key))
    .filter((e): e is SettingEntry => !!e && !!schema.value && isShown(e, schema.value))

// ---- Phone in portrait: the bottom sheet ----
const portraitQuery = window.matchMedia('(orientation: portrait)')
const portrait = ref(portraitQuery.matches)
const onOrientation = (e: MediaQueryListEvent) => (portrait.value = e.matches)
portraitQuery.addEventListener('change', onOrientation)
const sheet = computed(() => schema.value?.device === 'phone' && portrait.value)

// ---- Speed: the speeds of `speed.splist`, fast forward `speed.ffval`, slow motion `speed.slow` ----
const toSpeed = (id: string): Speed =>
  id === 'max' ? { kind: 'max' } : { kind: 'factor', value: Number(id) }
const NORMAL: Speed = { kind: 'factor', value: 1 }
const speeds = computed<Speed[]>(() => {
  const list = value('speed.splist')
  const ids = Array.isArray(list) ? list : ['0.5', '1', '2', '4', 'max']
  return ids.map(toSpeed)
})
const isSelected = (s: Speed) =>
  s.kind === props.speed.kind &&
  (s.kind === 'max' || (props.speed.kind === 'factor' && s.value === props.speed.value))
const speedKey = (s: Speed) => (s.kind === 'max' ? 'max' : String(s.value))
const ffSpeed = computed(() => toSpeed(String(value('speed.ffval') ?? '4')))
const slowSpeed = computed(() => toSpeed(String(value('speed.slow') ?? '0.5')))
const ffOn = computed(() => isSelected(ffSpeed.value))
const slowOn = computed(() => isSelected(slowSpeed.value))
const atNormal = computed(() => isSelected(NORMAL))
/** A toggle turned on goes to its speed; turned off, back to 1×. */
function toggleSpeed(s: Speed, on: boolean) {
  emit('speed', on ? NORMAL : s)
}

// ---- Shortcut hints, from the controller bindings («L3 + R1 salva») ----
const combos = ref<ComboRow[]>([])
const modifier = ref<Pos>('l3')
async function loadBindings() {
  try {
    const b = await emu.controls.bindings('generic', SESSION)
    combos.value = b.combos
    modifier.value = b.modifier
  } catch (e) {
    emit('error', errorText(e))
  }
}
const comboPos = (h: HotkeyId) => combos.value.find(r => r.hotkey === h)?.pos ?? null
const DPAD: Pos[] = ['up', 'down', 'left', 'right']
/** A button as the hints write it: the d-pad short («giù», «←»), the rest by name. */
const shortPos = (p: Pos) => (DPAD.includes(p) ? t(`qm.dir.${p}`) : posName('generic', p, false))
/** «L3 + R1», «L3 + ← →»; null when a shortcut has no button. */
function chip(...pos: Array<Pos | null>): string | null {
  const held = pos.filter((p): p is Pos => p !== null)
  if (held.length === 0 || held.length !== pos.length) return null
  const button =
    held.length === 2
      ? t('qm.hintPair', { a: shortPos(held[0]!), b: shortPos(held[1]!) })
      : shortPos(held[0]!)
  return t('controls.combo', { modifier: posName('generic', modifier.value, false), button })
}
interface Hint {
  chip: string
  label: string
}
const hints = (list: Array<[string | null, string]>): Hint[] =>
  list.flatMap(([c, label]) => (c === null ? [] : [{ chip: c, label: t(label) }]))
const saveHints = computed(() =>
  hints([
    [chip(comboPos('save')), 'qm.hint.save'],
    [chip(comboPos('load')), 'qm.hint.load'],
    [chip(comboPos('prev_slot'), comboPos('next_slot')), 'qm.hint.slot'],
  ]),
)
const speedHints = computed(() =>
  hints([
    [chip(comboPos('ff_hold')), 'qm.hint.ffHold'],
    [chip(comboPos('slow')), 'qm.hint.slow'],
  ]),
)
const boundCombos = computed(() => combos.value.filter(r => r.pos !== null).length)
/** The menus' back button: B on every controller (`pad.confirm` only moves where A is). */
const backButton = computed(() => t('controls.button.b'))

// ---- Quick saves (G07): the auto quick save and `saves.slots` slots ----
const slots = ref<SlotInfo[]>([])
const chosen = ref(props.currentSlot)
async function loadSlots() {
  const count = Number(value('saves.slots') ?? 10) || 10
  try {
    slots.value = await emu.listSlots(count)
  } catch (e) {
    emit('error', errorText(e))
  }
}
/** Tiles on the main screen: Automatico + slots 1–7 on desktop, three on the phone. */
const shownCount = computed(() => (sheet.value ? 3 : 8))
const mainSlots = computed(() => slots.value.slice(0, shownCount.value))
const moreSlots = computed(() => slots.value.slice(shownCount.value))
const chosenInfo = computed(() => slots.value.find(s => s.slot === chosen.value))
const chosenEmpty = computed(() => !chosenInfo.value || chosenInfo.value.savedAt === null)
function slotWhen(s: SlotInfo): string {
  if (s.savedAt === null) return t('qm.empty')
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'short', timeStyle: 'short' }).format(
    s.savedAt,
  )
}
const slotName = (slot: number) => (slot === 0 ? t('qm.auto') : t('qm.slot', { slot }))
const saveLabel = computed(() =>
  chosen.value === 0 ? t('qm.saveAuto') : t('qm.saveSlot', { slot: chosen.value }),
)
const loadLabel = computed(() => {
  if (chosenEmpty.value)
    return chosen.value === 0 ? t('qm.emptyAuto') : t('qm.emptySlot', { slot: chosen.value })
  return chosen.value === 0 ? t('qm.loadAuto') : t('qm.loadSlot', { slot: chosen.value })
})
const autoEvery = computed(() => {
  const id = value('saves.autoq')
  return typeof id === 'string' ? t(msg.option('saves.autoq', id)) : ''
})

/** Opening the menu focuses the selected slot (or «Altri slot» when it is not on screen). */
async function focusChosen() {
  await nextTick()
  const target =
    root.value?.querySelector<HTMLElement>('.slot.on') ??
    root.value?.querySelector<HTMLElement>('.more-link')
  target?.focus({ preventScroll: true })
}

/** The three rows of the main screen. */
const MORE = computed(() => [
  { view: 'screens' as const, icon: Rows2, sub: t('qm.row.screens.sub') },
  {
    view: 'controls' as const,
    icon: Gamepad2,
    sub: sheet.value ? t('qm.row.controls.subTouch') : t('qm.row.controls.sub'),
  },
  { view: 'tools' as const, icon: Wrench, sub: t('qm.row.tools.sub') },
])

let unlistenSettings: (() => void) | null = null
let unlistenEvents: (() => void) | null = null
onMounted(async () => {
  schema.value = await loadSchema()
  await loadSession()
  await loadSlots()
  void focusChosen()
  await loadBindings()
  unlistenSettings = await emu.settings.onChanged(() => void loadSession())
  // A save done here or with a shortcut: refresh the dates and thumbnails.
  unlistenEvents = await emu.onEvent(ev => {
    if (ev.type === 'stateSaved') void loadSlots()
  })
})
onBeforeUnmount(() => {
  unlistenSettings?.()
  unlistenEvents?.()
  portraitQuery.removeEventListener('change', onOrientation)
})
</script>

<template>
  <aside ref="root" class="menu" :class="{ sheet }" :aria-label="t('qm.label')">
    <span v-if="sheet" class="grip" aria-hidden="true"></span>
    <header class="head">
      <div class="title">{{ game.title }}</div>
      <div class="muted small">{{ playing }}</div>
    </header>

    <div ref="body" class="body">
      <template v-if="view === 'home'">
        <!-- G07: quick saves -->
        <section class="block" :aria-label="t('qm.savesTitle')">
          <div class="block-head">
            <span class="overline">{{ t('qm.savesTitle') }}</span>
            <button class="link more-link" @click="open('slots', $event)">
              {{ sheet ? t('qm.moreShort') : t('qm.more') }}
            </button>
          </div>
          <div class="slots" role="radiogroup" :aria-label="t('qm.slotsLabel')">
            <button
              v-for="s in mainSlots"
              :key="s.slot"
              class="slot"
              :class="{ on: s.slot === chosen }"
              role="radio"
              :aria-checked="s.slot === chosen"
              @click="chosen = s.slot"
            >
              <span class="thumb" :class="{ blank: s.savedAt === null }" aria-hidden="true">
                <img v-if="s.thumbUrl" :src="s.thumbUrl" alt="" />
              </span>
              <span class="slot-name">{{ slotName(s.slot) }}</span>
              <span class="slot-when num">{{ slotWhen(s) }}</span>
            </button>
          </div>
          <div class="pair">
            <button class="act" @click="emit('save', chosen)">
              <Save :size="18" :stroke-width="1.75" aria-hidden="true" />
              {{ saveLabel }}
            </button>
            <button class="act" :disabled="chosenEmpty" @click="emit('load', chosen)">
              <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
              {{ loadLabel }}
            </button>
          </div>
          <button
            v-if="canUndoLoad && value('saves.undo') !== false"
            class="btn undo"
            @click="emit('undoLoad')"
          >
            <RotateCcw :size="14" :stroke-width="1.75" aria-hidden="true" />
            {{ t('qm.undoLoad') }}
          </button>
          <div v-if="!sheet && saveHints.length" class="hints">
            <span v-for="h in saveHints" :key="h.label">
              <span class="chip">{{ h.chip }}</span> {{ h.label }}
            </span>
          </div>
        </section>

        <div class="sep"></div>

        <!-- Speed -->
        <section class="block" :aria-label="t('qm.speedTitle')">
          <div class="block-head">
            <span class="overline">{{ t('qm.speedTitle') }}</span>
            <span v-if="!sheet" class="muted small">
              {{ atNormal ? t('qm.speedNormal') : t('qm.speedNote') }}
            </span>
          </div>
          <div
            class="segmented"
            role="radiogroup"
            :aria-label="t('qm.speedTitle')"
            :style="{ gridTemplateColumns: `repeat(${speeds.length}, minmax(0, 1fr))` }"
          >
            <button
              v-for="s in speeds"
              :key="speedKey(s)"
              role="radio"
              :aria-checked="isSelected(s)"
              :class="{ on: isSelected(s) }"
              class="num"
              @click="emit('speed', s)"
            >
              {{ speedLabel(s) }}
            </button>
          </div>
          <div class="pair">
            <button
              class="act toggle"
              :class="{ on: ffOn }"
              :aria-pressed="ffOn"
              @click="toggleSpeed(ffSpeed, ffOn)"
            >
              <FastForward :size="16" fill="currentColor" :stroke-width="0" aria-hidden="true" />
              {{ t('qm.ff') }}
            </button>
            <button
              class="act toggle"
              :class="{ on: slowOn }"
              :aria-pressed="slowOn"
              @click="toggleSpeed(slowSpeed, slowOn)"
            >
              <Clock :size="16" :stroke-width="1.75" aria-hidden="true" />
              {{ t('qm.slow') }}
            </button>
          </div>
          <div v-if="!sheet && speedHints.length" class="hints">
            <span v-for="h in speedHints" :key="h.label">
              <span class="chip">{{ h.chip }}</span> {{ h.label }}
            </span>
          </div>
        </section>

        <div v-if="!sheet" class="sep"></div>

        <!-- G09–G11: one row each -->
        <nav class="rows" :aria-label="t('qm.moreLabel')">
          <button v-for="m in MORE" :key="m.view" class="more" @click="open(m.view, $event)">
            <span class="more-icon" aria-hidden="true">
              <component :is="m.icon" :size="18" :stroke-width="1.75" />
            </span>
            <span class="grow">
              <span class="more-name">{{ t(`qm.row.${m.view}.name`) }}</span>
              <span class="more-sub">{{ m.sub }}</span>
            </span>
            <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
          </button>
        </nav>
      </template>

      <!-- A sub-page: back one level (B / Esc), title, its content. -->
      <template v-else>
        <button class="back" @click="back">
          <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
          {{ view === 'shortcuts' ? t('qm.row.controls.name') : t('qm.label') }}
          <span class="chip">{{ backButton }}</span>
        </button>
        <div class="sub-title">{{ subTitle }}</div>

        <!-- G08: the slots that do not fit on the main screen -->
        <template v-if="view === 'slots'">
          <div
            v-if="moreSlots.length"
            class="slots"
            role="radiogroup"
            :aria-label="t('qm.slotsLabel')"
          >
            <button
              v-for="(s, i) in moreSlots"
              :key="s.slot"
              class="slot"
              :class="{ on: s.slot === chosen }"
              role="radio"
              :aria-checked="s.slot === chosen"
              :data-sub-initial="i === 0 ? '' : undefined"
              @click="chosen = s.slot"
            >
              <span class="thumb" :class="{ blank: s.savedAt === null }" aria-hidden="true">
                <img v-if="s.thumbUrl" :src="s.thumbUrl" alt="" />
              </span>
              <span class="slot-name">{{ slotName(s.slot) }}</span>
              <span class="slot-when num">{{ slotWhen(s) }}</span>
            </button>
          </div>
          <div class="pair">
            <button class="act" @click="emit('save', chosen)">
              <Save :size="18" :stroke-width="1.75" aria-hidden="true" />
              {{ saveLabel }}
            </button>
            <button class="act" :disabled="chosenEmpty" @click="emit('load', chosen)">
              <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
              {{ loadLabel }}
            </button>
          </div>
          <button
            v-if="canUndoLoad && value('saves.undo') !== false"
            class="btn undo"
            @click="emit('undoLoad')"
          >
            <RotateCcw :size="14" :stroke-width="1.75" aria-hidden="true" />
            {{ t('qm.undoLoad') }}
          </button>
          <div class="muted small">{{ t('qm.autoEvery', { every: autoEvery }) }}</div>
          <I18nT v-if="!sheet" keypath="qm.keys" tag="div" class="muted small">
            <template #save><kbd>F1</kbd></template>
            <template #load><kbd>F4</kbd></template>
          </I18nT>
        </template>

        <!-- G09: screens and picture (DS layouts arrive in Phase 2) -->
        <div v-else-if="view === 'screens'" class="list">
          <template v-if="schema">
            <SettingRowView
              v-for="e in rowsOf('screens')"
              :key="e.key"
              :entry="e"
              :row="session[e.key]"
              :schema="schema"
              level="session"
              :level-name="() => ''"
              :scoped="false"
              compact
              @set="v => setSession(e.key, v)"
              @error="text => emit('error', text)"
            />
          </template>
          <div class="line">
            <span class="grow">
              <span class="line-name">{{ t('settings.video.path.name') }}</span>
              <span v-if="!nativeAvailable" class="line-sub">{{ t('qm.noNative') }}</span>
            </span>
            <div
              class="segmented small-seg"
              role="radiogroup"
              :aria-label="t('settings.video.path.name')"
            >
              <button
                role="radio"
                :aria-checked="videoMode === 'native'"
                :class="{ on: videoMode === 'native' }"
                :disabled="!nativeAvailable"
                @click="emit('videoMode', 'native')"
              >
                {{ t('settings.video.path.option.accelerated') }}
              </button>
              <button
                role="radio"
                :aria-checked="videoMode === 'webview'"
                :class="{ on: videoMode === 'webview' }"
                @click="emit('videoMode', 'webview')"
              >
                {{ t('settings.video.path.option.compatible') }}
              </button>
            </div>
          </div>
        </div>

        <!-- G10: controls -->
        <div v-else-if="view === 'controls'" class="list">
          <div class="line">
            <span class="grow">
              <span class="line-name">{{ t('qm.controller') }}</span>
              <span class="line-sub">{{ controller ?? t('qm.noController') }}</span>
            </span>
          </div>
          <template v-if="schema">
            <SettingRowView
              v-for="e in rowsOf('controls')"
              :key="e.key"
              :entry="e"
              :row="session[e.key]"
              :schema="schema"
              level="session"
              :level-name="() => ''"
              :scoped="false"
              compact
              @set="v => setSession(e.key, v)"
              @error="text => emit('error', text)"
            />
          </template>
          <button class="line" disabled :title="t('qm.notYet')">
            <span class="grow line-name">{{ t('qm.remap') }}</span>
            <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
          </button>
          <button class="line" data-shortcuts data-sub-initial @click="open('shortcuts', $event)">
            <span class="grow">
              <span class="line-name">{{ t('qm.shortcuts') }}</span>
              <span class="line-sub">
                {{ t('qm.shortcutsHint', { modifier: posName('generic', modifier, false) }) }}
              </span>
            </span>
            <span class="muted num">{{ boundCombos }}</span>
            <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
          </button>
        </div>

        <!-- G10: the controller shortcuts, read-only (they are edited in Controls, C03) -->
        <div v-else-if="view === 'shortcuts'" class="list">
          <div
            v-for="(r, i) in combos"
            :key="r.hotkey"
            class="line"
            tabindex="0"
            :data-sub-initial="i === 0 ? '' : undefined"
          >
            <span class="grow line-name">{{ hotkeyName(r.hotkey) }}</span>
            <span class="muted">{{ comboText('generic', modifier, r.pos) }}</span>
          </div>
        </div>

        <!-- G11: tools -->
        <div v-else class="list">
          <button class="line" disabled :title="t('qm.notYet')">
            <span class="grow line-name">{{ t('qm.screenshot') }}</span>
          </button>
          <button class="line" data-sub-initial @click="emit('reset')">
            <span class="grow">
              <span class="line-name">{{ t('qm.reset') }}</span>
              <span class="line-sub">{{ t('qm.resetHint') }}</span>
            </span>
          </button>
          <button class="line" @click="emit('close')">
            <span class="grow">
              <span class="line-name">{{ t('qm.close') }}</span>
              <span v-if="game.autosaveOnExit" class="line-sub">{{ t('qm.closeHint') }}</span>
            </span>
          </button>
        </div>
      </template>
    </div>

    <footer class="foot">
      <div v-if="trial.length" class="trial" role="status">
        <div>
          <div class="trial-title">
            {{
              trial.length === 1
                ? t('qm.trialOne', { names: trial.join(', ') })
                : t('qm.trial', { n: trial.length, names: trial.join(', ') })
            }}
          </div>
          <div class="muted small">{{ t('qm.trialHint') }}</div>
        </div>
        <div class="row-actions">
          <button class="btn" @click="discard">{{ t('qm.undo') }}</button>
          <button class="btn" @click="commit('game')">{{ t('qm.saveGame') }}</button>
        </div>
        <button class="link underline" @click="commit('console')">
          {{ t('qm.saveConsole', { console: t(`console.${game.consoleId}`) }) }}
        </button>
      </div>
      <button class="btn btn-primary resume" @click="emit('resume')">
        <Play v-if="!sheet" :size="16" fill="currentColor" :stroke-width="0" aria-hidden="true" />
        {{ sheet ? t('qm.resumeShort') : t('qm.resume') }}
        <span v-if="!sheet" class="resume-keys">
          {{ t('qm.resumeKeys', { esc: t('flow.hintEsc'), back: backButton }) }}
        </span>
      </button>
    </footer>
  </aside>
</template>

<style scoped>
/* Desktop: a glass panel the full height of the window, on the right. */
.menu {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 460px;
  max-width: 100vw;
  padding: 20px 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: var(--glass-thick);
  border-left: 1px solid var(--line);
  box-shadow:
    inset 1px 0 0 var(--glass-edge),
    -24px 0 64px #0000008c;
  backdrop-filter: blur(24px) saturate(140%);
  z-index: 20;
}
.head .title {
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
.body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.block {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.block-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.sep {
  flex: 0 0 auto;
  height: 1px;
  background: var(--line);
}
.link {
  border: 0;
  background: none;
  padding: 0;
  color: var(--accent-text);
  font-weight: 500;
}
.link.underline {
  align-self: flex-start;
  text-decoration: underline;
}
.slots {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}
.slot {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px;
  border-radius: var(--radius-10);
  text-align: left;
  background: var(--surface);
  border: 1px solid var(--line);
}
.slot.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
}
.thumb {
  display: block;
  width: 100%;
  aspect-ratio: 3 / 2;
  border-radius: var(--radius-6);
  overflow: hidden;
  background: var(--surface-raised);
}
.thumb.blank {
  border: 1px dashed var(--line-strong);
}
.thumb img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}
.slot-name {
  font-weight: 600;
  font-size: 12px;
  line-height: 16px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.slot-when {
  font-size: 11px;
  line-height: 14px;
  color: var(--ink-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.act {
  height: 40px;
  min-width: 0;
  padding: 0 14px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--surface-raised);
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.act:hover:not(:disabled) {
  background: var(--surface-hover);
}
.act:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.act > svg {
  flex: 0 0 auto;
}
.toggle {
  font-weight: 500;
}
.toggle.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
  color: var(--accent-text);
  font-weight: 600;
}
/* «Annulla il caricamento»: a secondary action under the save / load pair. */
.btn.undo {
  align-self: flex-start;
}
.hints {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  font-size: 12px;
  line-height: 18px;
  color: var(--ink-muted);
}
.chip {
  display: inline-flex;
  align-items: center;
  height: 18px;
  padding: 0 5px;
  border-radius: 4px;
  border: 1px solid var(--line-strong);
  font-size: 11px;
  font-weight: 600;
  color: var(--ink);
}
.segmented {
  display: grid;
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-10);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.segmented button {
  height: 36px;
  min-width: 0;
  padding: 0 4px;
  border: 0;
  border-radius: 8px;
  background: none;
  font-weight: 600;
  white-space: nowrap;
}
.segmented button.on {
  background: var(--accent-soft);
  color: var(--accent-text);
}
.segmented button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.small-seg {
  flex: 0 0 auto;
  grid-auto-flow: column;
}
.small-seg button {
  height: 28px;
  padding: 0 10px;
  font-weight: 500;
}
.rows,
.list {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
}
.more + .more,
.list > * + * {
  border-top: 1px solid var(--line);
}
.more,
.line {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border: 0;
  background: none;
  text-align: left;
  color: var(--ink);
}
.more {
  min-height: 56px;
}
.line {
  min-height: 52px;
}
button.more:hover,
button.line:hover:not(:disabled) {
  background: var(--surface-hover);
}
.line:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.more > svg,
.line > svg {
  flex: 0 0 auto;
}
.more-icon {
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--surface-raised);
  display: grid;
  place-items: center;
  color: var(--ink-muted);
}
.more-name,
.line-name {
  display: block;
  font-weight: 600;
}
.line-name {
  font-weight: 500;
}
.more-sub,
.line-sub {
  display: block;
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.back {
  align-self: flex-start;
  height: 32px;
  padding: 0 10px 0 4px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--ink-muted);
  font-weight: 500;
}
.back:hover {
  background: var(--surface-hover);
}
.back .chip {
  margin-left: 4px;
}
.sub-title {
  font-size: 17px;
  line-height: 22px;
  font-weight: 600;
}
.grow {
  flex: 1;
  min-width: 0;
}
.foot {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.trial {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--accent-text);
}
.trial-title {
  font-weight: 600;
}
.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.row-actions .btn {
  flex: 1 1 100%;
}
.resume {
  width: 100%;
  height: 44px;
  border-radius: 8px;
  gap: 8px;
}
.resume-keys {
  font-weight: 400;
  opacity: 0.8;
}
.muted {
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}

/* Phone in portrait: a bottom sheet under the game (board «G07–G11 · Telefono · Menu rapido»). */
.menu.sheet {
  top: 64px;
  left: 0;
  width: auto;
  padding: 10px 16px 24px;
  gap: 14px;
  border-left: 0;
  border-top: 1px solid var(--line);
  border-radius: var(--radius-16) var(--radius-16) 0 0;
  box-shadow: 0 -24px 64px #0000008c;
  font-size: 15px;
  line-height: 22px;
}
.grip {
  flex: 0 0 auto;
  align-self: center;
  width: 40px;
  height: 4px;
  border-radius: 999px;
  background: var(--line-strong);
}
.sheet .body {
  gap: 14px;
}
.sheet .head .title {
  font-size: 17px;
  line-height: 22px;
}
.sheet .head .small {
  font-size: 13px;
}
.sheet .overline {
  font-size: 12px;
}
.sheet .more-link {
  font-size: 13px;
}
.sheet .slots {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.sheet .slot {
  border-radius: 12px;
}
.sheet .slot.on {
  border-width: 2px;
}
.sheet .slot-name {
  font-size: 13px;
}
.sheet .slot-when {
  font-size: 12px;
}
.sheet .act {
  height: 44px;
  border-radius: 12px;
  font-size: 15px;
}
.sheet .segmented {
  border-radius: 12px;
}
.sheet .segmented button {
  height: 40px;
  border-radius: 10px;
  font-size: 14px;
}
.sheet .rows,
.sheet .list {
  border-radius: 14px;
}
.sheet .more {
  min-height: 52px;
  padding: 4px 14px;
}
.sheet .more-icon {
  display: none;
}
.sheet .more-sub,
.sheet .line-sub {
  font-size: 13px;
}
.sheet .resume {
  height: 52px;
  border-radius: 12px;
  font-size: 16px;
}
</style>
