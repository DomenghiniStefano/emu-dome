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
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { controlsFocus } from '../gameDetails'
import { errorText } from '../i18n/labels'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import ControlsOverview from './ControlsOverview.vue'
import ControlsRemap from './ControlsRemap.vue'
import ControlsShortcuts from './ControlsShortcuts.vue'
import ControlsSticks from './ControlsSticks.vue'
import DeviceHeader from './controls/DeviceHeader.vue'
import NoController from './controls/NoController.vue'
import {
  captureDeviceOf,
  deviceDetail,
  deviceInUse,
  deviceName,
  familyOf,
  menuBackPos,
  type DeviceId,
} from './controls/devices'
import { cycle } from '../lib/cycle'
import { ControlsTab, tabOrRemap, tabsFor } from './controls/tabs'
import { useControlsData } from './controls/useControlsData'

const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

/** L07 «Rimappa i pulsanti per questo gioco»: the game this section opened for. */
const gameFocus = controlsFocus.value
controlsFocus.value = null

const View = { Overview: 'overview', Device: 'device' } as const
type View = (typeof View)[keyof typeof View]
/** C01, or one step below it (C02–C04): C02 when opened for a game. */
const view = ref<View>(gameFocus ? View.Device : View.Overview)
const root = ref<HTMLElement | null>(null)

/** The chosen device; null = the first controller. */
const chosen = ref<DeviceId | null>(null)
const { schema, globalRows, pads, states } = useControlsData({
  onError: e => emit('error', errorText(e)),
  onPadGone: id => {
    if (chosen.value === id) chosen.value = null
  },
})

const device = computed(() => deviceInUse(pads.value, chosen.value))
const family = computed(() => familyOf(device.value))
/** E05: no controller, and the keyboard not chosen. */
const empty = computed(() => device.value === null)
/** The menu's «back» (`pad.confirm`): B held 1 s cancels a capture. */
const captureDevice = computed(() =>
  captureDeviceOf(device.value, menuBackPos(globalRows.value['pad.confirm']?.value)),
)

// ---- Tabs ----
const tabs = computed(() => tabsFor(family.value))
const tab = ref<ControlsTab>(ControlsTab.Remap)
watch(tabs, list => (tab.value = tabOrRemap(list, tab.value)))
/** L1 / R1: the previous / next tab. */
function stepTabs(dir: -1 | 1) {
  if (empty.value || view.value !== View.Device) return
  tab.value = cycle(tabs.value, tab.value, dir)
}
/** Reka hands the tab chosen with a click, Enter or A. */
function chooseTab(value: string | number) {
  tab.value = tabOrRemap(tabs.value, value as ControlsTab)
}

/** The focus goes to the first control of the view just shown. */
async function focusView() {
  await nextTick()
  root.value?.querySelector<HTMLElement>('[data-nav-initial]:not(:disabled)')?.focus()
}
/** C01 → C02 / C03 / C04 on the device in use. */
function openTab(to: ControlsTab) {
  tab.value = tabOrRemap(tabs.value, to)
  view.value = View.Device
  void focusView()
}
/** Back from C02–C04 to C01; false when already there (the shell then goes to the rail). */
function back(): boolean {
  if (empty.value || view.value === View.Overview) return false
  view.value = View.Overview
  void focusView()
  return true
}
defineExpose({ tab: stepTabs, back })
</script>

<template>
  <div ref="root" class="flex min-h-0 flex-1">
    <NoController v-if="empty" @keyboard="chosen = 'keyboard'" />

    <ControlsOverview
      v-else-if="view === View.Overview && device"
      :pads="pads"
      :device="device"
      :states="states"
      :global-rows="globalRows"
      @pick="d => (chosen = d)"
      @open="openTab"
      @error="text => emit('error', text)"
    />

    <main v-else class="flex min-w-0 flex-1 flex-col gap-18 overflow-auto px-32 pt-24 pb-48">
      <DeviceHeader :name="deviceName(device)" :detail="deviceDetail(device)" @back="back" />
      <Tabs :model-value="tab" class="contents" @update:model-value="chooseTab">
        <TabsList :aria-label="t('controls.tabsLabel')">
          <TabsTrigger v-for="tb in tabs" :key="tb" :value="tb">
            {{ t(`controls.tab.${tb}`) }}
          </TabsTrigger>
        </TabsList>
      </Tabs>

      <template v-if="schema">
        <ControlsRemap
          v-if="tab === ControlsTab.Remap"
          :key="`remap-${family}`"
          :family="family"
          :device="captureDevice"
          :states="states"
          :global-rows="globalRows"
          :game="gameFocus"
          @error="text => emit('error', text)"
        />
        <ControlsShortcuts
          v-else-if="tab === ControlsTab.Shortcuts"
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
  </div>
</template>
