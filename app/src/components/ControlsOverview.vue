<script setup lang="ts">
// C01 «Controlli» (board «C01 · Desktop · Controlli», Controlli2, 17/11): what you use most
// first. The devices as cards (connected, battery; «In uso» on the chosen one), the
// shortcuts to remember read from the bindings in force (never a list here), the live button
// tester, then the rows down to C02 «Rimappa i pulsanti», C04 «Stick, grilletti e vibrazione»
// and the touch controls (customised on the phone).
//
// TODO(Cowork, C01): the board also lists controllers used before and now disconnected
// («Non collegato · ricordato»); EmuDome does not remember controllers yet.
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronRight, Gamepad2 } from '@lucide/vue'
import {
  emu,
  type ComboRow,
  type PadFamily,
  type PadInfo,
  type PadState,
  type Pos,
  type RemapRow,
  type SettingRow,
} from '../emu'
import { errorText, gamepadName } from '../i18n/labels'
import { buttonName, comboText, hotkeyName, keyName, posName } from '../controls'

const props = defineProps<{
  pads: PadInfo[]
  /** The device in use in this section. */
  device: PadInfo | 'keyboard'
  states: PadState[]
  globalRows: Record<string, SettingRow>
}>()
const emit = defineEmits<{
  pick: [number | 'keyboard']
  /** Go one step down: C02 remap, C03 shortcuts, C04 sticks. */
  open: ['remap' | 'shortcuts' | 'sticks']
  error: [string]
}>()
const { t } = useI18n()

const GLOBAL = { level: 'global' } as const
const family = computed<PadFamily>(() =>
  props.device === 'keyboard' ? 'keyboard' : props.device.family,
)
/** Shortcuts are a controller's: with the keyboard in use they are named as on a generic pad. */
const padFamily = computed<Exclude<PadFamily, 'keyboard'>>(() =>
  props.device === 'keyboard' ? 'generic' : props.device.family,
)

// ---- Bindings in force at «Tutti i giochi»: the shortcuts and what a press does in the game ----
const combos = ref<ComboRow[]>([])
const modifier = ref<Pos>('l3')
const remap = ref<RemapRow[]>([])
async function load() {
  try {
    const [pad, own] = await Promise.all([
      emu.controls.bindings(padFamily.value, GLOBAL),
      family.value === 'keyboard' ? emu.controls.bindings('keyboard', GLOBAL) : null,
    ])
    // The menu always opens with Home, whatever its combo.
    combos.value = pad.combos.filter(c => c.pos !== null || c.hotkey === 'menu')
    modifier.value = pad.modifier
    remap.value = (own ?? pad).remap
  } catch (e) {
    emit('error', errorText(e))
  }
}
watch(family, () => void load())
watch(
  () => [props.globalRows['pad.combos']?.value, props.globalRows['pad.remap']?.value],
  () => void load(),
  { deep: true },
)

function comboChip(c: ComboRow): string {
  return c.hotkey === 'menu'
    ? t('flow.hintHome')
    : comboText(padFamily.value, modifier.value, c.pos)
}

// ---- Device cards ----
const isOn = (d: PadInfo | 'keyboard') =>
  d === 'keyboard'
    ? props.device === 'keyboard'
    : props.device !== 'keyboard' && props.device.id === d.id
function padDetail(p: PadInfo): string {
  const parts = [t('controls.connected')]
  if (p.battery !== null) parts.push(t('controls.overview.battery', { percent: p.battery }))
  return parts.join(' · ')
}

// ---- Tester: what is held lights up; the last press says what it does in the game ----
const held = ref(new Set<string>())
let prevHeld = new Set<string>()
const stick = ref<[number, number]>([0, 0])
const last = ref('')
function sourceText(source: string): string {
  return family.value === 'keyboard' ? keyName(source) : posName(family.value, source as Pos)
}
function describe(source: string): string {
  const users = remap.value.filter(r => r.source === source).map(r => buttonName(r.button))
  return users.length
    ? t('controls.overview.lastPressed', { source: sourceText(source), button: users.join(', ') })
    : t('controls.overview.lastUnused', { source: sourceText(source) })
}
watch(
  () => props.states,
  states => {
    if (props.device === 'keyboard') return
    const id = props.device.id
    const state = states.find(s => s.id === id)
    const now = new Set<string>(state?.held ?? [])
    for (const p of now) if (!prevHeld.has(p)) last.value = describe(p)
    held.value = now
    prevHeld = now
    stick.value = state ? [state.axes[0] / 32768, state.axes[1] / 32768] : [0, 0]
  },
)
watch(
  () => props.device,
  () => {
    last.value = ''
    held.value = new Set()
    prevHeld = new Set()
    stick.value = [0, 0]
  },
)
function key(e: KeyboardEvent) {
  if (props.device !== 'keyboard' || e.repeat) return
  last.value = describe(e.code)
}

/** The outline's parts: position, place and shape (board «Sagoma del controller»). */
interface Part {
  pos: Pos
  style: Record<string, string>
  round: boolean
}
const part = (pos: Pos, x: number, y: number, w: number, h: number, round = true): Part => ({
  pos,
  round,
  style: { left: `${x}px`, top: `${y}px`, width: `${w}px`, height: `${h}px` },
})
const PARTS: Part[] = [
  part('l2', 52, -14, 48, 8, false),
  part('r2', 180, -14, 48, 8, false),
  part('l1', 48, 6, 56, 8, false),
  part('r1', 176, 6, 56, 8, false),
  part('back', 118, 52, 14, 8, false),
  part('start', 148, 52, 14, 8, false),
  part('north', 190, 40, 24, 24),
  part('west', 164, 66, 24, 24),
  part('east', 216, 66, 24, 24),
  part('south', 190, 92, 24, 24),
  part('up', 99, 92, 14, 14, false),
  part('left', 85, 106, 14, 14, false),
  part('right', 113, 106, 14, 14, false),
  part('down', 99, 120, 14, 14, false),
  part('r3', 148, 106, 32, 32),
]
const stickDot = computed(() => ({
  transform: `translate(${stick.value[0] * 12}px, ${stick.value[1] * 12}px)`,
}))

// ---- Rows down to C02 / C04 ----
const sticksSub = computed(() =>
  t('controls.overview.sticksSub', {
    dead: Number(props.globalRows['pad.dead']?.value ?? 15),
    trig: Number(props.globalRows['pad.trig']?.value ?? 50),
  }),
)

onMounted(() => {
  void load()
  window.addEventListener('keydown', key)
})
onBeforeUnmount(() => window.removeEventListener('keydown', key))
</script>

<template>
  <div class="overview">
    <h1 class="title">{{ t('shell.controls') }}</h1>

    <div class="devices" role="radiogroup" :aria-label="t('controls.devices')">
      <button
        v-for="(p, i) in pads"
        :key="p.id"
        class="device"
        role="radio"
        :aria-checked="isOn(p)"
        :data-nav-initial="i === 0 ? '' : undefined"
        @click="emit('pick', p.id)"
      >
        <span class="device-icon on" aria-hidden="true">
          <Gamepad2 :size="22" :stroke-width="1.75" />
        </span>
        <span class="device-text">
          <span class="device-name">{{ gamepadName(p) }}</span>
          <span class="device-detail">{{ padDetail(p) }}</span>
        </span>
        <span v-if="isOn(p)" class="badge">{{ t('controls.overview.inUse') }}</span>
      </button>
      <button
        class="device"
        role="radio"
        :aria-checked="isOn('keyboard')"
        :data-nav-initial="pads.length === 0 ? '' : undefined"
        @click="emit('pick', 'keyboard')"
      >
        <span class="device-icon on" aria-hidden="true">
          <Gamepad2 :size="22" :stroke-width="1.75" />
        </span>
        <span class="device-text">
          <span class="device-name">{{ t('controls.keyboard') }}</span>
          <span class="device-detail">{{ t('controls.keyboardDetail') }}</span>
        </span>
        <span v-if="isOn('keyboard')" class="badge">{{ t('controls.overview.inUse') }}</span>
      </button>
    </div>

    <div class="columns">
      <section class="block">
        <div class="block-head">
          <span class="overline">{{ t('controls.overview.shortcuts') }}</span>
          <span class="note">
            {{
              t('controls.overview.shortcutsHint', {
                modifier: posName(padFamily, modifier, false),
              })
            }}
          </span>
        </div>
        <div class="list">
          <div v-for="c in combos" :key="c.hotkey" class="combo">
            <span class="chip-cell"
              ><span class="chip">{{ comboChip(c) }}</span></span
            >
            <span>{{ hotkeyName(c.hotkey) }}</span>
          </div>
        </div>
        <button class="btn edit" @click="emit('open', 'shortcuts')">
          {{ t('controls.overview.editShortcuts') }}
        </button>
      </section>

      <section class="block">
        <span class="overline">{{ t('controls.tester') }}</span>
        <div class="tester">
          <div
            v-if="device !== 'keyboard'"
            class="outline"
            role="img"
            :aria-label="t('controls.overview.outline')"
          >
            <span class="stick" :class="{ lit: held.has('l3') }" style="left: 46px; top: 58px">
              <span class="dot" :style="stickDot"></span>
            </span>
            <span
              v-for="p in PARTS"
              :key="p.pos"
              class="part"
              :class="{ lit: held.has(p.pos), round: p.round }"
              :style="p.style"
            ></span>
          </div>
          <div class="last" aria-live="polite">{{ last || t('controls.overview.press') }}</div>
        </div>

        <div class="list">
          <button class="row" @click="emit('open', 'remap')">
            <span class="row-text">
              <span class="row-name">{{ t('controls.overview.remap') }}</span>
              <span class="row-sub">{{ t('controls.overview.remapSub') }}</span>
            </span>
            <ChevronRight class="chev" :size="16" :stroke-width="1.75" aria-hidden="true" />
          </button>
          <button class="row" :disabled="device === 'keyboard'" @click="emit('open', 'sticks')">
            <span class="row-text">
              <span class="row-name">{{ t('controls.overview.sticks') }}</span>
              <span class="row-sub">{{ sticksSub }}</span>
            </span>
            <ChevronRight class="chev" :size="16" :stroke-width="1.75" aria-hidden="true" />
          </button>
          <div class="row">
            <span class="row-text">
              <span class="row-name">{{ t('controls.overview.touch') }}</span>
              <span class="row-sub">{{ t('controls.overview.touchSub') }}</span>
            </span>
            <ChevronRight class="chev" :size="16" :stroke-width="1.75" aria-hidden="true" />
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.overview {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 24px 32px 48px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.overview > * {
  max-width: 1180px;
}
.title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
}

/* Device cards */
.devices {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 12px;
}
.device {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  text-align: left;
}
.device:hover {
  background: var(--surface-hover);
}
.device-icon {
  flex: 0 0 auto;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-10);
  background: var(--surface-raised);
  display: grid;
  place-items: center;
  color: var(--ink-faint);
}
.device-icon.on {
  color: var(--success);
}
.device-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.device-name {
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.device-detail {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.badge {
  flex: 0 0 auto;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--success) 14%, transparent);
  color: var(--success);
}

/* Two columns */
.columns {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}
@media (max-width: 900px) {
  .columns {
    grid-template-columns: minmax(0, 1fr);
  }
}
.block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.block-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
}
.note {
  font-size: 12px;
  color: var(--ink-muted);
}
.list {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.list > * + * {
  border-top: 1px solid var(--line);
}
.combo {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 40px;
  padding: 4px 16px;
}
.chip-cell {
  flex: 0 0 auto;
  min-width: 96px;
}
.chip {
  display: inline-flex;
  align-items: center;
  height: 20px;
  padding: 0 6px;
  border-radius: 4px;
  border: 1px solid var(--line-strong);
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}
.edit {
  align-self: flex-start;
}

/* Tester */
.tester {
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}
.outline {
  position: relative;
  width: 280px;
  height: 170px;
  margin-top: 14px;
  border-radius: 70px;
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.part {
  position: absolute;
  border: 1px solid var(--line-strong);
  border-radius: 3px;
}
.part.round {
  border-radius: 999px;
}
.part.lit,
.stick.lit {
  background: var(--accent-text);
  border-color: var(--accent-text);
}
.stick {
  position: absolute;
  width: 40px;
  height: 40px;
  border-radius: 999px;
  border: 1px solid var(--line-strong);
  display: grid;
  place-items: center;
}
.dot {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  background: var(--ink-muted);
}
.last {
  color: var(--ink-muted);
  text-align: center;
}

/* Rows down to C02 / C04 */
.row {
  display: flex;
  align-items: center;
  gap: 16px;
  min-height: 56px;
  padding: 8px 16px;
  border: 0;
  border-radius: 0;
  background: none;
  text-align: left;
}
button.row:hover {
  background: var(--surface-hover);
}
.row:disabled {
  opacity: 0.5;
  cursor: default;
}
.row-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.row-name {
  font-weight: 500;
}
.row-sub {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.chev {
  flex: 0 0 auto;
  color: var(--ink-muted);
}
</style>
