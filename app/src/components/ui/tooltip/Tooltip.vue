<script setup lang="ts">
// A tooltip on Reka's Tooltip (Trello #104): mounted while it shows, over its element (`anchor`),
// the `--spacing-4` token away, in the page's CSS px at any interface scale (zoomedReference.ts);
// pushed back in from a window edge. It hangs from the element, it does not wrap it: the element
// keeps being its component's root (a Button's `$el` stays the <button>), and useTooltip says
// when it shows. Reka's trigger is a hidden stand-in placed by `reference`, portaled with the
// content: Reka closes a tooltip whose trigger is inside anything that scrolls, and nav.ts scrolls
// the focused element into view right after focusing it; the tooltip follows the element instead
// (Floating UI watches the element's scrolling ancestors). Reka's other dismissing (Esc, a press
// elsewhere) reaches the owner as `close`.
import { computed } from 'vue'
import {
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger,
} from 'reka-ui'
import { tokenPx, zoomedReference } from '@/components/ui/select/zoomedReference'
import { TOOLTIP_EDGE, TOOLTIP_GAP, tooltipVariants } from './variants'

const props = defineProps<{ anchor: HTMLElement; text: string }>()
const emit = defineEmits<{ close: [] }>()

const reference = computed(() => zoomedReference(props.anchor))
</script>

<template>
  <TooltipProvider :delay-duration="0">
    <TooltipRoot :open="true" disable-hoverable-content @update:open="o => o || emit('close')">
      <TooltipPortal>
        <TooltipTrigger as="span" hidden :reference="reference" />
        <TooltipContent
          data-slot="tooltip-content"
          side="top"
          :side-offset="tokenPx(TOOLTIP_GAP)"
          :collision-padding="tokenPx(TOOLTIP_EDGE)"
          :class="tooltipVariants()"
        >
          {{ text }}
        </TooltipContent>
      </TooltipPortal>
    </TooltipRoot>
  </TooltipProvider>
</template>
