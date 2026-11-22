<script setup lang="ts">
// Kit proposal (not applied): quick-menu/ScreenCard.vue with the «in prova» look of TrialPanel,
// the accent edge on the plain surface, and the source line in the accent's text colour, so the
// card stops needing `violet-tint`; and the source line wraps to two lines instead of being cut
// (a locked card says why there). Everything else as in the component.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, ChevronRight } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { ScreenCard } from '@/components/quick-menu/screenCards'
import { useSay } from '@/i18n/message'

const props = defineProps<{ card: ScreenCard }>()

const { t } = useI18n()
const say = useSay()
const name = computed(() => say(props.card.name))
const sub = computed(() => {
  const s = props.card.sub
  return s.level ? t(s.key, { level: say(s.level) }) : t(s.key)
})
</script>

<template>
  <div
    class="flex min-w-0 flex-col gap-4 rounded-card border bg-surface px-12 py-10"
    :class="card.trial ? 'border-accent-text' : 'border-line'"
  >
    <span class="truncate text-caption-inline text-ink-muted">{{ name }}</span>
    <div class="flex items-center gap-4">
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Step"
        :disabled="!card.prev"
        :aria-label="t('qm.prev', { name })"
      >
        <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
      </Button>
      <span class="min-w-0 flex-1 truncate text-center font-semibold">
        {{ card.value ? say(card.value) : '' }}
      </span>
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Step"
        :disabled="!card.next"
        :aria-label="t('qm.next', { name })"
      >
        <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
      </Button>
    </div>
    <span
      class="line-clamp-2 text-overline-inline"
      :class="card.trial ? 'font-semibold text-accent-text' : 'text-ink-muted'"
    >
      {{ sub }}
    </span>
  </div>
</template>
