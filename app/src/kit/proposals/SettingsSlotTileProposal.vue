<script setup lang="ts">
// Proposal for SlotTile and SaveThumb (Kit review, not applied): the buttons sit at the bottom of
// the tile (`mt-auto`), so a row of tiles keeps them on one line when one tile has «Potrebbe non
// caricarsi»; a save with no picture shows the raised surface with an icon, not a black box that
// reads as a black screen.
import { Ellipsis, ImageOff, TriangleAlert } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardPad } from '@/components/ui/card'
import SaveThumb from '@/components/saves/SaveThumb.vue'

withDefaults(
  defineProps<{ name: string; when: string; thumbUrl?: string | null; outdated?: boolean }>(),
  { thumbUrl: null, outdated: false },
)
</script>

<template>
  <Card :pad="CardPad.Tight" class="min-w-0 gap-6">
    <SaveThumb v-if="thumbUrl" :src="thumbUrl" class="aspect-4/3 rounded-4" />
    <span
      v-else
      class="flex aspect-4/3 items-center justify-center rounded-4 bg-surface-raised text-ink-faint"
      aria-hidden="true"
    >
      <ImageOff :size="20" :stroke-width="1.75" />
    </span>
    <span class="font-semibold">{{ name }}</span>
    <span class="text-caption text-ink-muted tabular-nums">{{ when }}</span>
    <span v-if="outdated" class="inline-flex items-center gap-4 text-caption text-warning">
      <TriangleAlert :size="12" :stroke-width="1.75" aria-hidden="true" />
      Potrebbe non caricarsi
    </span>
    <span class="mt-auto flex gap-6 pt-2">
      <Button
        :variant="ButtonVariant.SoftAccent"
        :size="ButtonSize.TileAction"
        class="min-w-0 flex-1"
      >
        Gioca da qui
      </Button>
      <Button :size="ButtonSize.TileAction" aria-label="Altro">
        <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
      </Button>
    </span>
  </Card>
</template>
