<script setup lang="ts">
// A one-line text field (DetailsFiles «Nome mostrato»): 32px, edged in line-strong on
// surface-raised, the placeholder muted. shadcn-vue's Input: a native <input>, so nav.ts finds it
// (FOCUSABLE) and the keyboard types into it; every attribute (type, maxlength, placeholder,
// listeners) goes to the <input>.
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{ class?: HTMLAttributes['class'] }>()
const model = defineModel<string>({ required: true })
</script>

<template>
  <input
    v-model="model"
    data-slot="input"
    :class="
      cn(
        'h-control rounded-6 border border-line-strong bg-surface-raised px-10 py-0 font-sans text-body text-ink placeholder:text-ink-muted disabled:cursor-not-allowed disabled:opacity-disabled',
        props.class,
      )
    "
  />
</template>
