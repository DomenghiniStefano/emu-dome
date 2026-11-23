<script setup lang="ts">
import type { DialogDescriptionProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { DialogDescription } from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import type { DialogDescriptionSize } from './variants'
import { dialogDescriptionVariants } from './variants'

const props = defineProps<
  DialogDescriptionProps & { size?: DialogDescriptionSize; class?: HTMLAttributes['class'] }
>()
</script>

<template>
  <DialogDescription
    data-slot="dialog-description"
    :as="props.as"
    :as-child="props.asChild"
    :class="cn(dialogDescriptionVariants({ size }), props.class)"
  >
    <slot />
  </DialogDescription>
</template>
