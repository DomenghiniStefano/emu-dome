<script setup lang="ts">
// The two answers to the changes «in prova»: undo them, or save them for this game.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineEmits<{ discard: []; commitGame: [] }>()

const { t } = useI18n()
</script>

<template>
  <Button
    :variant="ButtonVariant.Soft"
    :size="ButtonSize.MenuPair"
    class="min-w-0"
    @click="$emit('discard')"
  >
    {{ t('qm.undo') }}
  </Button>
  <Button
    :variant="ButtonVariant.Strong"
    :size="ButtonSize.MenuPair"
    class="min-w-0"
    @click="$emit('commitGame')"
  >
    {{ t('qm.saveGame') }}
  </Button>
</template>
