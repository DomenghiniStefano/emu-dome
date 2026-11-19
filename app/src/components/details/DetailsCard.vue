<script setup lang="ts">
// A card of the game details' tabs: the Surface card, rounder and a little tighter on the phone
// (board «Tel-Dettagli»). The gap between its parts is the caller's.
import type { HTMLAttributes } from 'vue'
import { Card } from '@/components/ui/card'
import { cn } from '@/lib/cn'

const props = defineProps<{
  phone: boolean
  /** The element it renders (`aside` for a side card). */
  as?: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Card :as="as ?? 'div'" :class="cn(phone && 'rounded-16 p-14', props.class)">
    <slot />
  </Card>
</template>
