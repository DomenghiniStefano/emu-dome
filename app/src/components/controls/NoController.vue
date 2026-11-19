<script setup lang="ts">
// E05: no controller connected. The keyboard can be chosen meanwhile.
import { useI18n } from 'vue-i18n'
import { Gamepad2 } from '@lucide/vue'
import { Button, ButtonSize } from '@/components/ui/button'
import SectionTitle from './SectionTitle.vue'

const emit = defineEmits<{ keyboard: [] }>()
const { t } = useI18n()
</script>

<template>
  <section class="flex flex-1 flex-col px-32 py-24">
    <SectionTitle>{{ t('shell.controls') }}</SectionTitle>
    <div
      class="mx-auto flex max-w-empty flex-1 flex-col items-center justify-center gap-12 text-center"
    >
      <span
        class="grid size-thumb place-items-center rounded-16 border border-line bg-surface-raised text-ink-muted"
        aria-hidden="true"
      >
        <Gamepad2 :size="24" :stroke-width="1.75" />
      </span>
      <SectionTitle as="h2">{{ t('controls.e05.title') }}</SectionTitle>
      <p class="m-0 text-ink-muted">{{ t('controls.e05.lead') }}</p>
      <Button :size="ButtonSize.Medium" class="mt-4" data-nav-initial @click="emit('keyboard')">
        {{ t('controls.e05.keyboard') }}
      </Button>
    </div>
  </section>
</template>
