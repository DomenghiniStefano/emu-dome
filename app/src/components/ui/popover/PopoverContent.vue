<script setup lang="ts">
// The open popover: under its button (`anchor`), its right edge on the button's, the `--spacing-8`
// token away, in the page's CSS px at any interface scale (zoomedReference.ts).
//
// With the app's navigation:
// - its nav scope (PopoverNavScope) keeps the steps inside and focuses its `data-nav-initial`
//   control on opening (Reka's own first focus is turned off);
// - B is `back`, Esc and a click outside are Reka's: all three close it through `open`;
// - a press on the button itself is not «outside»: the button toggles it on its click;
// - the focus is not moved on closing: the screen puts it back on its button.
import type { PopoverContentEmits } from 'reka-ui'
import { computed } from 'vue'
import { PopoverContent, PopoverPortal, useForwardPropsEmits } from 'reka-ui'
import { tokenPx, zoomedReference } from '@/components/ui/select/zoomedReference'
import PopoverNavScope from './PopoverNavScope.vue'
import { POPOVER_GAP, PopoverVariant, popoverVariants } from './variants'

// Attributes (an `aria-label`) go to the panel, not to the portal around it.
defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<{ anchor: HTMLElement | null; variant?: PopoverVariant }>(),
  { variant: PopoverVariant.Panel },
)
const emits = defineEmits<PopoverContentEmits>()
const forwarded = useForwardPropsEmits({}, emits)

const reference = computed(() => (props.anchor ? zoomedReference(props.anchor) : undefined))

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
      :reference="reference"
      :class="popoverVariants({ variant })"
      @open-auto-focus.prevent
      @close-auto-focus.prevent
      @pointer-down-outside="onPointerDownOutside"
    >
      <PopoverNavScope>
        <slot />
      </PopoverNavScope>
    </PopoverContent>
  </PopoverPortal>
</template>
