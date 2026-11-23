<script setup lang="ts">
// «Gestisci lo spazio» S1, the space card on top (the owner's mockup): «380 MB liberi», «di
// 128 GB», one bar split into the games, the quick saves, EmuDome's other files and the other
// apps (the rest of the bar is the free room), and its legend with each part's size. Without
// the disk's size (the system did not say) only EmuDome's parts show.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { StorageBreakdown } from '@/emu'
import { Card } from '@/design-system/components/card'
import { sizeLabel } from '@/i18n/labels'
import { barSlices, type BarPart } from './storage'

const props = defineProps<{ breakdown: StorageBreakdown }>()
const { t } = useI18n()

/**
 * Each part's colour, in the bar and in the legend: EmuDome's in amber and light grey, the other
 * apps darker, the free room the track itself.
 */
const TONE: Record<BarPart, string> = {
  games: 'bg-accent',
  quick: 'bg-accent-deep',
  other: 'bg-ink-muted',
  apps: 'bg-line-strong',
}

const slices = computed(() =>
  barSlices(props.breakdown).filter(s => s.part !== 'apps' || props.breakdown.total !== null),
)
</script>

<template>
  <Card class="gap-8">
    <div class="flex items-baseline justify-between gap-8">
      <span v-if="breakdown.free !== null" class="font-semibold tabular-nums">
        {{ t('storage.page.free', { size: sizeLabel(breakdown.free) }) }}
      </span>
      <span v-if="breakdown.total !== null" class="text-caption text-ink-muted tabular-nums">
        {{ t('storage.page.of', { size: sizeLabel(breakdown.total) }) }}
      </span>
    </div>
    <!-- The widths are the parts' shares of the disk: a measured value, not a design one. -->
    <div class="flex h-8 overflow-hidden rounded-badge bg-surface-raised" aria-hidden="true">
      <span
        v-for="s in slices"
        :key="s.part"
        :class="['block h-full', TONE[s.part]]"
        :style="{ width: `${s.percent}%` }"
      ></span>
    </div>
    <ul class="m-0 flex list-none flex-wrap gap-x-12 gap-y-2 p-0 text-caption text-ink-muted">
      <li v-for="s in slices" :key="s.part" class="flex items-center gap-4 tabular-nums">
        <span :class="['size-8 flex-none rounded-badge', TONE[s.part]]" aria-hidden="true"></span>
        {{
          s.part === 'apps'
            ? t('storage.page.parts.apps')
            : t(`storage.page.parts.${s.part}`, { size: sizeLabel(s.bytes) })
        }}
      </li>
    </ul>
  </Card>
</template>
