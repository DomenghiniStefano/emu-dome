<script setup lang="ts">
// The open menu: under its button (`anchor`), its right edge on the button's, the `--spacing-6`
// token away, in the page's CSS px at any interface scale (zoomedReference.ts); pushed back in
// from a window edge it would cross, never closer to it than that same gap.
//
// With the app's navigation:
// - its nav scope (DropdownMenuNavScope) focuses the first item on opening (Reka's own first
//   focus is turned off); the steps move among the items (nav.ts, inside the scope), and Enter
//   is the item's (`ownsEnter`): A reaches Reka as Enter, which chooses the item once;
// - B is `back`, Esc and a click outside are Reka's: all three close it through `open`;
// - a press on the button itself is not «outside»: the button toggles it on its click;
// - the focus is not moved on closing: the screen puts it back on its button.
import type { DropdownMenuContentEmits } from 'reka-ui'
import { computed } from 'vue'
import { DropdownMenuContent, DropdownMenuPortal, useForwardPropsEmits } from 'reka-ui'
import {
  tokenPx,
  zoomedBoundary,
  zoomedReference,
} from '@/design-system/components/select/zoomedReference'
import DropdownMenuNavScope from './DropdownMenuNavScope.vue'
import { DROPDOWN_MENU_GAP, DropdownMenuVariant, dropdownMenuVariants } from './variants'

// Attributes (an `aria-label`) go to the menu, not to the portal around it.
defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<{ anchor: HTMLElement | null; variant?: DropdownMenuVariant }>(),
  { variant: DropdownMenuVariant.Actions },
)
const emits = defineEmits<DropdownMenuContentEmits>()
const forwarded = useForwardPropsEmits({}, emits)

const reference = computed(() => (props.anchor ? zoomedReference(props.anchor) : undefined))
const boundary = zoomedBoundary()

function onPointerDownOutside(e: CustomEvent<{ originalEvent: PointerEvent }>) {
  const target = e.detail.originalEvent.target
  if (target instanceof Node && props.anchor?.contains(target)) e.preventDefault()
}
</script>

<template>
  <DropdownMenuPortal>
    <DropdownMenuContent
      data-slot="dropdown-menu-content"
      v-bind="{ ...forwarded, ...$attrs }"
      side="bottom"
      align="end"
      :side-offset="tokenPx(DROPDOWN_MENU_GAP)"
      :collision-padding="tokenPx(DROPDOWN_MENU_GAP)"
      :collision-boundary="boundary"
      :reference="reference"
      :class="dropdownMenuVariants({ variant })"
      @open-auto-focus.prevent
      @close-auto-focus.prevent
      @pointer-down-outside="onPointerDownOutside"
    >
      <DropdownMenuNavScope>
        <slot />
      </DropdownMenuNavScope>
    </DropdownMenuContent>
  </DropdownMenuPortal>
</template>
