<script setup lang="ts">
// A text input. A native <input>, so nav.ts finds it (FOCUSABLE) and the keyboard types into it;
// `v-model` is its text, updated on every keystroke. A field that saves only when left (a
// setting) listens to `change` instead, which falls through to the <input>.
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import type { InputVariant } from './variants'
import { inputVariants } from './variants'

const props = defineProps<{ variant?: InputVariant; class?: HTMLAttributes['class'] }>()
const model = defineModel<string>()
</script>

<template>
  <input
    v-model="model"
    data-slot="input"
    :data-variant="variant"
    :class="cn(inputVariants({ variant }), props.class)"
  />
</template>
