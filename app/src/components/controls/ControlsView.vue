<script setup lang="ts">
// The Controls section (boards «Controlli» C01–C04, «Errori» E05). It opens on C01
// (ControlsOverview: device cards, where the device is chosen, shortcuts, tester, rows down); one
// step below, the chosen device with «‹ Controlli» back to C01 above its name and three tabs:
// button remapping (C02), shortcuts (C03), sticks and triggers (C04). No device column there
// (board C02, 18/11): the device is changed on C01. With no controller connected the section
// shows E05 until the keyboard is chosen (on a touch device E05 is a card above the touch controls
// row, #132).
//
// From Game details › Controlli (L07, the arrival's `game` from the shell's route) it opens on C02
// at «Solo questo gioco» for that game; from Settings' `pad.dev1` / `pad.dev2` (#112) on C02 at
// «Tutti i giochi», on the first controller or the keyboard.
//
// Controller: the shell's nav scope; this section's own steps (nav.ts `useNavSection`): L1 / R1
// change tab one step below C01, back goes from there to C01; on C01 back is left to the shell
// (the rail).
// The tester and the capture field read `emu://gamepad-state`, watched while this section is open.
//
// TODO(E06, Trello): unknown controller banner and guided setup (14 buttons, saved as an SDL
// mapping). SDL opens as gamepads only the controllers it has a mapping for, so an unknown one
// never reaches this list: detecting it needs the joystick events, not done yet.
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { GameFocus } from '@/lib/gameDetails'
import { useErrorSink } from '@/composables/useToasts'
import { useNavSection } from '@/nav'
import { Tabs, TabsList, TabsTrigger } from '@/design-system/components/tabs'
import ControlsOverview from './ControlsOverview.vue'
import ControlsRemap from './ControlsRemap.vue'
import ControlsShortcuts from './ControlsShortcuts.vue'
import ControlsSticks from './ControlsSticks.vue'
import DeviceHeader from './DeviceHeader.vue'
import NoController from './NoController.vue'
import {
  captureDeviceOf,
  deviceDetail,
  deviceInUse,
  deviceName,
  familyOf,
  menuBackPos,
  type DeviceId,
} from './devices'
import { cycle } from '@/lib/cycle'
import { isShown, lockReason } from '@/lib/settingsModel'
import { ControlsTab, tabOrRemap, tabsFor } from './tabs'
import { useControlsData } from './useControlsData'
import { SectionPage } from '@/design-system/components/section-page'

const props = defineProps<{
  /** Opened on C02 (the shell's route, read on mount): L07 «Rimappa i pulsanti per questo gioco»
   * for its `game`, or Settings' controller / keyboard rows at «Tutti i giochi» (`keyboard`). */
  arrival: { game: GameFocus | null; keyboard?: boolean } | null
}>()
const { t } = useI18n()

const arrival = props.arrival
const gameFocus = arrival?.game ?? null

const View = { Overview: 'overview', Device: 'device' } as const
type View = (typeof View)[keyof typeof View]
/** C01, or one step below it (C02–C04): C02 when opened for remapping. */
const view = ref<View>(arrival ? View.Device : View.Overview)
const root = ref<HTMLElement | null>(null)

/** The chosen device; null = the first controller. */
const chosen = ref<DeviceId | null>(arrival?.keyboard ? 'keyboard' : null)
const { schema, globalRows, pads, states } = useControlsData({
  onError: useErrorSink(),
  onPadGone: id => {
    if (chosen.value === id) chosen.value = null
  },
})

const device = computed(() => deviceInUse(pads.value, chosen.value))
/** C01's «Controlli touch» leads to the C06 editor where Settings' `touch.tedit` row opens it
 * (a phone; #112), and only says where it is done elsewhere. */
const touchEditable = computed(() => {
  const s = schema.value
  const e = s?.entries.find(x => x.key === 'touch.tedit')
  return !!s && !!e && isShown(e, s) && lockReason(e, s, 'global') === null
})
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
useNavSection(root, { tab: stepTabs, back })
</script>

<template>
  <div ref="root" class="flex min-h-0 flex-1">
    <NoController v-if="empty" :touch-editable="touchEditable" @keyboard="chosen = 'keyboard'" />

    <ControlsOverview
      v-else-if="view === View.Overview && device"
      :pads="pads"
      :device="device"
      :states="states"
      :global-rows="globalRows"
      :touch-editable="touchEditable"
      @pick="d => (chosen = d)"
      @open="openTab"
    />

    <SectionPage v-else as="main">
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
        />
        <ControlsShortcuts
          v-else-if="tab === ControlsTab.Shortcuts"
          :family="family"
          :device="captureDevice"
          :states="states"
          :schema="schema"
          :global-rows="globalRows"
        />
        <ControlsSticks
          v-else
          :device="captureDevice"
          :states="states"
          :schema="schema"
          :global-rows="globalRows"
        />
      </template>
    </SectionPage>
  </div>
</template>
