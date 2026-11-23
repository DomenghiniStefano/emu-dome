<script setup lang="ts">
// The Kit page's index: every section by group, with how many issues are still open under it,
// and the filter that shows only the sections with something to decide. A click scrolls to the
// section (no anchor: the page's hash is `#kit`, which main.ts reads).
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/design-system/components/badge'
import { Button, ButtonVariant } from '@/design-system/components/button'
import { Segmented, SegmentedItem } from '@/design-system/components/segmented'
import type { KitSectionEntry } from '@/design-system/kit/review'
import { KIT_GROUP_TITLES, KitGroup } from '@/design-system/kit/review'

const props = defineProps<{ sections: KitSectionEntry[] }>()
const onlyOpen = defineModel<boolean>('onlyOpen', { required: true })

const filter = computed({
  get: () => (onlyOpen.value ? 'open' : 'all'),
  set: v => (onlyOpen.value = v === 'open'),
})
const groups = computed(() =>
  Object.values(KitGroup)
    .map(group => ({
      group,
      title: KIT_GROUP_TITLES[group],
      sections: props.sections.filter(s => s.group === group && (!onlyOpen.value || s.open > 0)),
    }))
    .filter(g => g.sections.length > 0),
)
const totalOpen = computed(() => props.sections.reduce((n, s) => n + s.open, 0))
const go = (id: string) => document.getElementById(id)?.scrollIntoView({ block: 'start' })
</script>

<template>
  <nav class="flex flex-col gap-8 rounded-card border border-line bg-surface p-card-pad">
    <div class="flex flex-wrap items-center gap-12">
      <h2 class="m-0 text-heading text-ink">Review</h2>
      <Badge :variant="totalOpen > 0 ? BadgeVariant.Warning : BadgeVariant.Success">
        {{ totalOpen }} da decidere
      </Badge>
      <Segmented v-model="filter" aria-label="Filtro review">
        <SegmentedItem value="all">Tutti</SegmentedItem>
        <SegmentedItem value="open">Con problemi</SegmentedItem>
      </Segmented>
    </div>
    <div class="grid grid-cols-[repeat(auto-fill,minmax(var(--spacing-field),1fr))] gap-16">
      <div v-for="g in groups" :key="g.group" class="flex flex-col gap-4">
        <h3 class="m-0 text-caption text-ink-faint">{{ g.title }}</h3>
        <Button
          v-for="s in g.sections"
          :key="s.id"
          :variant="ButtonVariant.Menu"
          class="justify-between"
          @click="go(s.id)"
        >
          {{ s.title }}
          <Badge v-if="s.open > 0" :variant="BadgeVariant.Warning">{{ s.open }}</Badge>
        </Button>
      </div>
    </div>
  </nav>
</template>
