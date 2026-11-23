<script setup lang="ts">
// The head of C02–C04: «‹ Controlli» back to C01, above the device's name and state.
import { useI18n } from 'vue-i18n'
import { ChevronLeft } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Heading, HeadingVariant } from '@/design-system/components/heading'

defineProps<{ name: string; detail: string }>()
const emit = defineEmits<{ back: [] }>()
const { t } = useI18n()
</script>

<template>
  <div class="flex flex-col">
    <Button
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.TextLine"
      class="mb-4 -ml-6 self-start"
      :icon="ChevronLeft"
      data-nav-initial
      @click="emit('back')"
    >
      {{ t('shell.controls') }}
    </Button>
    <Heading as="h2" :variant="HeadingVariant.Title">{{ name }}</Heading>
    <div class="text-ink-muted">{{ detail }}</div>
  </div>
</template>
