<script setup lang="ts">
// The two answers to the changes «in prova»: undo them, or save them for this game.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

const props = defineProps<{
  /** The phone's sheet: the touch size. */
  touch?: boolean
}>()
defineEmits<{ discard: []; commitGame: [] }>()

const { t } = useI18n()
const size = computed(() => (props.touch ? ButtonSize.MenuPairTouch : ButtonSize.MenuCompact))
</script>

<template>
  <Button :variant="ButtonVariant.Soft" :size="size" class="min-w-0" @click="$emit('discard')">
    {{ t('qm.undo') }}
  </Button>
  <Button :variant="ButtonVariant.Strong" :size="size" class="min-w-0" @click="$emit('commitGame')">
    {{ t('qm.saveGame') }}
  </Button>
</template>
