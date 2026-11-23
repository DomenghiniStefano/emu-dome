<script setup lang="ts">
// Proposal for SettingCards (Kit review, not applied): every option of a short choice on one line
// (five fit at the page's width), so no card is left alone on a second line beside an empty band.
// The column's minimum is a narrower token (here the stat tiles' 160, standing in for a new
// `card-min-compact`); the preview keeps its height. Only a picture: no radio roles (the real
// component keeps them).
import { Card, CardDescription, CardVariant } from '@/design-system/components/card'
import { cardPreview } from '@/components/settings/settingRowModel'

defineProps<{
  settingKey: string
  options: { id: string; text: string; desc: string; notYet?: boolean }[]
  value: string
}>()
</script>

<template>
  <div class="grid grid-cols-[repeat(auto-fit,minmax(var(--spacing-stat-min),1fr))] gap-8">
    <Card
      v-for="(o, i) in options"
      :key="o.id"
      as="div"
      :variant="CardVariant.Raised"
      :selected="o.id === value"
      class="gap-6"
      :class="{ 'opacity-locked': o.notYet }"
    >
      <span
        class="block h-thumb rounded-control border border-line"
        aria-hidden="true"
        :style="{ background: cardPreview(settingKey, o.id, i) }"
      ></span>
      <span class="font-semibold">{{ o.text }}</span>
      <CardDescription v-if="o.desc">{{ o.desc }}</CardDescription>
    </Card>
  </div>
</template>
