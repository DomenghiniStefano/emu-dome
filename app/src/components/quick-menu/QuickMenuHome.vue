<script setup lang="ts">
// The quick menu's main screen under the quick saves (G07, «Tel-MenuRapido3»): the speed, then
// on desktop the tools and the rows «Schermi e immagine» and «Controlli», on the phone's sheet
// the grid that holds both. What a tool does is the menu's (QuickMenu.vue `runTool`).
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Gamepad2, Rows2 } from '@lucide/vue'
import type { ScreenStatus, SettingValue, Speed } from '@/emu'
import PageRow from './PageRow.vue'
import PhoneGrid from './PhoneGrid.vue'
import SpeedPicker from './SpeedPicker.vue'
import ToolTiles from './ToolTiles.vue'
import type { SpeedChoice } from './speedChoices'
import { quickTools, toolReasons, toolsNotYet, type Tool } from './tools'
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
  /** The open game's screens: swap and the lid pressed, the lid disabled where there is none. */
  screens: ScreenStatus | null
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
    screens: props.screens,
  }),
)
/** The note under the tools: why each disabled one cannot be used. */
const note = computed(() => {
  const names = toolsNotYet(tools.value)
    .map(key => t(key))
    .join(', ')
  const lines = toolReasons(tools.value).map(key => t(key))
  if (names) lines.unshift(t('qm.toolsNotYet', { names }))
  return lines.join(' ')
})
</script>

<template>
  <SpeedPicker
    :choices="choices"
    :speed="speed"
    :ff-option="ffOption"
    :slow-option="slowOption"
    @speed="s => emit('speed', s)"
    @set-speeds="e => emit('open', QuickMenuView.Speed, e)"
  />
  <template v-if="!sheet">
    <ToolTiles
      :tools="tools"
      :three-columns="dualScreen"
      :note="note"
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
    :note="note"
    @open="(view, e) => emit('open', view, e)"
    @run="(tool, e) => emit('run', tool, e)"
  />
</template>
