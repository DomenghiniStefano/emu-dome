<script setup lang="ts">
// The button of a file or action row («Scegli un’immagine…», «Esporta…»). A file row with no file
// chosen says so beside it. A row with no backend yet shows its button disabled, with the reason
// on hover.
import { useI18n } from 'vue-i18n'
import { Button, ButtonVariant } from '@/components/ui/button'

defineProps<{ text: string; noFile: string; notYet: boolean; disabled: boolean }>()
const emit = defineEmits<{ choose: [] }>()
const { t } = useI18n()
</script>

<template>
  <!-- line height 20: off the type scale (Cowork) -->
  <span
    v-if="noFile"
    class="rounded-full bg-surface-hover px-8 text-caption leading-[20px] whitespace-nowrap text-ink-muted"
  >
    {{ noFile }}
  </span>
  <template v-if="text">
    <Button v-if="notYet" :variant="ButtonVariant.RowSoft" disabled :title="t('settingsUi.notYet')">
      {{ text }}
    </Button>
    <Button v-else :variant="ButtonVariant.RowSoft" :disabled="disabled" @click="emit('choose')">
      {{ text }}
    </Button>
  </template>
</template>
