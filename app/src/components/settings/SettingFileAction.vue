<script setup lang="ts">
// The button of a file or action row («Scegli un’immagine…», «Esporta…»). Beside it, a file row
// says which file is chosen, or that none is. A row with no backend yet shows its button disabled, with the reason
// on hover. Its icon is the row's kind's (rowActionIcon.ts, #104).
import type { Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonFade, ButtonVariant } from '@/design-system/components/button'
import { Badge, BadgeSize, BadgeVariant } from '@/design-system/components/badge'

defineProps<{
  text: string
  icon?: Component
  /** «crt.slangp», «Nessun file scelto», or '' to show nothing. */
  fileState: string
  notYet: boolean
  disabled: boolean
}>()
const emit = defineEmits<{ choose: [] }>()
const { t } = useI18n()
</script>

<template>
  <Badge v-if="fileState" :variant="BadgeVariant.Quiet" :size="BadgeSize.Pill">
    {{ fileState }}
  </Badge>
  <template v-if="text">
    <Button
      v-if="notYet"
      :variant="ButtonVariant.Soft"
      :fade="ButtonFade.None"
      :icon="icon"
      disabled
      :title="t('settingsUi.notYet')"
    >
      {{ text }}
    </Button>
    <Button
      v-else
      :variant="ButtonVariant.Soft"
      :fade="ButtonFade.None"
      :icon="icon"
      :disabled="disabled"
      @click="emit('choose')"
    >
      {{ text }}
    </Button>
  </template>
</template>
