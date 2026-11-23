<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/design-system/lib/cn'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <span data-slot="card-description" :class="cn('text-caption text-ink-muted', props.class)">
    <slot />
  </span>
</template>
