<script setup lang="ts">
// One quick-save slot (board «Salvataggi» V02; #59 item 3, the one slot component): a 4:3
// thumbnail, the name, the date, «Gioca da qui» and «…», each its icon alone (#104). Saves (V02) uses it, and so will Game
// details › Salvataggi (L05). An empty slot shows no buttons. `outdated`: the state was made by
// an older version of the emulator, «Potrebbe non caricarsi» under the date.
//
// The parent decides what «…» offers (V03): the Automatico slot has no Rinomina, Blocca, Elimina.
import { useI18n } from 'vue-i18n'
import { Ellipsis, Play, TriangleAlert } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardPad } from '@/design-system/components/card'
import { cn } from '@/design-system/lib/cn'
import SaveThumb from './SaveThumb.vue'

withDefaults(
  defineProps<{
    name: string
    /** The date, already as text; null when unknown. */
    when?: string | null
    thumbUrl?: string | null
    empty?: boolean
    outdated?: boolean
  }>(),
  { when: null, thumbUrl: null, empty: false, outdated: false },
)
const emit = defineEmits<{ play: []; more: [Event] }>()

const { t } = useI18n()
</script>

<template>
  <Card :pad="CardPad.Tight" class="min-w-0 gap-4">
    <SaveThumb
      :src="empty ? null : thumbUrl"
      :class="cn('aspect-4/3 rounded-control', empty && 'bg-surface-raised')"
    />
    <span class="font-semibold">{{ name }}</span>
    <span v-if="when" class="text-caption text-ink-muted tabular-nums">{{ when }}</span>
    <span
      v-if="outdated && !empty"
      class="inline-flex items-center gap-4 text-caption text-warning"
    >
      <TriangleAlert :size="12" :stroke-width="1.75" aria-hidden="true" />
      {{ t('saves.quick.mightNotLoad') }}
    </span>
    <!-- at the bottom: a row of tiles keeps its buttons on one line when one has a line more -->
    <span v-if="!empty" class="mt-auto flex gap-6 pt-2">
      <Button
        :variant="ButtonVariant.SoftAccent"
        :size="ButtonSize.IconRow"
        :icon="Play"
        icon-filled
        :label="t('saves.quick.load')"
        @click="emit('play')"
      />
      <Button
        :size="ButtonSize.IconRow"
        :icon="Ellipsis"
        :label="t('saves.slotMore')"
        @click="emit('more', $event)"
      />
    </span>
  </Card>
</template>
