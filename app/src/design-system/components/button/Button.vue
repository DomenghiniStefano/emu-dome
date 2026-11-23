<script setup lang="ts">
// A button. Controller and keyboard: a native <button>, so nav.ts finds it (FOCUSABLE), moves to
// it spatially and clicks it on `confirm`; App.vue prevents the browser's own Enter activation,
// so one press is one click. Never a key handler of its own.
//
// Its icon (Trello #104): `icon` draws a Lucide icon before the words, sized by the button's size
// and one step larger on the phone (icon.ts). With `label` as well it draws the icon alone: the
// label is the button's accessible name and its tooltip (ui/tooltip: on a rest of the mouse and
// on the controller's or the keyboard's focus; a `title` the caller passes is shown there
// instead, and never as the browser's own tooltip too), and the slot is not shown. An icon alone
// goes with a square size (Icon, IconSmall, IconRow, RowAction, ToolIcon, IconTouch). `label`
// without `icon` names a button that draws its own picture in the slot (a glyph of its own size,
// the bell with its dot): the slot shows, with the same name and tooltip. Neither is for
// `as-child`, which renders the caller's element as it is.
import type { PrimitiveProps } from 'reka-ui'
import type { Component, HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { computed, useAttrs } from 'vue'
import { cn } from '@/design-system/lib/cn'
import { Tooltip, useTooltip } from '@/design-system/components/tooltip'
import { BUTTON_ICON_STROKE, buttonIconClass, buttonIconPx } from './icon'
import type { ButtonFade, ButtonSize, ButtonVariant } from './variants'
import { buttonVariants } from './variants'

// The caller's attributes go on the <button> as usual, bound by hand so that a `title` can be
// moved into the tooltip of an icon alone.
defineOptions({ inheritAttrs: false })

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
const attrs = useAttrs()

// HTML's default type is `submit`: a Button inside a form must not submit it. Only where we
// render the <button> ourselves; `as-child` and `as="a"` keep the caller's element.
const type = computed(() => (props.as === 'button' && !props.asChild ? 'button' : undefined))
const iconPx = computed(() => buttonIconPx(props.size))
const iconClass = computed(() => buttonIconClass(props.size))

const tip = useTooltip()
const tipOpen = tip.open
const tipAnchor = tip.anchor
const hasTip = computed(() => props.label !== undefined && !props.asChild)
const tipText = computed(() =>
  typeof attrs.title === 'string' && attrs.title !== '' ? attrs.title : props.label,
)
const bound = computed(() => {
  if (!hasTip.value) return attrs
  const rest: Record<string, unknown> = {}
  for (const [k, v] of Object.entries(attrs)) if (k !== 'title') rest[k] = v
  return rest
})
const listeners = computed(() => (hasTip.value ? tip.on : {}))
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
    v-bind="bound"
    :class="cn(buttonVariants({ variant, size, fade }), props.class)"
    v-on="listeners"
  >
    <component
      :is="icon"
      v-if="icon"
      :size="iconPx"
      :class="iconClass"
      :stroke-width="iconFilled ? 0 : BUTTON_ICON_STROKE"
      :fill="iconFilled ? 'currentColor' : 'none'"
      aria-hidden="true"
    />
    <slot v-if="label === undefined || !icon" />
    <Tooltip
      v-if="hasTip && tipOpen && tipAnchor && tipText"
      :anchor="tipAnchor"
      :text="tipText"
      @close="tip.hide"
    />
  </Primitive>
</template>
