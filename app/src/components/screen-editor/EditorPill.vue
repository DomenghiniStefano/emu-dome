<script setup lang="ts">
// The C09 editor's panel folded («Riduci», Select): a glass pill at the top centre with the
// selected screen, its width, «Mostra le opzioni» and «Salva», so the whole window is free to
// drag and resize.
import { useI18n } from 'vue-i18n'
import { Save, SlidersHorizontal } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Kbd, KbdSize } from '@/design-system/components/kbd'
import { posName } from '@/lib/controls'
import { inputMode } from '@/lib/inputMode'

defineProps<{
  screenName: string
  /** The selected screen's width, per cent, as the panel's field shows it. */
  width: string
  busy: boolean
  canSave: boolean
}>()
const emit = defineEmits<{ expand: []; save: [] }>()
const { t } = useI18n()
</script>

<template>
  <div
    class="flex items-center gap-8 rounded-panel border border-line py-4 pr-4 pl-12 shadow-glass glass-thick-flat"
  >
    <span class="font-semibold">{{ screenName }}</span>
    <span class="text-caption text-ink-muted tabular-nums">
      {{ t('screenEditor.width', { n: width }) }}
    </span>
    <Button
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.Small"
      :icon="SlidersHorizontal"
      @click="emit('expand')"
    >
      {{ t('screenEditor.expand') }}
      <Kbd v-if="inputMode === 'controller'" :size="KbdSize.Small">
        {{ posName('generic', 'back', false) }}
      </Kbd>
    </Button>
    <Button
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.Compact"
      :icon="Save"
      :disabled="busy || !canSave"
      @click="emit('save')"
    >
      {{ t('screenEditor.save') }}
    </Button>
  </div>
</template>
