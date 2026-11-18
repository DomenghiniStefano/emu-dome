<script setup lang="ts">
// L11: what the last scan added, one card per console, until closed.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { ScanSummary } from '../../library'
import { addedCards } from './libraryView'

const props = defineProps<{ summary: ScanSummary }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()

const title = computed(() =>
  t('library.added.title', { n: props.summary.added }, props.summary.added),
)
const cards = computed(() => addedCards(props.summary))
</script>

<template>
  <section class="glass flex flex-col gap-16 p-20" :aria-label="title">
    <div class="flex items-center justify-between gap-12">
      <h2 class="m-0 text-title">{{ title }}</h2>
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Icon"
        :aria-label="t('library.added.close')"
        :title="t('library.added.close')"
        @click="emit('close')"
      >
        <X :size="16" :stroke-width="1.75" aria-hidden="true" />
      </Button>
    </div>
    <div v-if="cards.length" class="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-12">
      <div v-for="c in cards" :key="c.key" class="rounded-10 border border-line bg-surface p-14">
        <!-- 32/38 semibold: `display` without its tracking (Cowork). -->
        <div class="text-[32px]/[38px] font-semibold tabular-nums">{{ c.count }}</div>
        <div class="text-ink-muted">{{ t(c.key) }}</div>
      </div>
    </div>
  </section>
</template>
