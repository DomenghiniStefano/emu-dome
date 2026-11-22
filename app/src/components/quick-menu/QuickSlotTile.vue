<script setup lang="ts">
// One quick-save slot to choose in the menu (G07, G08): its picture, name and date; on the phone's
// sheet larger text and a wider picture. The chosen one takes the focus when the menu or the page
// opens. Its picture is the saves' SaveThumb, cropped. (Saves' SlotTile is a different card, with
// its own actions; the two looks are Cowork's to bring together.)
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SlotInfo } from '@/emu'
import { Card, CardVariant } from '@/components/ui/card'
import { cn } from '@/lib/cn'
import SaveThumb from '@/components/saves/SaveThumb.vue'
import { slotDateLabel } from '@/i18n/labels'
import { isEmptySlot, slotName } from './slots'
import { useSay } from '@/i18n/message'
import { FocusMark } from './view'

const props = defineProps<{
  info: SlotInfo
  chosen: boolean
}>()
defineEmits<{ pick: [] }>()

const { t } = useI18n()
const say = useSay()
const empty = computed(() => isEmptySlot(props.info))
const when = computed(() =>
  props.info.savedAt === null ? t('qm.empty') : slotDateLabel(props.info.savedAt),
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
    :variant="CardVariant.Tile"
    :selected="chosen"
    class="min-w-0 gap-4 touch:gap-3"
    role="radio"
    :aria-checked="chosen"
    @click="$emit('pick')"
  >
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
    <span class="truncate text-caption font-semibold touch:text-label-tight">
      {{ say(slotName(info.slot)) }}
    </span>
    <span class="truncate text-fine text-ink-muted tabular-nums touch:text-caption-tight">
      {{ when }}
    </span>
  </Card>
</template>
