<script setup lang="ts">
// The C09 editor's panel folded («Riduci», Select): a glass pill at the top centre with the
// selected screen, its width, «Mostra le opzioni» and «Salva», so the whole window is free to
// drag and resize.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdSize } from '@/components/ui/kbd'
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
    class="flex items-center gap-10 rounded-full border border-line py-6 pr-6 pl-16 shadow-glass glass-thick-flat"
  >
    <span class="font-semibold">{{ screenName }}</span>
    <span class="text-caption text-ink-muted tabular-nums">
      {{ t('screenEditor.width', { n: width }) }}
    </span>
    <Button :variant="ButtonVariant.Soft" :size="ButtonSize.Small" @click="emit('expand')">
      {{ t('screenEditor.expand') }}
      <Kbd v-if="inputMode === 'controller'" :size="KbdSize.Small">
        {{ posName('generic', 'back', false) }}
      </Kbd>
    </Button>
    <Button
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.Compact"
      :disabled="busy || !canSave"
      @click="emit('save')"
    >
      {{ t('screenEditor.save') }}
    </Button>
  </div>
</template>
