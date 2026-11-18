<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <span data-slot="card-title" :class="cn('text-heading text-ink', props.class)">
    <slot />
  </span>
</template>
