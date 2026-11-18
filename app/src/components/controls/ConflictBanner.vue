<script setup lang="ts">
// «… è già usato per … Scambiali?» under the row that asked (C02, C03): swap the two, or cancel.
// It takes the focus as it appears, on «Scambia», so A answers it at once.
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'

defineProps<{ text: string }>()
const emit = defineEmits<{ swap: []; cancel: [] }>()
const { t } = useI18n()

const swapButton = ref<{ $el: HTMLElement } | null>(null)
onMounted(() => swapButton.value?.$el.focus())
</script>

<template>
  <div
    class="flex items-center gap-8 border-t border-warning-edge bg-warning-tint px-16 py-8"
    role="alert"
  >
    <span class="flex-1">{{ text }}</span>
    <Button ref="swapButton" @click="emit('swap')">{{ t('controls.swap') }}</Button>
    <Button @click="emit('cancel')">{{ t('controls.cancel') }}</Button>
  </div>
</template>
