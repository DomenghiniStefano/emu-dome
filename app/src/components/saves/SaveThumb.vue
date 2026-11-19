<script setup lang="ts">
// The last screen of a save (a quick save's thumbnail), letterboxed in black and drawn with crisp
// pixels; empty when there is none. The caller gives its shape (aspect ratio and radius), and may
// swap the black for a surface (an empty slot).
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{ src?: string | null; class?: HTMLAttributes['class'] }>()
</script>

<template>
  <span :class="cn('block overflow-hidden bg-letterbox', props.class)" aria-hidden="true">
    <img
      v-if="src"
      :src="src"
      alt=""
      class="block h-full w-full object-contain [image-rendering:pixelated]"
    />
  </span>
</template>
