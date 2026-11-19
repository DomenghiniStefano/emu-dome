<script setup lang="ts">
// One quick-save slot to choose in the menu (G07, G08): its picture, name and date. The chosen
// one takes the focus when the menu or the page opens. (Saves' SlotTile is a different card,
// with its own actions; the two looks are Cowork's to bring together.)
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { SlotInfo } from '@/emu'
import { Card, CardVariant } from '@/components/ui/card'
import { slotDateLabel } from '@/i18n/labels'
import { isEmptySlot, slotName } from './slots'
import { useSay } from '@/i18n/message'
import { FocusMark } from './view'

const props = defineProps<{
  info: SlotInfo
  chosen: boolean
  /** The phone's sheet: larger text, a wider picture. */
  touch?: boolean
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
    :variant="touch ? CardVariant.TileTouch : CardVariant.Tile"
    :selected="chosen"
    class="min-w-0"
    :class="touch ? 'gap-[3px]' : 'gap-4'"
    role="radio"
    :aria-checked="chosen"
    @click="$emit('pick')"
  >
    <span
      class="block w-full overflow-hidden rounded-6 bg-surface-raised"
      :class="[
        touch ? 'aspect-2/1' : 'aspect-3/2',
        { 'border border-dashed border-line-strong': empty },
      ]"
      aria-hidden="true"
    >
      <img
        v-if="info.thumbUrl"
        :src="info.thumbUrl"
        alt=""
        class="block size-full object-cover [image-rendering:pixelated]"
      />
    </span>
    <span
      class="truncate font-semibold"
      :class="touch ? 'text-(length:--text-label) leading-[16px]' : 'text-caption'"
    >
      {{ say(slotName(info.slot)) }}
    </span>
    <span
      class="truncate text-ink-muted tabular-nums"
      :class="
        touch
          ? 'text-(length:--text-caption) leading-[14px]'
          : 'text-(length:--text-overline) leading-[14px]'
      "
    >
      {{ when }}
    </span>
  </Card>
</template>
