<script setup lang="ts">
// C01 «Scorciatoie da ricordare»: what to press for each shortcut, the hint beside the heading,
// and «Modifica le scorciatoie» down to C03.
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import type { RememberLine } from './shortcutLines'

defineProps<{ lines: RememberLine[]; hint: string }>()
const emit = defineEmits<{ edit: [] }>()
const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col gap-8">
    <div class="flex flex-wrap items-baseline justify-between gap-8">
      <span class="text-overline text-ink-muted">{{ t('controls.overview.shortcuts') }}</span>
      <span class="text-caption-tall text-ink-muted">{{ hint }}</span>
    </div>
    <Card :variant="CardVariant.List">
      <div
        v-for="l in lines"
        :key="l.key"
        class="flex min-h-control-lg items-center gap-12 border-line px-16 py-4 not-first:border-t"
      >
        <span class="min-w-key-chip flex-none">
          <span
            class="inline-flex h-20 items-center rounded-4 border border-line-strong px-6 text-caption-tall font-semibold whitespace-nowrap"
            >{{ l.chip }}</span
          >
        </span>
        <span>{{ l.name }}</span>
      </div>
    </Card>
    <Button class="self-start" @click="emit('edit')">
      {{ t('controls.overview.editShortcuts') }}
    </Button>
  </section>
</template>
