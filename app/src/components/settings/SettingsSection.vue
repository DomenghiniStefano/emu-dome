<script setup lang="ts">
// A heading of a settings page and its rows on one card, in columns on a wide window
// (RowColumns). With `note`, a line beside the heading.
import { Card, CardVariant } from '@/design-system/components/card'
import { LabelledBlock } from '@/design-system/components/labelled-block'
import RowColumns from './RowColumns.vue'
defineProps<{ title: string; note?: string }>()
</script>

<template>
  <LabelledBlock :title="title" title-as="h3">
    <template v-if="note" #aside>
      <span class="text-caption text-ink-muted">{{ note }}</span>
    </template>
    <Card :variant="CardVariant.List">
      <RowColumns><slot /></RowColumns>
    </Card>
  </LabelledBlock>
</template>
