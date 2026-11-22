<script setup lang="ts">
// A row of a card list that leads one step down (Settings T01 «Usate più spesso», Controls C01
// «Rimappa i pulsanti»): its name, a line under it, the current value when given, a chevron. A
// button (nav.ts moves to it and clicks it), or `as="div"` for a row that only says where a thing
// is done (no hover, no pointer). A disabled one is half faded (ButtonFade.Dim). The value holds a
// third of the row at most, cut with an ellipsis (whole in its `title`), so the name keeps the rest.
import { ChevronRight } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'

withDefaults(
  defineProps<{
    name: string
    sub: string
    /** The current value, before the chevron. */
    value?: string
    as?: 'button' | 'div'
  }>(),
  { value: undefined, as: 'button' },
)
</script>

<template>
  <Button
    data-slot="link-row"
    :as="as"
    :variant="ButtonVariant.LinkRow"
    :size="ButtonSize.LinkRow"
    :fade="ButtonFade.Dim"
  >
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="font-medium text-ink">{{ name }}</span>
      <span class="text-caption">{{ sub }}</span>
    </span>
    <span v-if="value !== undefined" class="max-w-1/3 truncate" :title="value">{{ value }}</span>
    <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
  </Button>
</template>
