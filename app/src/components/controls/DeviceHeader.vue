<script setup lang="ts">
// The head of C02–C04: «‹ Controlli» back to C01, above the device's name and state.
import { useI18n } from 'vue-i18n'
import { ChevronLeft } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import SectionTitle from './SectionTitle.vue'

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
      data-nav-initial
      @click="emit('back')"
    >
      <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ t('shell.controls') }}
    </Button>
    <SectionTitle as="h2">{{ name }}</SectionTitle>
    <div class="text-ink-muted">{{ detail }}</div>
  </div>
</template>
