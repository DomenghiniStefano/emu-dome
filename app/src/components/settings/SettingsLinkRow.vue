<script setup lang="ts">
// T01: a row that opens a page (a cards choice, which does not fit a row; a recent change): the
// setting's name, a line under it, the current value when given, a chevron.
import { ChevronRight } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{ name: string; sub: string; value?: string }>()
</script>

<template>
  <Button :variant="ButtonVariant.LinkRow" :size="ButtonSize.LinkRow">
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="font-medium text-ink">{{ name }}</span>
      <span class="text-caption">{{ sub }}</span>
    </span>
    <span v-if="value !== undefined">{{ value }}</span>
    <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
  </Button>
</template>
