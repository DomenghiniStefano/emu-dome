<script setup lang="ts">
// «Prova i pulsanti» (C01 beside the shortcuts, C02 beside the remap table): its label, what to
// do when there is something to say (C02: the capture's hint), the 12-button tester, and what the
// last press did (`last`; `idle` until the first press).
import { useI18n } from 'vue-i18n'
import { Card } from '@/design-system/components/card'
import { Heading, HeadingVariant } from '@/design-system/components/heading'
import ButtonTester from './ButtonTester.vue'

defineProps<{
  hint?: string
  cells: { id: string; label: string }[]
  held: Set<string>
  last: string
  /** What the line under the tester says before any press. */
  idle: string
}>()
const { t } = useI18n()
</script>

<template>
  <Card class="gap-8">
    <Heading :variant="HeadingVariant.Overline">{{ t('controls.tester') }}</Heading>
    <div v-if="hint" class="text-ink-muted">{{ hint }}</div>
    <ButtonTester
      class="self-center"
      :cells="cells"
      :held="held"
      :label="t('controls.overview.outline')"
    />
    <div class="text-caption-tall text-ink-muted" aria-live="polite">
      {{ last || idle }}
    </div>
  </Card>
</template>
