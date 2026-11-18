<script setup lang="ts">
// The Controls section (boards «Controlli» C01–C04, «Errori» E05). It opens on C01
// (ControlsOverview: device cards, where the device is chosen, shortcuts, tester, rows down); one
// step below, the chosen device with «‹ Controlli» back to C01 above its name and three tabs:
// button remapping (C02), shortcuts (C03), sticks and triggers (C04). No device column there
// (board C02, 18/11): the device is changed on C01. With no controller connected the section
// shows E05 until the keyboard is chosen.
//
// From Game details › Controlli (L07, `controlsFocus`) it opens on C02 at «Solo questo gioco» for
// that game.
//
// Controller: the shell's nav scope; L1 / R1 change tab one step below C01 (exposed `tab`), back
// goes from there to C01 (exposed `back`, true when it did), both called by AppShell.
// The tester and the capture field read `emu://pad-state`, watched while this section is open.
//
// TODO(E06, Trello): unknown controller banner and guided setup (14 buttons, saved as an SDL
// mapping). SDL opens as gamepads only the controllers it has a mapping for, so an unknown one
// never reaches this list: detecting it needs the joystick events, not done yet.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, Gamepad2 } from '@lucide/vue'
import {
  emu,
  type PadFamily,
  type PadInfo,
  type PadState,
  type Pos,
  type SettingRow,
  type SettingsSchema,
} from '../emu'
import { errorText, gamepadName } from '../i18n/labels'
import { controlsFocus } from '../gameDetails'
import { loadSchema } from '../settingsModel'
import type { CaptureDevice } from '../controls'
import ControlsOverview from './ControlsOverview.vue'
import ControlsRemap from './ControlsRemap.vue'
import ControlsShortcuts from './ControlsShortcuts.vue'
import ControlsSticks from './ControlsSticks.vue'

const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

/** L07 «Rimappa i pulsanti per questo gioco»: the game this section opened for. */
const gameFocus = controlsFocus.value
controlsFocus.value = null

/** C01, or one step below it (C02–C04): C02 when opened for a game. */
const view = ref<'overview' | 'device'>(gameFocus ? 'device' : 'overview')
const root = ref<HTMLElement | null>(null)

const pads = ref<PadInfo[]>([])
/** The chosen device: a controller id or the keyboard; null = the first controller. */
const chosen = ref<number | 'keyboard' | null>(null)
const states = ref<PadState[]>([])

const device = computed<PadInfo | 'keyboard' | null>(() => {
  if (chosen.value === 'keyboard') return 'keyboard'
  return pads.value.find(p => p.id === chosen.value) ?? pads.value[0] ?? null
})
const family = computed<PadFamily>(() =>
  device.value === 'keyboard' || !device.value ? 'keyboard' : device.value.family,
)
/** E05: no controller, and the keyboard not chosen. */
const empty = computed(() => device.value === null)

const deviceName = computed(() =>
  device.value === 'keyboard' || !device.value ? t('controls.keyboard') : gamepadName(device.value),
)
/** «Collegato · batteria 80 %», as on the C01 card. */
const deviceDetail = computed(() => {
  const d = device.value
  if (d === 'keyboard' || !d) return t('controls.keyboardDetail')
  const parts = [t('controls.connected')]
  if (d.battery !== null) parts.push(t('controls.overview.battery', { percent: d.battery }))
  return parts.join(' · ')
})

// ---- Settings shared by the tabs: the schema and the pad page at «Tutti i giochi» ----
const schema = ref<SettingsSchema | null>(null)
const globalRows = ref<Record<string, SettingRow>>({})
async function loadGlobal() {
  try {
    const rows = await emu.settings.page('pad', { level: 'global' })
    globalRows.value = Object.fromEntries(rows.map(r => [r.key, r]))
  } catch (e) {
    emit('error', errorText(e))
  }
}

/** The menu's «back» position (`pad.confirm`): B held 1 s cancels a capture. */
const backPos = computed<Pos>(() =>
  globalRows.value['pad.confirm']?.value === 'bottom' ? 'east' : 'south',
)
const captureDevice = computed<CaptureDevice>(() =>
  device.value === 'keyboard' || !device.value
    ? { kind: 'keyboard' }
    : { kind: 'pad', id: device.value.id, back: backPos.value },
)

// ---- Tabs ----
type Tab = 'remap' | 'shortcuts' | 'sticks'
const tabs = computed<Tab[]>(() =>
  family.value === 'keyboard' ? ['remap', 'shortcuts'] : ['remap', 'shortcuts', 'sticks'],
)
const tab = ref<Tab>('remap')
watch(tabs, list => {
  if (!list.includes(tab.value)) tab.value = 'remap'
})
/** L1 / R1: the previous / next tab. */
function stepTab(dir: -1 | 1) {
  if (empty.value || view.value !== 'device') return
  const list = tabs.value
  tab.value = list[(list.indexOf(tab.value) + dir + list.length) % list.length]!
}

/** The focus goes to the first control of the view just shown. */
async function focusView() {
  await nextTick()
  root.value?.querySelector<HTMLElement>('[data-nav-initial]:not(:disabled)')?.focus()
}
/** C01 → C02 / C03 / C04 on the device in use. */
function openTab(to: Tab) {
  tab.value = tabs.value.includes(to) ? to : 'remap'
  view.value = 'device'
  void focusView()
}
/** Back from C02–C04 to C01; false when already there (the shell then goes to the rail). */
function back(): boolean {
  if (empty.value || view.value === 'overview') return false
  view.value = 'overview'
  void focusView()
  return true
}
defineExpose({ tab: stepTab, back })

function pick(d: number | 'keyboard') {
  chosen.value = d
}

const unlisten: Array<() => void> = []
onMounted(async () => {
  schema.value = await loadSchema()
  await loadGlobal()
  try {
    pads.value = await emu.controls.pads()
  } catch (e) {
    emit('error', errorText(e))
  }
  unlisten.push(
    await emu.onGamepadConnected(p => {
      if (!pads.value.some(x => x.id === p.id)) pads.value = [...pads.value, p]
    }),
    await emu.onGamepadDisconnected(p => {
      pads.value = pads.value.filter(x => x.id !== p.id)
      if (chosen.value === p.id) chosen.value = null
    }),
    await emu.onGamepadChanged(p => {
      pads.value = pads.value.map(x => (x.id === p.id ? p : x))
    }),
    await emu.controls.onPadState(s => (states.value = s)),
    await emu.settings.onChanged(e => {
      if (e.key.startsWith('pad.')) void loadGlobal()
    }),
  )
  await emu.controls.watch(true)
})
onBeforeUnmount(() => {
  unlisten.forEach(u => u())
  void emu.controls.watch(false)
})
</script>

<template>
  <div ref="root" class="controls">
    <!-- E05: no controller connected -->
    <section v-if="empty" class="empty">
      <h1 class="section-title">{{ t('shell.controls') }}</h1>
      <div class="empty-body">
        <span class="empty-icon" aria-hidden="true">
          <Gamepad2 :size="24" :stroke-width="1.75" />
        </span>
        <h2 class="empty-title">{{ t('controls.e05.title') }}</h2>
        <p class="muted">{{ t('controls.e05.lead') }}</p>
        <button class="btn tall" data-nav-initial @click="pick('keyboard')">
          {{ t('controls.e05.keyboard') }}
        </button>
      </div>
    </section>

    <ControlsOverview
      v-else-if="view === 'overview' && device"
      :pads="pads"
      :device="device"
      :states="states"
      :global-rows="globalRows"
      @pick="pick"
      @open="openTab"
      @error="text => emit('error', text)"
    />

    <template v-else>
      <main class="main">
        <div class="head">
          <button class="back" data-nav-initial @click="back">
            <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
            {{ t('shell.controls') }}
          </button>
          <h2 class="device-title">{{ deviceName }}</h2>
          <div class="muted">{{ deviceDetail }}</div>
        </div>
        <div class="tabs" role="tablist" :aria-label="t('controls.tabsLabel')">
          <button
            v-for="tb in tabs"
            :key="tb"
            class="tab"
            role="tab"
            :aria-selected="tb === tab"
            :class="{ on: tb === tab }"
            @click="tab = tb"
          >
            {{ t(`controls.tab.${tb}`) }}
          </button>
        </div>

        <template v-if="schema">
          <ControlsRemap
            v-if="tab === 'remap'"
            :key="`remap-${family}`"
            :family="family"
            :device="captureDevice"
            :states="states"
            :global-rows="globalRows"
            :game="gameFocus"
            @error="text => emit('error', text)"
          />
          <ControlsShortcuts
            v-else-if="tab === 'shortcuts'"
            :family="family"
            :device="captureDevice"
            :states="states"
            :schema="schema"
            :global-rows="globalRows"
            @error="text => emit('error', text)"
          />
          <ControlsSticks
            v-else
            :device="captureDevice"
            :states="states"
            :schema="schema"
            :global-rows="globalRows"
            @error="text => emit('error', text)"
          />
        </template>
      </main>
    </template>
  </div>
</template>

<style scoped>
.controls {
  flex: 1;
  min-height: 0;
  display: flex;
}
.section-title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
}
.muted {
  margin: 0;
  color: var(--ink-muted);
}

/* E05 */
.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 24px 32px;
}
.empty-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  text-align: center;
  max-width: 460px;
  margin: 0 auto;
}
.empty-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-16);
  background: var(--surface-raised);
  border: 1px solid var(--line);
  display: grid;
  place-items: center;
  color: var(--ink-muted);
}
.empty-title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
}
.btn.tall {
  height: 36px;
  margin-top: 4px;
}

/* C02–C04: back to C01, above the device name */
.back {
  align-self: flex-start;
  display: flex;
  align-items: center;
  gap: 2px;
  margin: 0 0 4px -6px;
  padding: 2px 8px 2px 4px;
  border: 0;
  border-radius: var(--radius-6);
  background: none;
  color: var(--ink-muted);
  font-weight: 500;
}
.back:hover {
  background: var(--surface-hover);
  color: var(--ink);
}
.head {
  display: flex;
  flex-direction: column;
}

/* Device page */
.main {
  flex: 1;
  min-width: 0;
  overflow: auto;
  padding: 24px 32px 48px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.device-title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
}
.tabs {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--line);
}
.tab {
  height: 40px;
  padding: 0 12px;
  border: 0;
  background: none;
  font-weight: 500;
  color: var(--ink-muted);
}
.tab.on {
  color: var(--ink);
  box-shadow: inset 0 -2px 0 var(--accent-text);
}
</style>
