<script setup lang="ts">
// E05: no controller connected. The keyboard can be chosen meanwhile.
//
// Off the design system's scales (spec Decision 5, Cowork): the button's 36px height.
import { useI18n } from 'vue-i18n'
import { Gamepad2 } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import SectionTitle from './SectionTitle.vue'

const emit = defineEmits<{ keyboard: [] }>()
const { t } = useI18n()
</script>

<template>
  <section class="flex flex-1 flex-col px-32 py-24">
    <SectionTitle>{{ t('shell.controls') }}</SectionTitle>
    <div
      class="mx-auto flex max-w-[460px] flex-1 flex-col items-center justify-center gap-12 text-center"
    >
      <span
        class="grid size-[56px] place-items-center rounded-16 border border-line bg-surface-raised text-ink-muted"
        aria-hidden="true"
      >
        <Gamepad2 :size="24" :stroke-width="1.75" />
      </span>
      <SectionTitle as="h2">{{ t('controls.e05.title') }}</SectionTitle>
      <p class="m-0 text-ink-muted">{{ t('controls.e05.lead') }}</p>
      <Button class="mt-4 h-[36px]" data-nav-initial @click="emit('keyboard')">
        {{ t('controls.e05.keyboard') }}
      </Button>
    </div>
  </section>
</template>
