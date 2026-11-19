<script setup lang="ts">
// A search field: magnifier, the label for screen readers, and a text input (`v-model`). The
// input is native, so nav.ts lands on it like any input and the arrows stay its own; the global
// focus ring shows on it while moving with the controller or the keyboard.
import type { HTMLAttributes } from 'vue'
import { Search } from '@lucide/vue'
import { cn } from '@/lib/cn'
import type { SearchFieldSize } from './variants'
import { searchFieldVariants } from './variants'

const props = defineProps<{
  /** What is searched, for screen readers (the field shows the placeholder). */
  label: string
  placeholder?: string
  size?: SearchFieldSize
  class?: HTMLAttributes['class']
}>()
const query = defineModel<string>({ required: true })
</script>

<template>
  <label data-slot="search-field" :class="cn(searchFieldVariants({ size }), props.class)">
    <Search :size="14" :stroke-width="1.75" aria-hidden="true" />
    <span class="sr-only">{{ label }}</span>
    <input
      v-model="query"
      type="search"
      :placeholder="placeholder"
      class="min-w-0 flex-1 border-0 bg-transparent text-ink outline-none [font:inherit]"
    />
  </label>
</template>
