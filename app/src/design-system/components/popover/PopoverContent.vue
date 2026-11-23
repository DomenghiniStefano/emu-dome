<script setup lang="ts">
// The open popover: under its button (`anchor`), its right edge on the button's, the `--spacing-8`
// token away, in the page's CSS px at any interface scale (zoomedReference.ts); pushed back in
// from a window edge it would cross, never closer to it than that same gap.
//
// With the app's navigation:
// - its nav scope (PopoverNavScope) keeps the steps inside and focuses its `data-nav-initial`
//   control on opening (Reka's own first focus is turned off);
// - B is `back`, Esc and a click outside are Reka's: all three close it through `open`;
// - a press on the button itself is not «outside»: the button toggles it on its click;
// - the focus is not moved on closing: the screen puts it back on its button.
//
// A Panel is never taller than the room Reka leaves it (#106): the default slot is its body,
// which scrolls (nav.ts brings the focused row into view), and the `footer` slot (its actions)
// stays under it, always in view. The other variants lay out what they hold themselves.
import type { PopoverContentEmits } from 'reka-ui'
import { computed } from 'vue'
import { PopoverContent, PopoverPortal, useForwardPropsEmits } from 'reka-ui'
import {
  tokenPx,
  zoomedReference,
  zoomedViewport,
} from '@/design-system/components/select/zoomedReference'
import PopoverNavScope from './PopoverNavScope.vue'
import {
  POPOVER_GAP,
  POPOVER_PANEL_BODY,
  POPOVER_PANEL_FOOTER,
  PopoverVariant,
  popoverVariants,
} from './variants'

// Attributes (an `aria-label`) go to the panel, not to the portal around it.
defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<{ anchor: HTMLElement | null; variant?: PopoverVariant }>(),
  { variant: PopoverVariant.Panel },
)
const emits = defineEmits<PopoverContentEmits>()
const forwarded = useForwardPropsEmits({}, emits)

const reference = computed(() => (props.anchor ? zoomedReference(props.anchor) : undefined))
// Reka types its boundary as elements; Floating UI, which it hands it to, also takes a rectangle.
const boundary = zoomedViewport() as unknown as Element

function onPointerDownOutside(e: CustomEvent<{ originalEvent: PointerEvent }>) {
  const target = e.detail.originalEvent.target
  if (target instanceof Node && props.anchor?.contains(target)) e.preventDefault()
}
</script>

<template>
  <PopoverPortal>
    <PopoverContent
      data-slot="popover-content"
      v-bind="{ ...forwarded, ...$attrs }"
      side="bottom"
      align="end"
      :side-offset="tokenPx(POPOVER_GAP)"
      :collision-padding="tokenPx(POPOVER_GAP)"
      :collision-boundary="boundary"
      :reference="reference"
      :class="popoverVariants({ variant })"
      @open-auto-focus.prevent
      @close-auto-focus.prevent
      @pointer-down-outside="onPointerDownOutside"
    >
      <PopoverNavScope>
        <template v-if="variant === PopoverVariant.Panel">
          <div data-slot="popover-body" :class="POPOVER_PANEL_BODY">
            <slot />
          </div>
          <div v-if="$slots.footer" data-slot="popover-footer" :class="POPOVER_PANEL_FOOTER">
            <slot name="footer" />
          </div>
        </template>
        <slot v-else />
      </PopoverNavScope>
    </PopoverContent>
  </PopoverPortal>
</template>
