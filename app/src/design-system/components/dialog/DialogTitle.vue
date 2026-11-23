<script setup lang="ts">
import type { DialogTitleProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { DialogTitle } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import type { DialogTitleSize, DialogTone } from './variants'
import { dialogTitleVariants } from './variants'

const props = defineProps<
  DialogTitleProps & {
    tone?: DialogTone
    size?: DialogTitleSize
    class?: HTMLAttributes['class']
  }
>()
</script>

<template>
  <DialogTitle
    data-slot="dialog-title"
    :as="props.as"
    :as-child="props.asChild"
    :class="cn(dialogTitleVariants({ tone, size }), props.class)"
  >
    <slot />
  </DialogTitle>
</template>
