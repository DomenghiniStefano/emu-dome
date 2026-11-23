<script setup lang="ts">
// The open list (SettingSelectMenu): thick glass under the field, at least as wide as the field,
// 320px at most before it scrolls, its right edge on the field's. With `anchor` it hangs from that
// element instead (a choice inside a title: the title's block), at least as wide as it. The
// options take the text colour of where the field sits (muted in a muted sentence, accent in a
// chosen card), as the list drawn in place used to: the portal would otherwise lose it.
import type { SelectContentEmits, SelectContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import {
  injectSelectRootContext,
  SelectContent,
  SelectPortal,
  SelectViewport,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/design-system/lib/cn'
import SelectNavScope from './SelectNavScope.vue'
import { zoomedBoundary, zoomedReference } from './zoomedReference'

const props = withDefaults(
  defineProps<
    SelectContentProps & { anchor?: HTMLElement | null; class?: HTMLAttributes['class'] }
  >(),
  { position: 'popper', side: 'bottom', align: 'end', sideOffset: 4 },
)
const emits = defineEmits<SelectContentEmits>()

const delegated = computed(() => {
  const { class: _class, anchor: _anchor, ...rest } = props
  return rest
})
const forwarded = useForwardPropsEmits(delegated, emits)

// The trigger (or the anchor) in CSS px, so the list lands under it at any interface scale, and
// the window in CSS px as the edge it is kept inside (zoomedReference.ts, #107).
const boundary = zoomedBoundary()
const select = injectSelectRootContext()
const reference = computed(() => {
  const el = props.anchor ?? select.triggerElement.value
  return el ? zoomedReference(el) : undefined
})
/** The field's text colour, read each time the list opens. */
const ink = computed(() => {
  const el = select.triggerElement.value
  return select.open.value && el ? getComputedStyle(el).color : undefined
})
</script>

<template>
  <SelectPortal>
    <SelectContent
      data-slot="select-content"
      v-bind="forwarded"
      :collision-boundary="boundary"
      :reference="reference"
      :style="{ color: ink }"
      :class="
        cn(
          'glass-thick z-menu max-h-menu overflow-hidden',
          anchor ? 'min-w-(--reka-popper-anchor-width)' : 'min-w-(--reka-select-trigger-width)',
          props.class,
        )
      "
    >
      <SelectNavScope>
        <SelectViewport class="flex flex-col gap-px p-2">
          <slot />
        </SelectViewport>
      </SelectNavScope>
    </SelectContent>
  </SelectPortal>
</template>
