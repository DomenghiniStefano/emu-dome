<script setup lang="ts">
// The C06 editor's toolbar folded («Riduci», Select): a glass pill at the top centre with the
// picked control, «Mostra gli strumenti» and «Salva», so the whole screen is free to drag and
// pinch (as the C09 editor's pill).
import { useI18n } from 'vue-i18n'
import { Save, SlidersHorizontal } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{
  /** The picked control's name (or «Tocca un comando»). */
  name: string
  busy: boolean
  canSave: boolean
}>()
const emit = defineEmits<{ expand: []; save: [] }>()
const { t } = useI18n()
</script>

<template>
  <div
    class="flex max-w-full items-center gap-8 rounded-full border border-line py-6 pr-6 pl-16 shadow-glass glass-thick-flat"
  >
    <span class="min-w-0 truncate font-semibold">{{ name }}</span>
    <Button
      :variant="ButtonVariant.Soft"
      :size="ButtonSize.Tool"
      :icon="SlidersHorizontal"
      @click="emit('expand')"
    >
      {{ t('touch.editor.expand') }}
    </Button>
    <Button
      :variant="ButtonVariant.Primary"
      :size="ButtonSize.Tool"
      :icon="Save"
      :disabled="busy || !canSave"
      @click="emit('save')"
    >
      {{ t('touch.editor.save') }}
    </Button>
  </div>
</template>
