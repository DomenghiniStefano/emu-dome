<script setup lang="ts">
// L11: what the last scan added, one card per console, until closed.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { StatTile } from '@/components/ui/stat-tile'
import type { ScanSummary } from '../../library'
import { consoleCounts } from '../../lib/consoleCounts'
import { Heading } from '@/components/ui/heading'

const props = defineProps<{ summary: ScanSummary }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()

const title = computed(() =>
  t('library.added.title', { n: props.summary.added }, props.summary.added),
)
const cards = computed(() => consoleCounts(props.summary.perConsole, { hideEmpty: true }))
</script>

<template>
  <section class="glass flex flex-col gap-16 p-20" :aria-label="title">
    <div class="flex items-center justify-between gap-12">
      <Heading>{{ title }}</Heading>
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
    <div
      v-if="cards.length"
      class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-stat-min),1fr))] gap-12"
    >
      <StatTile v-for="c in cards" :key="c.id" :value="c.count" :label="t(c.label)" />
    </div>
  </section>
</template>
