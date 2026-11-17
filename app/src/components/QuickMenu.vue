<script setup lang="ts">
// The quick menu (boards «G07–G11 · Desktop · Menu rapido»): glass panel 420 px from the right,
// game paused, five fixed tabs (L1 / R1 move between them), «Riprendi il gioco» in accent.
// Settings changed here are «in prova» (Session level): live, listed in the bar at the bottom,
// saved for this game or its console, or undone; closing the game drops them.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { I18nT, useI18n } from 'vue-i18n'
import { Camera, ChevronRight, Gamepad2, Play, Power, RotateCcw } from '@lucide/vue'
import {
  emu,
  type GameInfo,
  type SettingEntry,
  type SettingRow,
  type SettingsSchema,
  type SettingValue,
  type SlotInfo,
  type Speed,
  type VideoMode,
} from '../emu'
import { errorText, playTimeLabel, speedLabel } from '../i18n/labels'
import { useNavScope } from '../nav'
import { loadSchema, msg } from '../settingsModel'
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
}>()
const emit = defineEmits<{
  videoMode: [VideoMode]
  resume: []
  speed: [Speed]
  save: [number]
  load: [number]
  reset: []
  close: []
  error: [string]
}>()

const { t, locale } = useI18n()

type Tab = 'saves' | 'speed' | 'screens' | 'controls' | 'tools'
const TABS: Tab[] = ['saves', 'speed', 'screens', 'controls', 'tools']
const tab = ref<Tab>('saves')

// Controller and keyboard: B / Esc resume, L1 / R1 change tab, the focus starts on «Riprendi».
const root = ref<HTMLElement | null>(null)
useNavScope(root, {
  back: () => emit('resume'),
  autofocus: true,
  tab: dir => {
    const i = TABS.indexOf(tab.value)
    tab.value = TABS[(i + dir + TABS.length) % TABS.length] ?? 'saves'
  },
})

const playing = computed(() =>
  t('qm.playing', {
    console: t(`console.${props.game.consoleId}`),
    time: playTimeLabel(Math.max(60, (Date.now() - props.startedAt) / 1000)),
  }),
)

// ---- Session settings («in prova») ----
const schema = ref<SettingsSchema | null>(null)
const session = ref<Record<string, SettingRow>>({})
const SESSION = { level: 'session' } as const
const PAGES = ['video', 'fx', 'speed', 'saves']

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

// ---- Speed: the speeds of `speed.splist`, in order ----
const speeds = computed<Speed[]>(() => {
  const list = value('speed.splist')
  const ids = Array.isArray(list) ? list : ['0.5', '1', '1.5', '2', '3', '4', 'max']
  return ids.map(id => (id === 'max' ? { kind: 'max' } : { kind: 'factor', value: Number(id) }))
})
const isSelected = (s: Speed) =>
  s.kind === props.speed.kind &&
  (s.kind === 'max' || (props.speed.kind === 'factor' && s.value === props.speed.value))
const speedKey = (s: Speed) => (s.kind === 'max' ? 'max' : String(s.value))

// ---- Quick saves (G07): the auto quick save and `saves.slots` slots ----
const slots = ref<SlotInfo[]>([])
const chosen = ref(1)
async function loadSlots() {
  const count = Number(value('saves.slots') ?? 10) || 10
  try {
    slots.value = await emu.listSlots(count)
  } catch (e) {
    emit('error', errorText(e))
  }
}
const chosenInfo = computed(() => slots.value.find(s => s.slot === chosen.value))
const firstFree = computed(() => slots.value.find(s => s.slot > 0 && s.savedAt === null)?.slot ?? 1)
function slotWhen(s: SlotInfo): string {
  if (s.savedAt === null) return t('qm.empty')
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'short', timeStyle: 'short' }).format(
    s.savedAt,
  )
}
function saveChosen() {
  emit('save', chosen.value === 0 ? firstFree.value : chosen.value)
}
const autoEvery = computed(() => {
  const id = value('saves.autoq')
  return typeof id === 'string' ? t(msg.option('saves.autoq', id)) : ''
})

// Setting rows each tab shows at the Session level.
const ROWS: Record<Tab, string[]> = {
  saves: [],
  speed: ['speed.ffmode', 'speed.ffval', 'speed.rew'],
  screens: ['video.scale', 'video.filter'],
  controls: [],
  tools: [],
}
const rowsOf = (k: Tab) =>
  ROWS[k]
    .map(key => entry(key))
    .filter((e): e is SettingEntry => !!e && !!schema.value && e.phase <= schema.value.currentPhase)

let unlistenSettings: (() => void) | null = null
let unlistenEvents: (() => void) | null = null
onMounted(async () => {
  schema.value = await loadSchema()
  await loadSession()
  await loadSlots()
  unlistenSettings = await emu.settings.onChanged(() => void loadSession())
  // A save done here or with a shortcut: refresh the dates and thumbnails.
  unlistenEvents = await emu.onEvent(ev => {
    if (ev.type === 'stateSaved') void loadSlots()
  })
})
onBeforeUnmount(() => {
  unlistenSettings?.()
  unlistenEvents?.()
})
watch(tab, k => {
  if (k === 'saves') void loadSlots()
})
</script>

<template>
  <aside ref="root" class="menu glass-thick" :aria-label="t('qm.label')">
    <header class="head">
      <div class="title">{{ game.title }}</div>
      <div class="muted">{{ playing }}</div>
    </header>

    <div class="tabs" role="tablist" :aria-label="t('qm.tabs')">
      <button
        v-for="k in TABS"
        :key="k"
        class="tab"
        :class="{ on: k === tab }"
        role="tab"
        :aria-selected="k === tab"
        @click="tab = k"
      >
        {{ t(`qm.tab.${k}`) }}
      </button>
    </div>

    <div class="body">
      <!-- G07: quick saves -->
      <template v-if="tab === 'saves'">
        <div class="slots">
          <button
            v-for="s in slots"
            :key="s.slot"
            class="slot"
            :class="{ on: s.slot === chosen }"
            :aria-pressed="s.slot === chosen"
            @click="chosen = s.slot"
          >
            <span class="thumb" aria-hidden="true">
              <img v-if="s.thumbUrl" :src="s.thumbUrl" alt="" />
            </span>
            <span class="slot-text">
              <span class="slot-name">
                {{ s.slot === 0 ? t('qm.auto') : t('qm.slot', { slot: s.slot }) }}
              </span>
              <span class="muted small num">{{ slotWhen(s) }}</span>
            </span>
          </button>
        </div>
        <div class="row-actions">
          <button
            class="btn"
            :disabled="!chosenInfo || chosenInfo.savedAt === null"
            @click="emit('load', chosen)"
          >
            {{ chosen === 0 ? t('qm.loadAuto') : t('qm.loadSlot', { slot: chosen }) }}
          </button>
          <button class="btn" @click="saveChosen">
            {{ chosen === 0 ? t('qm.saveFree') : t('qm.saveSlot', { slot: chosen }) }}
          </button>
        </div>
        <div class="muted small">{{ t('qm.autoEvery', { every: autoEvery }) }}</div>
        <I18nT keypath="qm.keys" tag="div" class="muted small">
          <template #save><kbd>F1</kbd></template>
          <template #load><kbd>F4</kbd></template>
        </I18nT>
      </template>

      <!-- G08: speed -->
      <template v-else-if="tab === 'speed'">
        <div class="overline">{{ t('qm.speedTitle') }}</div>
        <div class="segmented" role="radiogroup" :aria-label="t('qm.speedTitle')">
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
        <div class="muted small">{{ t('qm.speedNote') }}</div>
      </template>

      <!-- G09: screens (DS layouts arrive in Phase 2) -->
      <template v-else-if="tab === 'screens'">
        <div class="overline">{{ t('qm.displayTitle') }}</div>
        <div class="segmented" role="radiogroup" :aria-label="t('settings.video.path.name')">
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
        <div v-if="!nativeAvailable" class="muted small">{{ t('qm.noNative') }}</div>
        <div class="overline">{{ t('qm.imageTitle') }}</div>
      </template>

      <!-- G10: controls -->
      <template v-else-if="tab === 'controls'">
        <div class="line">
          <Gamepad2 :size="16" :stroke-width="1.75" aria-hidden="true" />
          <div>
            <div>{{ t('qm.controller') }}</div>
            <div class="muted small">{{ controller ?? t('qm.noController') }}</div>
          </div>
        </div>
        <div class="line">
          <div class="grow">
            <div>{{ t('qm.shortcuts') }}</div>
            <div class="muted small">{{ t('qm.shortcutsHint') }}</div>
          </div>
          <button class="btn" disabled :title="t('qm.notYet')">{{ t('qm.showList') }}</button>
        </div>
        <button class="item" disabled :title="t('qm.notYet')">
          <span class="grow">{{ t('qm.remap') }}</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </button>
      </template>

      <!-- G11: tools -->
      <template v-else>
        <button class="item" disabled :title="t('qm.notYet')">
          <Camera :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span class="grow">{{ t('qm.screenshot') }}</span>
          <kbd>F12</kbd>
        </button>
        <button class="item" @click="emit('reset')">
          <RotateCcw :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span class="grow">{{ t('qm.reset') }}</span>
          <span class="muted small">{{ t('qm.resetHint') }}</span>
        </button>
        <button class="item" @click="emit('close')">
          <Power :size="16" :stroke-width="1.75" aria-hidden="true" />
          <span class="grow">{{ t('qm.close') }}</span>
          <span v-if="game.autosaveOnExit" class="muted small">{{ t('qm.closeHint') }}</span>
        </button>
      </template>

      <div v-if="schema && rowsOf(tab).length" class="rows">
        <SettingRowView
          v-for="e in rowsOf(tab)"
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
      </div>
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
        <button class="link" @click="commit('console')">
          {{ t('qm.saveConsole', { console: t(`console.${game.consoleId}`) }) }}
        </button>
      </div>
      <button class="btn btn-primary resume" data-nav-initial @click="emit('resume')">
        <Play :size="16" fill="currentColor" :stroke-width="0" aria-hidden="true" />
        {{ t('qm.resume') }}
        <kbd class="esc">{{ t('flow.hintEsc') }}</kbd>
      </button>
    </footer>
  </aside>
</template>

<style scoped>
.menu {
  position: fixed;
  top: 16px;
  right: 16px;
  bottom: 16px;
  width: 420px;
  max-width: calc(100vw - 32px);
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  z-index: 20;
}
.head .title {
  font-size: 17px;
  line-height: 22px;
  font-weight: 600;
}
.tabs {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.tab {
  height: 28px;
  padding: 0 4px;
  border: 0;
  border-radius: 4px;
  background: none;
  color: var(--ink-muted);
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tab.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  font-weight: 600;
}
.body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.slots {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.slot {
  display: grid;
  grid-template-columns: 64px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  padding: 8px;
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
  width: 64px;
  height: 43px;
  border-radius: 4px;
  overflow: hidden;
  background: var(--surface-raised);
}
.thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}
.slot-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.slot-name {
  font-weight: 600;
}
.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.row-actions .btn {
  flex: 1;
}
.segmented {
  display: flex;
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.segmented button {
  flex: 1;
  height: 28px;
  padding: 0 4px;
  border: 0;
  border-radius: 4px;
  background: none;
  color: var(--ink-muted);
  font-weight: 500;
  white-space: nowrap;
}
.segmented button.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  font-weight: 600;
}
.rows {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
}
.line,
.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  text-align: left;
}
.item:hover:not(:disabled) {
  background: var(--surface-hover);
}
.item:disabled {
  opacity: 0.55;
  cursor: not-allowed;
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
.link {
  align-self: flex-start;
  border: 0;
  background: none;
  padding: 0;
  color: var(--accent-text);
  text-decoration: underline;
}
.resume {
  width: 100%;
}
.esc {
  margin-left: auto;
}
.muted {
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
kbd {
  font: inherit;
  font-size: 11px;
  padding: 1px 5px;
  border: 1px solid var(--line-strong);
  border-radius: 4px;
}
</style>
