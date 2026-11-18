<script setup lang="ts">
import type { DialogDescriptionProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { DialogDescription } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<DialogDescriptionProps & { class?: HTMLAttributes['class'] }>()
</script>

<template>
  <DialogDescription
    data-slot="dialog-description"
    :as="props.as"
    :as-child="props.asChild"
    :class="cn('m-0 text-body text-ink-muted', props.class)"
  >
    <slot />
  </DialogDescription>
</template>
