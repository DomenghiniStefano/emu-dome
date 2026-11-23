<script setup lang="ts">
// L11: what the last scan added, the count per console in one row, until closed.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import type { ScanSummary } from '@/lib/library'
import { consoleCounts } from '@/lib/consoleCounts'
import LibraryBanner from './LibraryBanner.vue'

const props = defineProps<{ summary: ScanSummary }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()

const title = computed(() =>
  t('library.added.title', { n: props.summary.added }, props.summary.added),
)
const counts = computed(() => consoleCounts(props.summary.perConsole, { hideEmpty: true }))
</script>

<template>
  <LibraryBanner :aria-label="title">
    <span class="font-semibold">{{ title }}</span>
    <span class="min-w-0 grow basis-menu-min text-ink-muted tabular-nums">
      <template v-for="(c, i) in counts" :key="c.id">
        <template v-if="i > 0"> · </template>
        <span class="font-semibold text-ink">{{ c.count }}</span> {{ t(c.label) }}
      </template>
    </span>
    <Button
      class="ml-auto"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.IconSmall"
      :label="t('library.added.close')"
      @click="emit('close')"
    >
      <X :size="16" :stroke-width="1.75" aria-hidden="true" />
    </Button>
  </LibraryBanner>
</template>
