<script setup lang="ts">
// The quick menu (boards «MenuRapido3» G07–G11, G16, G17 and «Tel-MenuRapido3», 18/11).
// Desktop: a glass panel in the centre, 640 px wide and as tall as its content; the game is
// paused behind it, dimmed. Top to bottom: quick saves (Automatico + 1–4, «Altri slot e
// salvataggi» = G08), speed (Normale / Avanti veloce / Rallentatore, «Imposta le velocità» = G16),
// tools, the rows «Schermi e immagine» (G09: the menu becomes a bar at the bottom and the game
// stays visible above it) and «Controlli» (G10), then «Chiudi il gioco» and «Riprendi il gioco».
// Phone in portrait: a bottom sheet under the game, same order for the thumb.
// Settings changed here are «in prova» (Session level): live, listed in the bar, saved for this
// game or its console, or undone; closing the game drops them.
//
// Controller / keyboard: opening the menu focuses the selected slot; B / Esc go back one level
// (App.vue sends Esc here as a «back» step), and from the main screen resume the game.
// Hints show only the input in use: Esc with mouse and keyboard, B with a controller, none on touch.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  ArchiveRestore,
  ArrowDownUp,
  Camera,
  ChevronLeft,
  ChevronRight,
  Gamepad2,
  Laptop,
  Maximize,
  Play,
  Power,
  RefreshCw,
  RotateCcw,
  Rows2,
  Save,
  VolumeX,
} from '@lucide/vue'
import {
  AUTO_SLOT,
  emu,
  type ComboRow,
  type GameInfo,
  type Pos,
  type SettingEntry,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
  type SlotInfo,
  type Speed,
  type VideoMode,
} from '../emu'
import { errorText } from '../i18n/labels'
import { comboText, hotkeyName, posName } from '../controls'
import { useNavScope } from '../nav'
import { inputMode } from '../inputMode'
import { lockReason, isShown, loadSchema, msg } from '../settingsModel'
import { FF_DEFAULT, NORMAL, SLOW_DEFAULT, sameSpeed, speedFromOption } from '../speeds'
import SettingRowView from './SettingRow.vue'

const props = defineProps<{
  game: GameInfo
  speed: Speed
  videoMode: VideoMode
  nativeAvailable: boolean
  /** Name of the controller in use, null with none. */
  controller: string | null
  /** The slot the controller shortcuts save to and load from: selected when the menu opens. */
  currentSlot: number
  /** A quick save was loaded in this game and can be undone (`saves.undo`): the slot, else null. */
  undoLoadSlot: number | null
  muted: boolean
  fullscreen: boolean
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
  mute: [boolean]
  fullscreen: []
  screenshot: []
  /** G09: px at the bottom of the window the bar covers (0 = none); the game shrinks above. */
  inset: [number]
  error: [string]
}>()

const { t, locale } = useI18n()

// ---- Main screen and sub-pages ----
type View = 'home' | 'slots' | 'screens' | 'controls' | 'shortcuts' | 'speed' | 'restart'
const view = ref<View>('home')
const CONTROL = 'button, input, select, [tabindex]'
/** The control a sub-page was opened from, focused again on the way back. */
let opener: HTMLElement | null = null

const root = ref<HTMLElement | null>(null)
const bar = ref<HTMLElement | null>(null)

function open(v: View, e: Event) {
  if (view.value === 'home')
    opener = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
  view.value = v
  void nextTick(() => {
    // The page's marked control (or the first one inside a marked block), else its back button.
    const mark = root.value?.querySelector<HTMLElement>('[data-sub-initial]:not(:disabled)')
    const first = mark?.matches(CONTROL)
      ? mark
      : mark?.querySelector<HTMLElement>(`:is(${CONTROL}):not(:disabled)`)
    ;(first ?? root.value?.querySelector<HTMLElement>('.back'))?.focus()
  })
}
function back() {
  if (view.value === 'home') return emit('resume')
  if (view.value === 'shortcuts') {
    view.value = 'controls'
    void nextTick(() => root.value?.querySelector<HTMLElement>('[data-shortcuts]')?.focus())
    return
  }
  view.value = 'home'
  void loadSlots()
  void nextTick(() => opener?.focus())
}

useNavScope(root, { back, autofocus: true })

const consoleName = computed(() => t(`console.${props.game.consoleId}`))

const SUB_TITLES: Record<Exclude<View, 'home' | 'restart'>, string> = {
  slots: 'qm.more',
  screens: 'qm.row.screens.name',
  controls: 'qm.row.controls.name',
  shortcuts: 'qm.shortcuts',
  speed: 'qm.speedPage',
}
const subTitle = computed(() =>
  view.value === 'home' || view.value === 'restart' ? '' : t(SUB_TITLES[view.value]),
)

// ---- The input in use: hints show only its keys ----
/** The key that goes back / resumes: Esc on the keyboard, B on a controller, none on touch. */
const backKey = computed(() => {
  if (inputMode.value === 'keyboard') return t('flow.hintEsc')
  if (inputMode.value === 'controller') return t('controls.button.b')
  return null
})

// ---- Session settings («in prova») ----
const schema = ref<SettingsSchema | null>(null)
const session = ref<Record<string, SettingRow>>({})
const SESSION = { level: 'session' } as const
const PAGES = ['video', 'fx', 'audio', 'speed', 'saves', 'touch']

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
const trialTitle = computed(() =>
  trial.value.length === 1
    ? t('qm.trialOne', { names: trial.value.join(', ') })
    : t('qm.trial', { n: trial.value.length, names: trial.value.join(', ') }),
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
  speed: ['speed.ffval', 'speed.slow', 'audio.ff'],
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
const phone = computed(() => schema.value?.device === 'phone')
const sheet = computed(() => phone.value && portrait.value)

// ---- Speed (G07): Normale, Avanti veloce (`speed.ffval`), Rallentatore (`speed.slow`) ----
const ffSpeed = computed(() => speedFromOption(value('speed.ffval')) ?? FF_DEFAULT)
const slowSpeed = computed(() => speedFromOption(value('speed.slow')) ?? SLOW_DEFAULT)
const optionText = (key: string, fallback: string) => {
  const id = value(key)
  return typeof id === 'string' ? t(msg.option(key, id)) : fallback
}
const speedChoices = computed(() => [
  { id: 'normal', label: t('qm.normal'), value: t('speed.factor', { value: 1 }), speed: NORMAL },
  { id: 'ff', label: t('qm.ff'), value: optionText('speed.ffval', ''), speed: ffSpeed.value },
  { id: 'slow', label: t('qm.slow'), value: optionText('speed.slow', ''), speed: slowSpeed.value },
])
// G16 changed the speed of the mode in use: the game follows at once.
watch([ffSpeed, slowSpeed], ([ff, slow], [oldFf, oldSlow]) => {
  if (!sameSpeed(ff, oldFf) && sameSpeed(props.speed, oldFf)) emit('speed', ff)
  else if (!sameSpeed(slow, oldSlow) && sameSpeed(props.speed, oldSlow)) emit('speed', slow)
})

// ---- Controller shortcuts (G10, read-only: they are edited in Controls, C03) ----
const combos = ref<ComboRow[]>([])
const modifier = ref<Pos>('l3')
async function loadBindings() {
  try {
    const b = await emu.controls.bindings('generic', SESSION)
    // Reserved shortcuts (features still to come) are listed in Controls (C03), not here.
    combos.value = b.combos.filter(c => c.status === 'active')
    modifier.value = b.modifier
  } catch (e) {
    emit('error', errorText(e))
  }
}
const boundCombos = computed(() => combos.value.filter(r => r.pos !== null).length)

// ---- Quick saves (G07, G08): the auto quick save and `saves.slots` slots ----
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
/** Tiles on the main screen: Automatico + slots 1–4 on desktop, Automatico + 1–2 on the phone. */
const shownCount = computed(() => (sheet.value ? 3 : 5))
const mainSlots = computed(() => slots.value.slice(0, shownCount.value))
const chosenInfo = computed(() => slots.value.find(s => s.slot === chosen.value))
const chosenEmpty = computed(() => !chosenInfo.value || chosenInfo.value.savedAt === null)
const isAuto = computed(() => chosen.value === AUTO_SLOT)
function slotWhen(s: SlotInfo): string {
  if (s.savedAt === null) return t('qm.empty')
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'short', timeStyle: 'short' }).format(
    s.savedAt,
  )
}
const slotName = (slot: number) => (slot === AUTO_SLOT ? t('qm.auto') : t('qm.slot', { slot }))
const saveLabel = computed(() => t('qm.saveSlot', { slot: chosen.value }))
const loadLabel = computed(() => {
  if (isAuto.value) return t('qm.loadAuto')
  if (chosenEmpty.value) return t('qm.emptySlot', { slot: chosen.value })
  return t('qm.loadSlot', { slot: chosen.value })
})
function pick(slot: number) {
  chosen.value = slot
  savedSlot.value = null
}

// «Salvato nello slot N · Annulla»: a save made here stays in the menu with its undo.
const savedSlot = ref<number | null>(null)
let saving: number | null = null
function save() {
  saving = chosen.value
  savedSlot.value = null
  emit('save', chosen.value)
}
async function undoSave() {
  try {
    await emu.undoSave()
  } catch (e) {
    emit('error', errorText(e))
  }
}
/** Carica loads and resumes at once (App.vue shows G17). */
function load() {
  if (!chosenEmpty.value) emit('load', chosen.value)
}

const undoAllowed = computed(() => value('saves.undo') !== false)
/** With a controller, «Annulla il caricamento» sits at the top until the next save or load. */
const showUndoLoad = computed(
  () => props.undoLoadSlot !== null && undoAllowed.value && inputMode.value === 'controller',
)
const loadedText = computed(() =>
  props.undoLoadSlot === AUTO_SLOT
    ? t('toast.autoLoaded')
    : t('toast.stateLoaded', { slot: props.undoLoadSlot ?? 0 }),
)

/** Opening the menu focuses the selected slot (or «Altri slot» when it is not on screen). */
async function focusChosen() {
  await nextTick()
  const target =
    root.value?.querySelector<HTMLElement>('.slot.on') ??
    root.value?.querySelector<HTMLElement>('.more-link')
  target?.focus({ preventScroll: true })
}

// ---- Tools (Strumenti) ----
const dual = computed(() => props.game.consoleId === 'nds' || props.game.consoleId === '3ds')
interface Tool {
  id: string
  icon: Component
  label: string
  /** null = no backend yet: shown disabled, named in the note under the tiles. */
  run: (() => void) | null
  pressed?: boolean
}
const tools = computed<Tool[]>(() => {
  const list: Tool[] = [
    { id: 'shot', icon: Camera, label: t('qm.screenshot'), run: () => emit('screenshot') },
    {
      id: 'mute',
      icon: VolumeX,
      label: t('qm.mute'),
      run: () => emit('mute', !props.muted),
      pressed: props.muted,
    },
  ]
  if (!phone.value)
    list.push({
      id: 'full',
      icon: Maximize,
      label: t('qm.fullscreen'),
      run: () => emit('fullscreen'),
      pressed: props.fullscreen,
    })
  if (dual.value)
    list.push(
      { id: 'swap', icon: ArrowDownUp, label: t('qm.swapScreens'), run: null },
      { id: 'lid', icon: Laptop, label: t('qm.closeLid'), run: null },
    )
  list.push({
    id: 'restart',
    icon: RefreshCw,
    label: t('qm.reset'),
    run: () => (view.value = 'restart'),
  })
  return list
})
const toolsNotYet = computed(() =>
  tools.value
    .filter(x => x.run === null)
    .map(x => x.label)
    .join(', '),
)
function runTool(tool: Tool, e: Event) {
  if (!tool.run) return
  if (tool.id === 'restart') {
    opener = e.currentTarget instanceof HTMLElement ? e.currentTarget : null
    view.value = 'restart'
    void nextTick(() => root.value?.querySelector<HTMLElement>('[data-sub-initial]')?.focus())
    return
  }
  tool.run()
}
function restart() {
  view.value = 'home'
  emit('reset')
}

// ---- G09: one card per setting, ‹ value › ----
interface Card {
  key: string
  name: string
  value: string
  sub: string
  prev: (() => void) | null
  next: (() => void) | null
  trial: boolean
}
function sourceText(r: SettingRow | undefined): string {
  switch (r?.source) {
    case 'session':
      return t('qm.source.session')
    case 'game':
      return t('qm.source.game')
    case 'console':
      return t('settingsUi.sameAs', {
        level: t('settingsUi.scope.console', { console: consoleName.value }),
      })
    case 'global':
    case 'default':
    case undefined:
      return t('settingsUi.sameAs', { level: t('settingsUi.scope.global') })
  }
}
/** ‹ › step through `ids` from `current`; null at the ends. */
function stepper(ids: string[], current: string, set: (id: string) => void) {
  const i = ids.indexOf(current)
  return {
    prev: i > 0 ? () => set(ids[i - 1]!) : null,
    next: i >= 0 && i < ids.length - 1 ? () => set(ids[i + 1]!) : null,
  }
}
const cards = computed<Card[]>(() => {
  if (!schema.value) return []
  const out: Card[] = rowsOf('screens').map(e => {
    const k = e.kind
    const ids = k.type === 'choice' ? k.options.filter(id => !e.hiddenOptions.includes(id)) : []
    const current = String(value(e.key) ?? '')
    const locked = !!schema.value && lockReason(e, schema.value, 'session') !== null
    const steps = locked
      ? { prev: null, next: null }
      : stepper(ids, current, id => void setSession(e.key, id))
    return {
      key: e.key,
      name: t(msg.name(e.key)),
      value: current ? t(msg.option(e.key, current)) : '',
      sub: sourceText(session.value[e.key]),
      trial: session.value[e.key]?.source === 'session',
      ...steps,
    }
  })
  const modes: VideoMode[] = props.nativeAvailable ? ['native', 'webview'] : ['webview']
  const method = stepper(modes, props.videoMode, m => emit('videoMode', m as VideoMode))
  out.push({
    key: 'video.path',
    name: t('settings.video.path.name'),
    value: t(
      `settings.video.path.option.${props.videoMode === 'native' ? 'accelerated' : 'compatible'}`,
    ),
    sub: props.nativeAvailable
      ? t(`settingsUi.onlyHere.${phone.value ? 'phone' : 'desktop'}`)
      : t('qm.noNative'),
    trial: false,
    ...method,
  })
  return out
})

// G09 on desktop: the bar covers the bottom of the window, the game shrinks above it.
let barObserver: ResizeObserver | null = null
function reportInset() {
  const el = bar.value
  emit('inset', el ? Math.ceil(window.innerHeight - el.getBoundingClientRect().top + 12) : 0)
}
watch(
  () => view.value === 'screens' && !sheet.value,
  async on => {
    barObserver?.disconnect()
    barObserver = null
    if (!on) return emit('inset', 0)
    await nextTick()
    if (!bar.value) return
    barObserver = new ResizeObserver(reportInset)
    barObserver.observe(bar.value)
    reportInset()
  },
)
/** The game shows through (G09, and the phone's G09 sheet under it): no dimming. */
const live = computed(() => view.value === 'screens')

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
    if (ev.type === 'stateSaved') {
      if (ev.slot === saving) savedSlot.value = ev.slot
      saving = null
      void loadSlots()
    }
    if (ev.type === 'saveUndone') {
      savedSlot.value = null
      void loadSlots()
    }
    if (ev.type === 'error') saving = null
  })
})
onBeforeUnmount(() => {
  unlistenSettings?.()
  unlistenEvents?.()
  barObserver?.disconnect()
  emit('inset', 0)
  portraitQuery.removeEventListener('change', onOrientation)
})
</script>

<template>
  <div class="qm" :class="{ sheet, live }">
    <div v-if="!live && !sheet" class="scrim" aria-hidden="true"></div>

    <!-- G09 on desktop: a bar at the bottom, the game visible above it. -->
    <section
      v-if="view === 'screens' && !sheet"
      ref="root"
      class="panel bar"
      role="dialog"
      :aria-label="t('qm.row.screens.name')"
    >
      <div ref="bar" class="bar-inner">
        <div class="bar-head">
          <button class="back" @click="back">
            <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
            {{ t('qm.label') }}
            <span v-if="backKey" class="chip">{{ backKey }}</span>
          </button>
          <span class="bar-title">{{ t('qm.row.screens.name') }}</span>
          <span class="grow"></span>
          <span class="muted small">{{ t('qm.liveNote') }}</span>
        </div>
        <div class="cards">
          <div v-for="(c, i) in cards" :key="c.key" class="card" :class="{ trial: c.trial }">
            <span class="card-name">{{ c.name }}</span>
            <div class="card-value">
              <button
                class="step"
                :disabled="!c.prev"
                :aria-label="t('qm.prev', { name: c.name })"
                :data-sub-initial="i === 0 ? '' : undefined"
                @click="c.prev?.()"
              >
                <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
              <span class="card-current">{{ c.value }}</span>
              <button
                class="step"
                :disabled="!c.next"
                :aria-label="t('qm.next', { name: c.name })"
                @click="c.next?.()"
              >
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
            </div>
            <span class="card-sub">{{ c.sub }}</span>
          </div>
        </div>
        <div v-if="trial.length" class="bar-trial" role="status">
          <span class="grow trial-title">{{ trialTitle }}</span>
          <button class="act slim" @click="discard">{{ t('qm.undo') }}</button>
          <button class="act slim strong" @click="commit('game')">{{ t('qm.saveGame') }}</button>
        </div>
      </div>
    </section>

    <section
      v-else
      ref="root"
      class="panel menu"
      :class="{ [`at-${view}`]: true }"
      :role="view === 'restart' ? 'alertdialog' : 'dialog'"
      :aria-label="view === 'restart' ? t('qm.restartTitle') : t('qm.label')"
    >
      <span v-if="sheet" class="grip" aria-hidden="true"></span>

      <!-- G11: the confirmation of Riavvia, in place of the menu. -->
      <template v-if="view === 'restart'">
        <div class="sub-title">{{ t('qm.restartTitle') }}</div>
        <p class="muted restart-text">{{ t('qm.restartText') }}</p>
        <div class="restart-actions">
          <button class="act" data-sub-initial @click="back">{{ t('qm.cancel') }}</button>
          <button class="act danger-outline" @click="restart">
            <RefreshCw :size="18" :stroke-width="1.75" aria-hidden="true" />
            {{ t('qm.restartConfirm') }}
          </button>
        </div>
      </template>

      <template v-else>
        <header v-if="!sheet && view === 'home'" class="head">
          <span class="title">{{ game.title }}</span>
          <span class="muted small">{{ consoleName }}</span>
        </header>

        <!-- With a controller, after a load: «Annulla il caricamento» at the top (G17). -->
        <div v-if="showUndoLoad && view === 'home'" class="undo-load" role="status">
          <span class="grow">{{ loadedText }}</span>
          <button class="act slim" @click="emit('undoLoad')">
            <RotateCcw :size="16" :stroke-width="1.75" aria-hidden="true" />
            {{ t('qm.undoLoad') }}
          </button>
        </div>

        <!-- A sub-page: back one level (B / Esc), title. -->
        <div v-if="view !== 'home'" class="sub-head">
          <button
            v-if="sheet"
            class="back icon-only"
            :aria-label="t('qm.label')"
            :title="t('qm.label')"
            @click="back"
          >
            <ChevronLeft :size="22" :stroke-width="1.75" aria-hidden="true" />
          </button>
          <button v-else class="back" @click="back">
            <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
            {{ view === 'shortcuts' ? t('qm.row.controls.name') : t('qm.label') }}
            <span v-if="backKey" class="chip">{{ backKey }}</span>
          </button>
          <div class="sub-title">{{ subTitle }}</div>
        </div>

        <div class="body">
          <!-- G07 and G08: quick saves -->
          <section
            v-if="view === 'home' || view === 'slots'"
            class="block"
            :aria-label="t('qm.savesTitle')"
          >
            <div v-if="view === 'home'" class="block-head">
              <span class="overline">{{ t('qm.savesTitle') }}</span>
              <button class="link more-link" @click="open('slots', $event)">
                {{ sheet ? t('qm.moreShort') : t('qm.more') }}
              </button>
            </div>
            <div
              class="slots"
              :class="{ all: view === 'slots' }"
              role="radiogroup"
              :aria-label="t('qm.slotsLabel')"
            >
              <button
                v-for="s in view === 'home' ? mainSlots : slots"
                :key="s.slot"
                class="slot"
                :class="{ on: s.slot === chosen }"
                role="radio"
                :aria-checked="s.slot === chosen"
                :data-sub-initial="s.slot === chosen ? '' : undefined"
                :data-nav-initial="s.slot === chosen ? '' : undefined"
                @click="pick(s.slot)"
              >
                <span class="thumb" :class="{ blank: s.savedAt === null }" aria-hidden="true">
                  <img v-if="s.thumbUrl" :src="s.thumbUrl" alt="" />
                </span>
                <span class="slot-name">{{ slotName(s.slot) }}</span>
                <span class="slot-when num">{{ slotWhen(s) }}</span>
              </button>
            </div>
            <div class="pair">
              <button class="act" :disabled="chosenEmpty" @click="load">
                <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
                {{ loadLabel }}
              </button>
              <span v-if="isAuto" class="auto-note">{{ t('qm.autoUpdates') }}</span>
              <button v-else class="act" @click="save">
                <Save :size="18" :stroke-width="1.75" aria-hidden="true" />
                {{ saveLabel }}
              </button>
            </div>
            <div v-if="savedSlot !== null" class="saved" role="status">
              {{ t('qm.saved', { slot: savedSlot }) }}
              <button class="link" @click="undoSave">{{ t('qm.undoSave') }}</button>
            </div>
            <!-- G08: the in-game save's backups live in Saves (V04), with the game closed. -->
            <template v-if="view === 'slots'">
              <button class="row" disabled>
                <span class="row-icon" aria-hidden="true">
                  <ArchiveRestore :size="18" :stroke-width="1.75" />
                </span>
                <span class="grow row-name">{{ t('qm.backup') }}</span>
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
              <span class="muted small">{{ t('qm.backupNotHere') }}</span>
            </template>
          </section>

          <template v-if="view === 'home'">
            <!-- Speed -->
            <section class="block" :aria-label="t('qm.speedTitle')">
              <div class="block-head">
                <span class="overline">{{ t('qm.speedTitle') }}</span>
                <button class="link" @click="open('speed', $event)">{{ t('qm.setSpeeds') }}</button>
              </div>
              <div class="segmented speeds" role="radiogroup" :aria-label="t('qm.speedTitle')">
                <button
                  v-for="c in speedChoices"
                  :key="c.id"
                  role="radio"
                  :aria-checked="sameSpeed(speed, c.speed)"
                  :class="{ on: sameSpeed(speed, c.speed) }"
                  @click="emit('speed', c.speed)"
                >
                  <span class="speed-label">{{ c.label }}</span>
                  <span class="speed-value num">{{ c.value }}</span>
                </button>
              </div>
            </section>

            <!-- Tools, then Schermi e immagine and Controlli: tiles on desktop, a grid on the phone. -->
            <section v-if="!sheet" class="block" :aria-label="t('qm.toolsTitle')">
              <span class="overline">{{ t('qm.toolsTitle') }}</span>
              <div class="tiles" :class="{ three: dual }">
                <button
                  v-for="x in tools"
                  :key="x.id"
                  class="tile"
                  :class="{ on: x.pressed }"
                  :disabled="!x.run"
                  :aria-pressed="x.pressed === undefined ? undefined : x.pressed"
                  @click="runTool(x, $event)"
                >
                  <component :is="x.icon" :size="20" :stroke-width="1.75" aria-hidden="true" />
                  {{ x.label }}
                </button>
              </div>
              <span v-if="toolsNotYet" class="muted small">
                {{ t('qm.toolsNotYet', { names: toolsNotYet }) }}
              </span>
            </section>

            <nav v-if="!sheet" class="rows" :aria-label="t('qm.moreLabel')">
              <button class="row" @click="open('screens', $event)">
                <span class="row-icon" aria-hidden="true">
                  <Rows2 :size="18" :stroke-width="1.75" />
                </span>
                <span class="grow row-name">{{ t('qm.row.screens.name') }}</span>
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
              <button class="row" @click="open('controls', $event)">
                <span class="row-icon" aria-hidden="true">
                  <Gamepad2 :size="18" :stroke-width="1.75" />
                </span>
                <span class="grow row-name">{{ t('qm.row.controls.name') }}</span>
                <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
              </button>
            </nav>

            <!-- Phone: one grid of two columns, for the thumb. -->
            <nav v-else class="grid2" :aria-label="t('qm.toolsTitle')">
              <button class="cell" @click="open('screens', $event)">
                <Rows2 :size="18" :stroke-width="1.75" aria-hidden="true" />
                <span class="cell-label">{{ t('qm.row.screens.name') }}</span>
              </button>
              <button class="cell" @click="open('controls', $event)">
                <Gamepad2 :size="18" :stroke-width="1.75" aria-hidden="true" />
                <span class="cell-label">{{ t('qm.row.controls.name') }}</span>
              </button>
              <button
                v-for="x in tools"
                :key="x.id"
                class="cell"
                :class="{ on: x.pressed }"
                :disabled="!x.run"
                :aria-pressed="x.pressed === undefined ? undefined : x.pressed"
                @click="runTool(x, $event)"
              >
                <component :is="x.icon" :size="18" :stroke-width="1.75" aria-hidden="true" />
                <span class="cell-label">{{ x.label }}</span>
              </button>
            </nav>
            <span v-if="sheet && toolsNotYet" class="muted small">
              {{ t('qm.toolsNotYet', { names: toolsNotYet }) }}
            </span>
          </template>

          <!-- G16: the speeds, at the levels each one accepts -->
          <template v-else-if="view === 'speed'">
            <div class="list" data-sub-initial>
              <template v-if="schema">
                <SettingRowView
                  v-for="e in rowsOf('speed')"
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
            </div>
            <p class="muted small note">{{ t('qm.speedsAlsoIn') }}</p>
          </template>

          <!-- Phone G09: the cards in the sheet, the game at full brightness above. -->
          <div v-else-if="view === 'screens'" class="cards phone-cards">
            <div v-for="c in cards" :key="c.key" class="card" :class="{ trial: c.trial }">
              <span class="card-name">{{ c.name }}</span>
              <div class="card-value">
                <button
                  class="step"
                  :disabled="!c.prev"
                  :aria-label="t('qm.prev', { name: c.name })"
                  @click="c.prev?.()"
                >
                  <ChevronLeft :size="18" :stroke-width="1.75" aria-hidden="true" />
                </button>
                <span class="card-current">{{ c.value }}</span>
                <button
                  class="step"
                  :disabled="!c.next"
                  :aria-label="t('qm.next', { name: c.name })"
                  @click="c.next?.()"
                >
                  <ChevronRight :size="18" :stroke-width="1.75" aria-hidden="true" />
                </button>
              </div>
              <span class="card-sub">{{ c.sub }}</span>
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
              <span class="grow">
                <span class="line-name">{{ t('qm.remap') }}</span>
                <span class="line-sub">{{ t('qm.notYet') }}</span>
              </span>
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

          <!-- G10: the controller shortcuts, read-only -->
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
        </div>

        <!-- Changes «in prova» (G16, G10 and the phone's G09). -->
        <div v-if="trial.length" class="trial" role="status">
          <div>
            <div class="trial-title">{{ trialTitle }}</div>
            <div class="muted small">{{ t('qm.trialHint') }}</div>
          </div>
          <div class="row-actions">
            <button class="act slim" @click="discard">{{ t('qm.undo') }}</button>
            <button class="act slim strong" @click="commit('game')">
              {{ t('qm.saveGame') }}
            </button>
          </div>
          <button class="link underline" @click="commit('console')">
            {{ t('qm.saveConsole', { console: consoleName }) }}
          </button>
        </div>

        <!-- «Chiudi il gioco» far from Riprendi: bottom left on desktop, beside it on the phone. -->
        <footer v-if="view === 'home'" class="foot">
          <button class="act danger" @click="emit('close')">
            <Power v-if="!sheet" :size="18" :stroke-width="1.75" aria-hidden="true" />
            {{ t('qm.close') }}
          </button>
          <button class="btn btn-primary resume" @click="emit('resume')">
            <Play
              v-if="!sheet"
              :size="18"
              fill="currentColor"
              :stroke-width="0"
              aria-hidden="true"
            />
            {{ sheet ? t('qm.resumeShort') : t('qm.resume') }}
            <span v-if="backKey && !sheet" class="chip on-accent">{{ backKey }}</span>
          </button>
        </footer>
      </template>
    </section>
  </div>
</template>

<style scoped>
.qm {
  position: fixed;
  inset: 0;
  z-index: 20;
  pointer-events: none;
}
.qm > * {
  pointer-events: auto;
}
/* The paused game behind the menu: dimmed (blurred where the webview draws it). */
.scrim {
  position: absolute;
  inset: 0;
  background: #00000099;
  backdrop-filter: blur(2px);
}
.panel {
  position: absolute;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 16px;
  border-radius: var(--radius-16);
  background: var(--glass-thick);
  border: 1px solid var(--line);
  box-shadow:
    inset 0 1px 0 var(--glass-edge),
    0 32px 96px #000000a6;
  backdrop-filter: blur(24px) saturate(140%);
}
/* Desktop: centred, 640 px wide, as tall as its content. */
.menu {
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 640px;
  max-width: calc(100vw - 32px);
  max-height: calc(100vh - 32px);
  padding: 20px;
}
.head {
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.head .title {
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
.body {
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
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 8px;
}
.slots.all {
  grid-template-columns: repeat(4, minmax(0, 1fr));
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
  box-sizing: border-box;
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
  height: 44px;
  min-width: 0;
  padding: 0 14px;
  border-radius: var(--radius-10);
  border: 1px solid var(--line);
  background: var(--surface-raised);
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  white-space: nowrap;
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
.act.slim {
  height: 36px;
  font-weight: 500;
}
.act.strong {
  font-weight: 600;
  border-color: var(--line-strong);
}
.act.danger {
  color: var(--danger);
}
.act.danger-outline {
  color: var(--danger);
  border-color: var(--danger);
}
.auto-note {
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 8px;
  border-radius: var(--radius-10);
  border: 1px dashed var(--line-strong);
  color: var(--ink-muted);
  font-size: 12px;
  text-align: center;
}
.saved {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 20px;
  color: var(--success);
  font-weight: 500;
}
.saved .link {
  padding: 0 4px;
}
.undo-load {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 8px 8px 14px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  font-weight: 600;
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
  margin-left: 4px;
}
.chip.on-accent {
  color: var(--on-accent);
  border-color: #ffffff80;
}
.segmented {
  display: grid;
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-10);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.speeds {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.speeds button {
  height: 52px;
  min-width: 0;
  border: 0;
  border-radius: 8px;
  background: none;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  line-height: 17px;
}
.speeds button.on {
  background: var(--accent-soft);
  color: var(--accent-text);
}
.speed-label {
  font-weight: 600;
  white-space: nowrap;
}
.speed-value {
  font-size: 12px;
  font-weight: 500;
  opacity: 0.8;
}
.tiles {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}
.tiles.three {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.tile {
  min-width: 0;
  height: 68px;
  padding: 0 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border-radius: var(--radius-10);
  border: 1px solid var(--line);
  background: var(--surface-raised);
  font-weight: 600;
  font-size: 12px;
  white-space: nowrap;
}
.tile:hover:not(:disabled),
.cell:hover:not(:disabled) {
  background: var(--surface-hover);
}
.tile.on,
.cell.on {
  background: var(--accent-soft);
  border-color: var(--accent-text);
  color: var(--accent-text);
}
.tile:disabled,
.cell:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.rows {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.row {
  min-width: 0;
  height: 48px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-radius: var(--radius-10);
  border: 1px solid var(--line);
  background: var(--surface);
  text-align: left;
  color: var(--ink);
}
.row:hover:not(:disabled) {
  background: var(--surface-hover);
}
.row:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.row-icon {
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--surface-raised);
  display: grid;
  place-items: center;
  color: var(--ink-muted);
}
.row-name {
  font-weight: 600;
}
.row > svg {
  flex: 0 0 auto;
  color: var(--ink-muted);
}
.list {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
}
.list > * + * {
  border-top: 1px solid var(--line);
}
.line {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 52px;
  padding: 8px 12px;
  border: 0;
  background: none;
  text-align: left;
  color: var(--ink);
}
button.line:hover:not(:disabled) {
  background: var(--surface-hover);
}
.line:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.line > svg {
  flex: 0 0 auto;
}
.line-name {
  display: block;
  font-weight: 500;
}
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
.sub-head {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.sub-title {
  font-size: 17px;
  line-height: 22px;
  font-weight: 600;
}
.note {
  margin: 0;
}
.restart-text {
  margin: 0;
}
.restart-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
}
.grow {
  flex: 1;
  min-width: 0;
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
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 4px;
}
.resume {
  width: 260px;
  height: 44px;
  border-radius: var(--radius-10);
  gap: 8px;
  font-size: 14px;
}
.muted {
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}

/* G09 on desktop: a bar at the bottom, 1040 px, the game above it. */
.bar {
  left: 50%;
  bottom: 20px;
  transform: translateX(-50%);
  width: 1040px;
  max-width: calc(100vw - 32px);
  padding: 0;
}
.bar-inner {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px 16px;
}
.bar-head {
  display: flex;
  align-items: center;
  gap: 12px;
}
.bar-title {
  font-size: 15px;
  font-weight: 600;
}
.cards {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}
.card {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-radius: var(--radius-10);
  border: 1px solid var(--line);
  background: var(--surface);
}
.card.trial {
  border-color: var(--accent-text);
  background: color-mix(in srgb, var(--violet) 10%, transparent);
}
.card-name,
.card-sub {
  font-size: 12px;
  color: var(--ink-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card-sub {
  font-size: 11px;
}
.card-value {
  display: flex;
  align-items: center;
  gap: 4px;
}
.card-current {
  flex: 1;
  min-width: 0;
  text-align: center;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.step {
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  display: grid;
  place-items: center;
  color: var(--ink-muted);
}
.step:hover:not(:disabled) {
  background: var(--surface-hover);
}
.step:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.bar-trial {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Phone in portrait: a bottom sheet under the game (board «Tel-MenuRapido3», 390 × 844: top
   304 px = 44 px + the 3:2 game image; G08 starts higher, G09 lower). */
.sheet .scrim {
  display: none;
}
.sheet .menu {
  left: 0;
  right: 0;
  bottom: 0;
  top: calc(100vw * 2 / 3 + 44px);
  width: auto;
  max-width: none;
  max-height: none;
  transform: none;
  padding: 8px 16px 18px;
  gap: 10px;
  border-radius: 20px 20px 0 0;
  border-width: 1px 0 0;
  box-shadow: 0 -24px 64px #0000008c;
  font-size: 15px;
  line-height: 22px;
}
.sheet .menu.at-slots {
  top: calc(100vw * 2 / 3 - 24px);
}
.sheet .menu.at-screens {
  top: calc(100vw * 2 / 3 + 112px);
}
.grip {
  flex: 0 0 auto;
  align-self: center;
  width: 40px;
  height: 4px;
  border-radius: 999px;
  background: #ffffff33;
}
.sheet .body {
  flex: 1;
  gap: 10px;
}
.sheet .sub-head {
  flex-direction: row;
  align-items: center;
}
.back.icon-only {
  width: 44px;
  height: 44px;
  margin-left: -10px;
  padding: 0;
  justify-content: center;
  border-radius: var(--radius-10);
}
.sheet .overline {
  font-size: 12px;
}
.sheet .more-link,
.sheet .block-head .link {
  font-size: 13px;
  min-height: 44px;
}
.sheet .slots {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.sheet .slots.all {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}
.sheet .slot {
  border-radius: 12px;
  gap: 3px;
}
.sheet .slot.on {
  border-width: 2px;
}
.sheet .thumb {
  aspect-ratio: 2 / 1;
}
.sheet .slot-name {
  font-size: 13px;
}
.sheet .slot-when {
  font-size: 12px;
}
.sheet .act {
  height: 48px;
  border-radius: 12px;
  font-size: 14px;
}
.sheet .segmented {
  border-radius: 12px;
}
.sheet .speeds button {
  border-radius: 10px;
}
.sheet .speed-label {
  font-size: 13px;
}
.grid2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.cell {
  min-width: 0;
  height: 44px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 12px;
  border: 1px solid var(--line);
  background: var(--surface);
  font-weight: 600;
  font-size: 14px;
  white-space: nowrap;
  text-align: left;
}
.cell > svg {
  flex: 0 0 auto;
  color: var(--ink-muted);
}
.cell-label {
  overflow: hidden;
  text-overflow: ellipsis;
}
.phone-cards {
  grid-template-columns: 1fr 1fr;
}
.phone-cards .step {
  width: 44px;
  height: 44px;
}
.sheet .foot {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 8px;
}
.sheet .resume {
  width: auto;
  height: 48px;
  border-radius: 12px;
  font-size: 16px;
}
.sheet .row-actions .act {
  height: 44px;
}
</style>
