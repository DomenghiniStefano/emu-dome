<script setup lang="ts">
// L11: what the last scan added, the count per console in one row, until closed. Under it, the
// files of a folder just picked that were skipped, by reason (#127: an encrypted 3DS game, a file
// not recognised), each with what to do; a warning when nothing was added.
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
const skipped = computed(() => {
  const { encrypted, unrecognised } = props.summary.skipped
  return [
    encrypted > 0 && t('library.added.encrypted', { n: encrypted }, encrypted),
    unrecognised > 0 && t('library.added.unrecognised', { n: unrecognised }, unrecognised),
  ].filter((line): line is string => typeof line === 'string')
})
</script>

<template>
  <LibraryBanner :aria-label="title" :warning="summary.added === 0">
    <span class="font-semibold">{{ title }}</span>
    <span v-if="counts.length" class="min-w-0 grow basis-menu-min text-ink-muted tabular-nums">
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
    <p v-for="line in skipped" :key="line" class="m-0 basis-full text-caption text-ink-muted">
      {{ line }}
    </p>
  </LibraryBanner>
</template>
