<script setup lang="ts">
// The quick menu's main screen under the quick saves (G07, «Tel-MenuRapido3»): the speed, then
// on desktop the tools and the rows «Schermi e immagine» and «Controlli», on the phone's sheet
// the grid that holds both. What a tool does is the menu's (QuickMenu.vue `runTool`).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Gamepad2, Rows2 } from '@lucide/vue'
import type { SettingValue, Speed } from '@/emu'
import PageRow from './PageRow.vue'
import PhoneGrid from './PhoneGrid.vue'
import SpeedPicker from './SpeedPicker.vue'
import ToolTiles from './ToolTiles.vue'
import type { SpeedChoice } from './speedChoices'
import { quickTools, toolsNotYet, type Tool } from './tools'
import { QuickMenuView } from './view'

const props = defineProps<{
  choices: SpeedChoice[]
  speed: Speed
  ffOption: SettingValue | null
  slowOption: SettingValue | null
  /** The phone held upright: the bottom sheet. */
  sheet: boolean
  /** The phone, upright or not (its own tools). */
  phone: boolean
  /** NDS and 3DS: their screen tools. */
  dualScreen: boolean
  muted: boolean
  fullscreen: boolean
}>()
const emit = defineEmits<{
  speed: [Speed]
  open: [QuickMenuView, Event]
  run: [Tool, Event]
}>()

const { t } = useI18n()

const tools = computed(() =>
  quickTools({
    phone: props.phone,
    dualScreen: props.dualScreen,
    muted: props.muted,
    fullscreen: props.fullscreen,
  }),
)
const notYet = computed(() => {
  const names = toolsNotYet(tools.value)
    .map(key => t(key))
    .join(', ')
  return names ? t('qm.toolsNotYet', { names }) : ''
})
</script>

<template>
  <SpeedPicker
    :choices="choices"
    :speed="speed"
    :ff-option="ffOption"
    :slow-option="slowOption"
    :touch="sheet"
    @speed="s => emit('speed', s)"
    @set-speeds="e => emit('open', QuickMenuView.Speed, e)"
  />
  <template v-if="!sheet">
    <ToolTiles
      :tools="tools"
      :three-columns="dualScreen"
      :not-yet="notYet"
      @run="(tool, e) => emit('run', tool, e)"
    />
    <nav class="grid grid-cols-[1fr_1fr] gap-8" :aria-label="t('qm.moreLabel')">
      <PageRow
        :icon="Rows2"
        :label="t('qm.row.screens.name')"
        @click="e => emit('open', QuickMenuView.Screens, e)"
      />
      <PageRow
        :icon="Gamepad2"
        :label="t('qm.row.controls.name')"
        @click="e => emit('open', QuickMenuView.Controls, e)"
      />
    </nav>
  </template>
  <PhoneGrid
    v-else
    :tools="tools"
    :not-yet="notYet"
    @open="(view, e) => emit('open', view, e)"
    @run="(tool, e) => emit('run', tool, e)"
  />
</template>
