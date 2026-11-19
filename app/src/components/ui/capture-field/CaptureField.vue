<script setup lang="ts">
// The capture field (design system «CaptureField»): a button that names what a row is bound to
// and, once pressed, listens for the next button or key (the screen runs the capture, `useCapture`
// in src/controls.ts, and says so with `listening`). A native <button>, so nav.ts moves to it and
// clicks it on `confirm` like any button.
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { CaptureFieldVariant } from './variants'
import { captureFieldVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: CaptureFieldVariant
  /** Waiting for the button or key to bind. */
  listening?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'button' })
</script>

<template>
  <Primitive
    data-slot="capture-field"
    :as="as"
    :as-child="asChild"
    :type="as === 'button' && !asChild ? 'button' : undefined"
    :data-listening="listening ? '' : undefined"
    :class="cn(captureFieldVariants({ variant }), props.class)"
  >
    <slot />
  </Primitive>
</template>
