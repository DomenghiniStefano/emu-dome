<script setup lang="ts">
// A button. Controller and keyboard: a native <button>, so nav.ts finds it (FOCUSABLE), moves to
// it spatially and clicks it on `confirm`; App.vue prevents the browser's own Enter activation,
// so one press is one click. Never a key handler of its own.
//
// Its icon (Trello #104): `icon` draws a Lucide icon before the words, sized by the button's size
// (icon.ts). With `label` as well it draws the icon alone: the label is the button's accessible
// name and its tooltip (`title`; a `title` the caller passes wins), and the slot is not shown. An
// icon alone goes with a square size (Icon, IconSmall, IconRow, ToolIcon, IconTouch). Neither is
// for `as-child`, which renders the caller's element as it is.
import type { PrimitiveProps } from 'reka-ui'
import type { Component, HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { BUTTON_ICON_STROKE, buttonIconPx } from './icon'
import type { ButtonFade, ButtonSize, ButtonVariant } from './variants'
import { buttonVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: ButtonVariant
  size?: ButtonSize
  /** How it shows that it is disabled (ButtonFade). */
  fade?: ButtonFade
  /** A Lucide icon drawn before the words (or alone, with `label`). */
  icon?: Component
  /** The icon drawn as a solid shape instead of a line (the app's «Gioca» / «Riprendi» triangle). */
  iconFilled?: boolean
  /** Set: the icon alone, and this its accessible name and tooltip. */
  label?: string
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), { as: 'button' })

// HTML's default type is `submit`: a Button inside a form must not submit it. Only where we
// render the <button> ourselves; `as-child` and `as="a"` keep the caller's element.
const type = computed(() => (props.as === 'button' && !props.asChild ? 'button' : undefined))
const iconPx = computed(() => buttonIconPx(props.size))
</script>

<template>
  <Primitive
    data-slot="button"
    :data-variant="variant"
    :data-size="size"
    :as="as"
    :as-child="asChild"
    :type="type"
    :aria-label="label"
    :title="label"
    :class="cn(buttonVariants({ variant, size, fade }), props.class)"
  >
    <component
      :is="icon"
      v-if="icon"
      :size="iconPx"
      :stroke-width="iconFilled ? 0 : BUTTON_ICON_STROKE"
      :fill="iconFilled ? 'currentColor' : 'none'"
      aria-hidden="true"
    />
    <slot v-if="label === undefined" />
  </Primitive>
</template>
