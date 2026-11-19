<script setup lang="ts">
// The last screen of a save (a quick save's thumbnail), drawn with crisp pixels; empty when there
// is none. Letterboxed in black by default; `crop` fills the box instead, cutting the edges (the
// small tiles of the quick menu and «Continua a giocare»). The caller gives its shape (aspect
// ratio, width, radius), and may swap the black for a surface or the game's colour.
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  src?: string | null
  crop?: boolean
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <span :class="cn('block overflow-hidden bg-letterbox', props.class)" aria-hidden="true">
    <img
      v-if="src"
      :src="src"
      alt=""
      class="pixelated block size-full"
      :class="crop ? 'object-cover' : 'object-contain'"
    />
  </span>
</template>
