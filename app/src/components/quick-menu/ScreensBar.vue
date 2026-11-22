<script setup lang="ts">
// G09 on desktop: the menu becomes a bar at the bottom of the window and the game stays visible
// above it, shrunk by the height the bar covers (`inset`, px; none when the bar goes). For a
// two-screen game the layout card comes first (five columns) and «Personalizza gli schermi»
// opens the C09 editor.
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Rows2 } from '@lucide/vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { bottomInset, NO_INSET, type InsetPx } from '@/components/game/inset'
import BackButton from './BackButton.vue'
import ScreenCardView from './ScreenCard.vue'
import ScreenPicks from './ScreenPicks.vue'
import TrialActions from './TrialActions.vue'
import type { ScreenCard } from './screenCards'
import type { ScreenPick, ScreenPickId } from './screenPicks'
import { FocusMark } from './view'

defineProps<{
  cards: ScreenCard[]
  picks: ScreenPick[]
  /** The changes «in prova», as the title over them says; '' with none. */
  trialTitle: string
  hint: string | null
  /** NDS and 3DS: the layout card (five columns) and «Personalizza gli schermi». */
  dualScreen: boolean
}>()
const emit = defineEmits<{
  back: []
  discard: []
  commitGame: []
  inset: [InsetPx]
  pick: [ScreenPickId]
  customise: [Event]
}>()

const { t } = useI18n()
const customiseMark = { [FocusMark.Customise]: '' }

const bar = ref<HTMLElement | null>(null)
/** The gap kept between the game and the bar. */
const GAP = 12
function reportInset() {
  const el = bar.value
  emit(
    'inset',
    el
      ? bottomInset(Math.ceil(window.innerHeight - el.getBoundingClientRect().top + GAP))
      : NO_INSET,
  )
}
let observer: ResizeObserver | null = null
onMounted(() => {
  if (!bar.value) return
  observer = new ResizeObserver(reportInset)
  observer.observe(bar.value)
  reportInset()
})
onBeforeUnmount(() => {
  observer?.disconnect()
  emit('inset', NO_INSET)
})
</script>

<template>
  <div ref="bar" class="flex flex-col gap-12 px-16 py-14">
    <div class="flex items-center gap-12">
      <BackButton :label="t('qm.label')" :hint="hint" @click="emit('back')" />
      <span class="text-heading">{{ t('qm.row.screens.name') }}</span>
      <span class="min-w-0 flex-1"></span>
      <span class="text-caption text-ink-muted">{{ t('qm.liveNote') }}</span>
      <Button
        v-if="dualScreen"
        v-bind="customiseMark"
        :variant="ButtonVariant.Soft"
        @click="emit('customise', $event)"
      >
        <Rows2 :size="16" :stroke-width="1.75" aria-hidden="true" />
        {{ t('settings.screens.custom.button') }}
      </Button>
    </div>
    <ScreenPicks :picks="picks" @pick="id => emit('pick', id)" />
    <div class="grid gap-8" :class="dualScreen ? 'grid-cols-5' : 'grid-cols-4'">
      <ScreenCardView v-for="(c, i) in cards" :key="c.key" :card="c" :first="i === 0" />
    </div>
    <div v-if="trialTitle" class="flex items-center gap-8" role="status">
      <span class="min-w-0 flex-1 font-semibold">{{ trialTitle }}</span>
      <TrialActions @discard="emit('discard')" @commit-game="emit('commitGame')" />
    </div>
  </div>
</template>
