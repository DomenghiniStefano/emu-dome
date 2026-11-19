<script setup lang="ts">
// A card of the game details' tabs: the Surface card, rounder and a little tighter on the phone
// (board «Tel-Dettagli», CardVariant.Detail). The gap between its parts is the caller's.
import type { HTMLAttributes } from 'vue'
import { Card, CardVariant } from '@/components/ui/card'

const props = defineProps<{
  /** The element it renders (`aside` for a side card). */
  as?: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Card :as="as ?? 'div'" :variant="CardVariant.Detail" :class="props.class">
    <slot />
  </Card>
</template>
