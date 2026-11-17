<script setup lang="ts">
// C02 «Rimappatura dei pulsanti» (board «Controlli», CaptureField): which button of the chosen
// device gives each game button, at «Tutti i giochi», «Giochi <console>» or «Solo questo gioco».
// A row (click, A or Enter) starts the capture: 5 s to press the button; B or Esc held 1 s
// cancel. A button already used by another row asks «… è già … Scambiali?».
// Beside it the tester: every position held lights up, and the last press says what it does.
//
// Values: `pad.remap` (overrides per controller type, `crates/emudome-core/src/bindings.rs`),
// resolved with the defaults by `emu.controls.bindings`.
import { computed, onBeforeUnmount, onMounted, ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type Button,
  type ConsoleId,
  type PadFamily,
  type PadState,
  type RemapRow,
  type SettingRow,
  type SettingsScope,
} from '../emu'
import { errorText } from '../i18n/labels'
import { library } from '../library'
import {
  buttonName,
  keyName,
  posName,
  POSITIONS,
  remapList,
  remapWithout,
  sourceName,
  useCapture,
  type CaptureDevice,
} from '../controls'

const props = defineProps<{
  family: PadFamily
  device: CaptureDevice
  states: PadState[]
  globalRows: Record<string, SettingRow>
}>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

// ---- Level: «Tutti i giochi», «Giochi <console>», «Solo questo gioco» ----
type Level = 'global' | 'console' | 'game'
const level = ref<Level>('global')
const contextConsole = ref<ConsoleId | null>(null)
const gameScope = ref<SettingsScope | null>(null)
async function findContext() {
  gameScope.value = await emu.settings.gameScope().catch(() => null)
  const open = gameScope.value
  if (open && open.level === 'game') return (contextConsole.value = open.console)
  // As the Settings section does: the console of the last game played (TODO Cowork, #41).
  const last = [...library.games]
    .filter(g => g.lastPlayed !== null)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))[0]
  contextConsole.value = last?.console ?? null
}
const levels = computed<Level[]>(() => {
  const out: Level[] = ['global']
  if (contextConsole.value) out.push('console')
  if (gameScope.value) out.push('game')
  return out
})
const scope = computed<SettingsScope>(() => {
  if (level.value === 'game' && gameScope.value) return gameScope.value
  if (level.value === 'console' && contextConsole.value)
    return { level: 'console', console: contextConsole.value }
  return { level: 'global' }
})
function levelName(l: Level): string {
  if (l === 'console' && contextConsole.value)
    return t('settingsUi.scope.console', { console: t(`console.${contextConsole.value}`) })
  if (l === 'game') return t('controls.levelGame')
  return t('settingsUi.scope.global')
}

// ---- The table at this level ----
const rows = ref<RemapRow[]>([])
/** `pad.remap` as stored or inherited at this level (the list a change starts from). */
const stored = ref<string[]>([])
const storedHere = ref(false)
async function load() {
  try {
    const [b, page] = await Promise.all([
      emu.controls.bindings(props.family, scope.value),
      emu.settings.page('pad', scope.value),
    ])
    rows.value = b.remap
    const row = page.find(r => r.key === 'pad.remap')
    stored.value = Array.isArray(row?.value) ? row.value : []
    storedHere.value = row?.source === scope.value.level
  } catch (e) {
    emit('error', errorText(e))
  }
}
watch(scope, () => void load(), { deep: true })

async function write(list: string[]) {
  try {
    if (list.length === 0 && level.value === 'global' && storedHere.value)
      await emu.settings.reset('pad.remap', scope.value)
    else await emu.settings.set('pad.remap', scope.value, list)
  } catch (e) {
    emit('error', errorText(e))
  }
}

/** How many rows use each source: a source used twice is flagged. */
const uses = computed(() => {
  const m = new Map<string, number>()
  for (const r of rows.value) if (r.source) m.set(r.source, (m.get(r.source) ?? 0) + 1)
  return m
})
const name = (source: string | null) => sourceName(props.family, source)

// ---- Capture and conflicts ----
interface Conflict {
  button: Button
  other: Button
  source: string
}
const conflict = ref<Conflict | null>(null)
const last = ref('')

const capture = useCapture({
  device: () => props.device,
  state: id => props.states.find(s => s.id === id),
  done: (id, source) => void assign(id as Button, source),
})
watch(
  () => props.states,
  s => {
    capture.onPads(s)
    tester(s)
  },
)

function startCapture(button: Button) {
  conflict.value = null
  if (capture.target.value === button) return capture.stop()
  capture.start(button)
}

async function assign(button: Button, source: string) {
  const row = rows.value.find(r => r.button === button)
  if (!row || row.source === source) return
  const other = rows.value.find(r => r.button !== button && r.source === source)
  if (other) {
    conflict.value = { button, other: other.button, source }
    await nextTick()
    document.querySelector<HTMLElement>('[data-conflict-swap]')?.focus()
    return
  }
  last.value = t('controls.assigned', { source: name(source), button: buttonName(button) })
  await write(remapList(stored.value, props.family, rows.value, { [button]: source }))
}

async function swap() {
  const c = conflict.value
  if (!c) return
  conflict.value = null
  const mine = rows.value.find(r => r.button === c.button)?.source ?? null
  last.value = t('controls.assigned', { source: name(c.source), button: buttonName(c.button) })
  await write(
    remapList(stored.value, props.family, rows.value, { [c.button]: c.source, [c.other]: mine }),
  )
}

async function restoreDefaults() {
  conflict.value = null
  await write(remapWithout(stored.value, props.family))
}

// ---- A/B layout (`pad.ab`, «Tutti i giochi») ----
const ab = computed(() => props.globalRows['pad.ab']?.value ?? 'nintendo')
async function setAb(id: 'nintendo' | 'printed') {
  try {
    await emu.settings.set('pad.ab', { level: 'global' }, id)
  } catch (e) {
    emit('error', errorText(e))
  }
}

// ---- Tester: what is held lights up; the last press says what it does in the game ----
const held = ref(new Set<string>())
let prevHeld = new Set<string>()
function describe(source: string) {
  const users = rows.value.filter(r => r.source === source).map(r => buttonName(r.button))
  return users.length
    ? t('controls.pressInGame', { source: name(source), button: users.join(', ') })
    : t('controls.pressUnused', { source: name(source) })
}
function tester(states: PadState[]) {
  if (props.device.kind !== 'pad') return
  const id = props.device.id
  const now = new Set<string>(states.find(s => s.id === id)?.held ?? [])
  if (!capture.target.value) for (const p of now) if (!prevHeld.has(p)) last.value = describe(p)
  held.value = now
  prevHeld = now
}
function key(e: KeyboardEvent, down: boolean) {
  if (props.family !== 'keyboard' || capture.target.value) return
  const next = new Set(held.value)
  if (down) {
    if (!next.has(e.code) && !e.repeat) last.value = describe(e.code)
    next.add(e.code)
  } else next.delete(e.code)
  held.value = next
}
const keydown = (e: KeyboardEvent) => key(e, true)
const keyup = (e: KeyboardEvent) => key(e, false)

/** The tester's cells: every position, or the keys in use on the keyboard. */
const cells = computed(() =>
  props.family === 'keyboard'
    ? [...new Set(rows.value.map(r => r.source).filter((s): s is string => !!s))].map(s => ({
        id: s,
        label: keyName(s),
      }))
    : POSITIONS.map(p => ({ id: p as string, label: posName(props.family, p, false) })),
)

const hint = computed(() => {
  const target = capture.target.value
  if (!target) return t('controls.testerHint')
  const back = props.device.kind === 'pad' ? posName(props.family, props.device.back, false) : 'Esc'
  return t(props.device.kind === 'pad' ? 'controls.captureHintPad' : 'controls.captureHintKey', {
    button: buttonName(target as Button),
    back,
  })
})

onMounted(async () => {
  await findContext()
  await load()
  window.addEventListener('keydown', keydown)
  window.addEventListener('keyup', keyup)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', keydown)
  window.removeEventListener('keyup', keyup)
})

let unlisten: (() => void) | null = null
onMounted(async () => {
  unlisten = await emu.settings.onChanged(e => {
    if (e.key.startsWith('pad.')) void load()
  })
})
onBeforeUnmount(() => unlisten?.())
</script>

<template>
  <div class="remap">
    <div class="toolbar">
      <div v-if="levels.length > 1" class="field">
        <span class="label">{{ t('controls.appliesTo') }}</span>
        <div class="segments" role="radiogroup" :aria-label="t('controls.appliesTo')">
          <button
            v-for="l in levels"
            :key="l"
            class="seg"
            :class="{ on: l === level }"
            role="radio"
            :aria-checked="l === level"
            @click="level = l"
          >
            {{ levelName(l) }}
          </button>
        </div>
      </div>
      <div v-if="family !== 'keyboard'" class="field">
        <span class="label">{{ t('settings.pad.ab.name') }}</span>
        <div class="segments" role="radiogroup" :aria-label="t('settings.pad.ab.name')">
          <button
            v-for="id in ['nintendo', 'printed'] as const"
            :key="id"
            class="seg"
            :class="{ on: id === ab }"
            role="radio"
            :aria-checked="id === ab"
            @click="setAb(id)"
          >
            {{ t(`controls.ab.${id}`) }}
          </button>
        </div>
      </div>
    </div>

    <div class="grid">
      <section class="table">
        <div class="thead overline">
          <span>{{ t('controls.inGame') }}</span>
          <span>{{ family === 'keyboard' ? t('controls.onKeyboard') : t('controls.onPad') }}</span>
          <span></span>
        </div>
        <template v-for="(r, i) in rows" :key="r.button">
          <div class="tr" :class="{ capturing: capture.target.value === r.button }">
            <span class="game">{{ buttonName(r.button) }}</span>
            <button
              class="chip"
              :class="{ listening: capture.target.value === r.button }"
              :data-nav-initial="i === 0 ? '' : undefined"
              :aria-label="
                t('controls.changeButton', { button: buttonName(r.button), source: name(r.source) })
              "
              @click="startCapture(r.button)"
            >
              <template v-if="capture.target.value === r.button">
                {{
                  family === 'keyboard'
                    ? t('controls.pressKey', { n: capture.left.value })
                    : t('controls.pressButton', { n: capture.left.value })
                }}
              </template>
              <template v-else>{{ name(r.source) }}</template>
            </button>
            <span class="status" :class="{ warn: r.source && (uses.get(r.source) ?? 0) > 1 }">
              <template v-if="capture.target.value === r.button"></template>
              <template v-else-if="r.source && (uses.get(r.source) ?? 0) > 1">
                {{ t('controls.usedTwice') }}
              </template>
              <template v-else-if="r.changed">{{ t('settingsUi.changed') }}</template>
            </span>
          </div>
          <div v-if="conflict && conflict.button === r.button" class="conflict" role="alert">
            <span class="conflict-text">
              {{
                t('controls.conflict', {
                  source: name(conflict.source),
                  button: buttonName(conflict.other),
                })
              }}
            </span>
            <button class="btn" data-conflict-swap @click="swap">{{ t('controls.swap') }}</button>
            <button class="btn" @click="conflict = null">{{ t('controls.cancel') }}</button>
          </div>
        </template>
      </section>

      <aside class="side">
        <div class="card">
          <div class="overline">{{ t('controls.tester') }}</div>
          <div class="muted">{{ hint }}</div>
          <div class="cells">
            <span v-for="c in cells" :key="c.id" class="cell" :class="{ lit: held.has(c.id) }">
              {{ c.label }}
            </span>
          </div>
          <div class="small muted" aria-live="polite">{{ last || t('controls.noPress') }}</div>
        </div>
        <div class="card">
          <div class="overline">{{ t('controls.otherActions') }}</div>
          <button class="btn start" @click="restoreDefaults">
            {{ t('controls.restoreDefaults') }}
          </button>
          <button class="btn start" disabled :title="t('settingsUi.notYet')">
            {{ t('controls.copyFrom') }}
          </button>
          <button class="btn start" disabled :title="t('settingsUi.notYet')">
            {{ t('controls.turbo') }}
          </button>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.remap {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
}
.field {
  display: flex;
  align-items: center;
  gap: 10px;
}
.label {
  font-weight: 500;
  color: var(--ink-muted);
}
.segments {
  display: flex;
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.seg {
  height: 28px;
  padding: 0 10px;
  border: 0;
  border-radius: 4px;
  font-weight: 600;
  background: none;
  color: var(--ink-muted);
}
.seg.on {
  background: var(--accent-soft);
  color: var(--accent-text);
}
.grid {
  display: grid;
  grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}
.table,
.card {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
}
.table {
  overflow: hidden;
}
.thead,
.tr {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr) auto;
  gap: 12px;
  align-items: center;
  padding: 8px 16px;
}
.thead {
  background: var(--surface-raised);
}
.tr {
  border-top: 1px solid var(--line);
}
.tr.capturing {
  background: color-mix(in srgb, var(--violet) 10%, transparent);
}
.game {
  font-weight: 600;
}
.chip {
  justify-self: start;
  height: 30px;
  padding: 0 12px;
  border-radius: var(--radius-6);
  font-weight: 500;
  border: 1px solid var(--line);
  background: var(--surface-raised);
}
.chip:hover {
  background: var(--surface-hover);
}
.chip.listening {
  border: 1px dashed var(--accent-text);
  background: none;
  color: var(--accent-text);
}
.status {
  font-size: 12px;
  color: var(--ink-muted);
}
.status.warn {
  color: var(--warning);
}
.conflict {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: color-mix(in srgb, var(--warning) 10%, transparent);
  border-top: 1px solid color-mix(in srgb, var(--warning) 33%, transparent);
}
.conflict-text {
  flex: 1;
}
.side {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.card {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.muted {
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
}
.cells {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 6px;
}
.cell {
  height: 40px;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: var(--surface-raised);
  font-weight: 600;
  font-size: 12px;
  display: grid;
  place-items: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0 4px;
}
.cell.lit {
  color: var(--accent-text);
  border-color: var(--accent-text);
  background: var(--accent-soft);
}
.btn.start {
  align-self: flex-start;
}
.btn:disabled {
  opacity: 0.5;
}
</style>
