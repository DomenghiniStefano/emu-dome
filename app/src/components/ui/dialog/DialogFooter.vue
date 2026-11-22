<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <div data-slot="dialog-footer" :class="cn('mt-4 flex flex-wrap justify-end gap-6', props.class)">
    <slot />
  </div>
</template>
