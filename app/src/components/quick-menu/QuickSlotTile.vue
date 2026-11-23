<script setup lang="ts">
// One quick-save slot to choose in the menu (G07, G08): its picture, name and date; on the phone's
// sheet larger text and a wider picture. The chosen one takes the focus when the menu or the page
// opens. Its picture is the saves' SaveThumb, cropped. (Saves' SlotTile is a different card, with
// its own actions; the two looks are still to be brought together (design).)
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SlotInfo } from '@/emu'
import { Badge, BadgeSize } from '@/components/ui/badge'
import { Card, CardVariant } from '@/components/ui/card'
import { cn } from '@/lib/cn'
import SaveThumb from '@/components/saves/SaveThumb.vue'
import { slotDateLabel, slotTimeLabel } from '@/i18n/labels'
import { isEmptySlot, slotName } from './slots'
import { useSay } from '@/i18n/message'
import { FocusMark } from './view'

const props = defineProps<{
  info: SlotInfo
  chosen: boolean
  /** A tag over the picture (the phone's «Auto», «Rapido»), already translated. */
  tag?: string
  /** The card's look: Tile (G07, G08), TileQuiet on the phone's «Salvataggi» (no amber). */
  variant?: CardVariant
  /** The date as the time alone today, else the short date (the phone's narrow tiles). */
  shortDate?: boolean
}>()
defineEmits<{ pick: [] }>()

const { t } = useI18n()
const say = useSay()
const empty = computed(() => isEmptySlot(props.info))
const when = computed(() =>
  props.info.savedAt === null
    ? t('qm.empty')
    : props.shortDate
      ? slotTimeLabel(props.info.savedAt)
      : slotDateLabel(props.info.savedAt),
)
const marks = computed(() =>
  props.chosen
    ? { [FocusMark.SubInitial]: '', 'data-nav-initial': '', [FocusMark.ChosenSlot]: '' }
    : {},
)
</script>

<template>
  <Card
    as="button"
    v-bind="marks"
    :variant="variant ?? CardVariant.Tile"
    :selected="chosen"
    class="min-w-0 gap-4 touch:gap-3"
    role="radio"
    :aria-checked="chosen"
    @click="$emit('pick')"
  >
    <span class="relative block w-full">
      <SaveThumb
        :src="info.thumbUrl"
        crop
        :class="
          cn(
            'aspect-3/2 w-full rounded-badge bg-surface-raised touch:aspect-2/1',
            empty && 'border border-dashed border-line-strong',
          )
        "
      />
      <Badge v-if="tag" :size="BadgeSize.Tag" class="absolute top-2 left-2">{{ tag }}</Badge>
    </span>
    <span class="truncate text-caption font-semibold touch:text-label-tight">
      {{ say(slotName(info.slot)) }}
    </span>
    <span class="truncate text-fine text-ink-muted tabular-nums touch:text-caption-tight">
      {{ when }}
    </span>
  </Card>
</template>
