<script setup lang="ts">
// G09 on desktop: the menu becomes a bar at the bottom of the window and the game stays visible
// above it, shrunk by the height the bar covers (`inset`, px; 0 when the bar goes).
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BackButton from './BackButton.vue'
import ScreenCardView from './ScreenCard.vue'
import TrialActions from './TrialActions.vue'
import type { ScreenCard } from './screenCards'

defineProps<{
  cards: ScreenCard[]
  /** The changes «in prova», as the title over them says; '' with none. */
  trialTitle: string
  hint: string | null
}>()
const emit = defineEmits<{
  back: []
  discard: []
  commitGame: []
  inset: [number]
}>()

const { t } = useI18n()

const bar = ref<HTMLElement | null>(null)
/** The gap kept between the game and the bar. */
const GAP = 12
function reportInset() {
  const el = bar.value
  emit('inset', el ? Math.ceil(window.innerHeight - el.getBoundingClientRect().top + GAP) : 0)
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
  emit('inset', 0)
})
</script>

<template>
  <div ref="bar" class="flex flex-col gap-12 px-16 py-14">
    <div class="flex items-center gap-12">
      <BackButton :label="t('qm.label')" :hint="hint" @click="emit('back')" />
      <span class="text-heading">{{ t('qm.row.screens.name') }}</span>
      <span class="min-w-0 flex-1"></span>
      <span class="text-caption text-ink-muted">{{ t('qm.liveNote') }}</span>
    </div>
    <div class="grid grid-cols-4 gap-8">
      <ScreenCardView v-for="(c, i) in cards" :key="c.key" :card="c" :first="i === 0" />
    </div>
    <div v-if="trialTitle" class="flex items-center gap-8" role="status">
      <span class="min-w-0 flex-1 font-semibold">{{ trialTitle }}</span>
      <TrialActions @discard="emit('discard')" @commit-game="emit('commitGame')" />
    </div>
  </div>
</template>
