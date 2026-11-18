<script setup lang="ts">
// A button. Controller and keyboard: a native <button>, so nav.ts finds it (FOCUSABLE), moves to
// it spatially and clicks it on `confirm`; App.vue prevents the browser's own Enter activation,
// so one press is one click. Never a key handler of its own.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import type { ButtonSize, ButtonVariant } from './variants'
import { buttonVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: ButtonVariant
  size?: ButtonSize
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'button' })

// HTML's default type is `submit`: a Button inside a form must not submit it. Only where we
// render the <button> ourselves; `as-child` and `as="a"` keep the caller's element.
const type = computed(() => (props.as === 'button' && !props.asChild ? 'button' : undefined))
</script>

<template>
  <Primitive
    data-slot="button"
    :data-variant="variant"
    :data-size="size"
    :as="as"
    :as-child="asChild"
    :type="type"
    :class="cn(buttonVariants({ variant, size }), props.class)"
  >
    <slot />
  </Primitive>
</template>
