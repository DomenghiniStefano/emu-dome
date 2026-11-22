<script setup lang="ts">
// C01 «Scorciatoie da ricordare»: what to press for each shortcut, the hint beside the heading,
// and «Modifica le scorciatoie» down to C03.
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import type { RememberLine } from './shortcutLines'
import { Kbd } from '@/components/ui/kbd'
import { LabelledBlock } from '@/components/ui/labelled-block'

defineProps<{ lines: RememberLine[]; hint: string }>()
const emit = defineEmits<{ edit: [] }>()
const { t } = useI18n()
</script>

<template>
  <LabelledBlock :title="t('controls.overview.shortcuts')">
    <template #aside>
      <span class="text-caption-tall text-ink-muted">{{ hint }}</span>
    </template>
    <Card :variant="CardVariant.List">
      <div
        v-for="l in lines"
        :key="l.key"
        class="flex min-h-control items-center gap-8 border-line px-12 py-2 not-first:border-t"
      >
        <span class="min-w-key-chip flex-none">
          <Kbd>{{ l.chip }}</Kbd>
        </span>
        <span>{{ l.name }}</span>
      </div>
    </Card>
    <Button class="self-start" @click="emit('edit')">
      {{ t('controls.overview.editShortcuts') }}
    </Button>
  </LabelledBlock>
</template>
