<script setup lang="ts">
// One G09 card: the setting's name, ‹ value ›, and where the value comes from. A change «in
// prova» is marked with the accent.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, ChevronRight } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { ScreenCard } from './screenCards'
import { useSay } from '@/i18n/message'
import { FocusMark } from './view'

const props = defineProps<{
  card: ScreenCard
  /** The page's first control: its ‹ takes the focus when the page opens. */
  first?: boolean
}>()

const { t } = useI18n()
const say = useSay()
const name = computed(() => say(props.card.name))
const sub = computed(() => {
  const s = props.card.sub
  return s.level ? t(s.key, { level: say(s.level) }) : t(s.key)
})
const firstMark = computed(() => (props.first ? { [FocusMark.SubInitial]: '' } : {}))
</script>

<template>
  <div
    class="flex min-w-0 flex-col gap-4 rounded-10 border px-12 py-10"
    :class="card.trial ? 'border-accent-text bg-violet-tint' : 'border-line bg-surface'"
  >
    <span class="truncate text-caption-inline text-ink-muted">{{ name }}</span>
    <div class="flex items-center gap-4">
      <Button
        v-bind="firstMark"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Step"
        :disabled="!card.prev"
        :aria-label="t('qm.prev', { name })"
        @click="card.prev?.()"
      >
        <ChevronLeft :size="16" class="touch:size-18" :stroke-width="1.75" aria-hidden="true" />
      </Button>
      <span class="min-w-0 flex-1 truncate text-center font-semibold">
        {{ card.value ? say(card.value) : '' }}
      </span>
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Step"
        :disabled="!card.next"
        :aria-label="t('qm.next', { name })"
        @click="card.next?.()"
      >
        <ChevronRight :size="16" class="touch:size-18" :stroke-width="1.75" aria-hidden="true" />
      </Button>
    </div>
    <span class="truncate text-overline-inline text-ink-muted">{{ sub }}</span>
  </div>
</template>
