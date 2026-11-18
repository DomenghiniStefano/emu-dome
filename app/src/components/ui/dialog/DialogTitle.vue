<script setup lang="ts">
import type { DialogTitleProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { DialogTitle } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<DialogTitleProps & { class?: HTMLAttributes['class'] }>()
</script>

<template>
  <DialogTitle
    data-slot="dialog-title"
    :as="props.as"
    :as-child="props.asChild"
    :class="cn('m-0 text-dialog text-ink', props.class)"
  >
    <slot />
  </DialogTitle>
</template>
