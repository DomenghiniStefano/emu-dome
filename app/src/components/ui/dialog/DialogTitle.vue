<script setup lang="ts">
import type { DialogTitleProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { DialogTitle } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { DialogTone } from './variants'
import { dialogTitleVariants } from './variants'

const props = defineProps<
  DialogTitleProps & { tone?: DialogTone; class?: HTMLAttributes['class'] }
>()
</script>

<template>
  <DialogTitle
    data-slot="dialog-title"
    :as="props.as"
    :as-child="props.asChild"
    :class="cn(dialogTitleVariants({ tone }), props.class)"
  >
    <slot />
  </DialogTitle>
</template>
