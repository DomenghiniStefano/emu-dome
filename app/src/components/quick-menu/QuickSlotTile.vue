<script setup lang="ts">
// One quick-save slot to choose in the menu (G07, G08): its picture, name and date; on the phone's
// sheet larger text and a wider picture. The chosen one takes the focus when the menu or the page
// opens. (Saves' SlotTile is a different card,
// with its own actions; the two looks are Cowork's to bring together.)
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SlotInfo } from '@/emu'
import { Card, CardVariant } from '@/components/ui/card'
import { isEmptySlot, slotDate, slotName } from './slots'
import { useSay } from './useSay'
import { FocusMark } from './view'

const props = defineProps<{
  info: SlotInfo
  chosen: boolean
}>()
defineEmits<{ pick: [] }>()

const { t, locale } = useI18n()
const say = useSay()
const empty = computed(() => isEmptySlot(props.info))
const when = computed(() => slotDate(props.info, locale.value) ?? t('qm.empty'))
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
    <span
      class="block aspect-3/2 w-full overflow-hidden rounded-6 bg-surface-raised touch:aspect-2/1"
      :class="{ 'border border-dashed border-line-strong': empty }"
      aria-hidden="true"
    >
      <img
        v-if="info.thumbUrl"
        :src="info.thumbUrl"
        alt=""
        class="pixelated block size-full object-cover"
      />
    </span>
    <span class="truncate text-caption font-semibold touch:text-label-tight">
      {{ say(slotName(info.slot)) }}
    </span>
    <span class="truncate text-fine text-ink-muted tabular-nums touch:text-caption-tight">
      {{ when }}
    </span>
  </Card>
</template>
