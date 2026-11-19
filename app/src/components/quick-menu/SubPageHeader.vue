<script setup lang="ts">
// The head of a quick menu page: back one level (B / Esc), then the page's title. On the phone's
// sheet the chevron alone, beside the title.
import BackButton from './BackButton.vue'
import PageTitle from './PageTitle.vue'

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
  <div class="flex flex-col gap-8 touch:flex-row touch:items-center">
    <BackButton
      :label="backLabel"
      :hint="touch ? null : hint"
      :icon-only="touch"
      @click="$emit('back')"
    />
    <PageTitle>{{ title }}</PageTitle>
  </div>
</template>
