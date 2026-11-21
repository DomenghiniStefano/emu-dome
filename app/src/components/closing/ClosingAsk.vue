<script setup lang="ts">
// The question when closing a game does not end well (#86, owner's choice 3a: the strip grows into
// a card, in the same place). Saving is slow: «Close without saving» / «Wait» (focus and B on
// «Wait»). A save failed: what failed, then «Close without saving» (focus and B on it). Only the
// look and its controller scope: ClosingOverlay.vue places it.
import { ref, useId } from 'vue'
import { TriangleAlert } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { errorText } from '@/i18n/labels'
import { useNavScope, useNavSection } from '@/nav'
import type { ClosingAsk } from './closing'

const props = withDefaults(
  defineProps<{
    ask: ClosingAsk
    /**
     * Takes the focus and the steps when shown. Off on the Kit page, which shows several among the
     * other primitives: there it is a part of the page (a nav section), not a scope of its own.
     */
    focus?: boolean
  }>(),
  { focus: true },
)
const emit = defineEmits<{ wait: []; closeAnyway: [] }>()
const { t } = useI18n()

const root = ref<HTMLElement | null>(null)
const titleId = useId()
const back = () => (props.ask.kind === 'timeout' ? emit('wait') : emit('closeAnyway'))
// Read once: whether it is the app's question or a sample on the Kit page does not change.
if (props.focus) useNavScope(root, { back, autofocus: true })
else useNavSection(root, { back })
</script>

<template>
  <section
    ref="root"
    role="alertdialog"
    :aria-labelledby="titleId"
    class="glass-thick-flat flex w-full max-w-closing-card flex-col gap-16 rounded-panel border border-line p-card-pad shadow-panel"
  >
    <div class="flex items-start gap-12">
      <span
        class="grid size-control flex-none place-items-center rounded-full"
        :class="
          ask.kind === 'timeout' ? 'bg-warning-soft text-warning' : 'bg-surface-raised text-danger'
        "
        aria-hidden="true"
      >
        <TriangleAlert :size="18" :stroke-width="2" />
      </span>
      <div class="flex flex-col gap-4">
        <h2 :id="titleId" class="m-0 text-heading text-ink">
          {{ ask.kind === 'timeout' ? t('closing.timeoutTitle') : t('closing.failedTitle') }}
        </h2>
        <p v-if="ask.kind === 'timeout'" class="m-0 text-body text-ink-muted">
          {{ t('closing.timeoutText') }}
        </p>
        <template v-else>
          <p v-if="ask.error" class="m-0 text-body text-ink-muted">{{ errorText(ask.error) }}</p>
          <p class="m-0 text-body text-ink-muted">{{ t('closing.failedText') }}</p>
        </template>
      </div>
    </div>
    <div class="flex flex-wrap justify-end gap-8">
      <Button
        :variant="ButtonVariant.Danger"
        :size="ButtonSize.Medium"
        :data-nav-initial="ask.kind === 'failed' ? '' : undefined"
        @click="emit('closeAnyway')"
      >
        {{ t('closing.closeAnyway') }}
      </Button>
      <Button
        v-if="ask.kind === 'timeout'"
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        data-nav-initial
        @click="emit('wait')"
      >
        {{ t('closing.wait') }}
      </Button>
    </div>
  </section>
</template>
