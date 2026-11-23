<script setup lang="ts">
// The head of a quick menu page: back one level (B / Esc), then the page's title. On the phone
// (owner, 2026-11-22) «‹ Indietro» with the title at the other end of the row.
import BackButton from './BackButton.vue'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

defineProps<{
  title: string
  /** Where back leads. */
  backLabel: string
  hint: string | null
  touch?: boolean
}>()
defineEmits<{ back: [] }>()
</script>

<template>
  <div class="flex flex-col gap-8 touch:flex-row touch:items-center touch:justify-between">
    <BackButton :label="backLabel" :hint="touch ? null : hint" @click="$emit('back')" />
    <Heading as="h2" :variant="HeadingVariant.PageTitle">{{ title }}</Heading>
  </div>
</template>
