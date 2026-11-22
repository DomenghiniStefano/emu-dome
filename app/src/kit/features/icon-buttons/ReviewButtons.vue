<script setup lang="ts">
// One side of a #104 review issue: its rows of buttons, now or as proposed, each drawn with the
// real Button. A row beside a list item draws the item's words first, as the row does in the app;
// a phone row is drawn at the touch density.
import { Button } from '@/components/ui/button'
import type { ButtonSpec, ReviewRow } from './review'

const props = defineProps<{ rows: ReviewRow[]; side: 'now' | 'next' }>()

const spec = (pair: { now: ButtonSpec; next: ButtonSpec }) =>
  props.side === 'now' ? pair.now : pair.next
</script>

<template>
  <div class="flex w-full flex-col gap-8">
    <template v-for="(row, i) in rows" :key="i">
      <span v-if="row.phone" class="text-caption text-ink-faint">Telefono</span>
      <div
        :data-density="row.phone ? 'touch' : undefined"
        class="flex flex-wrap items-center gap-8"
        :class="
          row.context &&
          'rounded-card border border-line bg-surface-raised py-4 pr-4 pl-control-pad'
        "
      >
        <span v-if="row.context" class="min-w-0 flex-1 truncate text-body text-ink">
          {{ row.context }}
        </span>
        <template v-for="(pair, j) in row.buttons" :key="j">
          <Button
            v-if="spec(pair).iconOnly"
            :variant="spec(pair).variant"
            :size="spec(pair).size"
            :icon="spec(pair).icon"
            :icon-filled="spec(pair).filled"
            :label="spec(pair).text"
          />
          <Button
            v-else
            :variant="spec(pair).variant"
            :size="spec(pair).size"
            :icon="spec(pair).icon"
            :icon-filled="spec(pair).filled"
          >
            {{ spec(pair).text }}
          </Button>
        </template>
      </div>
    </template>
  </div>
</template>
