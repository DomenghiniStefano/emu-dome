<script setup lang="ts">
// A heading of a settings page and its rows on one card. With `note`, a line beside the heading.
import { Card, CardVariant } from '@/components/ui/card'
import { LabelledBlock } from '@/components/ui/labelled-block'
defineProps<{ title: string; note?: string }>()
</script>

<template>
  <LabelledBlock :title="title" title-as="h3">
    <template v-if="note" #aside>
      <span class="text-caption text-ink-muted">{{ note }}</span>
    </template>
    <Card :variant="CardVariant.List">
      <slot />
    </Card>
  </LabelledBlock>
</template>
