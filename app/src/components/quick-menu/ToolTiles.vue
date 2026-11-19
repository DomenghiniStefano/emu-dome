<script setup lang="ts">
// The tools on desktop (Strumenti): one tile each; those with no backend yet are disabled and
// named in the note under them.
import { useI18n } from 'vue-i18n'
import { Card, CardVariant } from '@/components/ui/card'
import SectionLabel from './SectionLabel.vue'
import type { Tool } from './tools'

defineProps<{
  tools: Tool[]
  /** Six tools (NDS, 3DS): three columns. */
  threeColumns: boolean
  /** The note about the tools not available yet; '' with none. */
  notYet: string
}>()
defineEmits<{ run: [Tool, Event] }>()

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col gap-10" :aria-label="t('qm.toolsTitle')">
    <SectionLabel>{{ t('qm.toolsTitle') }}</SectionLabel>
    <div class="grid gap-8" :class="threeColumns ? 'grid-cols-3' : 'grid-cols-4'">
      <Card
        v-for="x in tools"
        :key="x.id"
        as="button"
        :variant="CardVariant.Tool"
        :selected="x.pressed"
        class="min-w-0 gap-6"
        :disabled="!x.available"
        :aria-pressed="x.pressed"
        @click="$emit('run', x, $event)"
      >
        <component :is="x.icon" :size="20" :stroke-width="1.75" aria-hidden="true" />
        {{ t(x.label) }}
      </Card>
    </div>
    <span v-if="notYet" class="text-caption text-ink-muted">{{ notYet }}</span>
  </section>
</template>
