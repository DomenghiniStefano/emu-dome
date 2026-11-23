<script setup lang="ts">
// The console a layout editor opens for (Trello #111), asked when its row is at «Tutti i giochi»
// (or at a console the editor has nothing for): C09 offers the two-screen consoles, C06 every
// console with touch controls (layoutEditors.ts). The first is focused; B, Esc and «Annulla»
// leave. Controller / keyboard: FlowDialog's own nav scope.
import { useI18n } from 'vue-i18n'
import { Gamepad2 } from '@lucide/vue'
import type { ConsoleId } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import FlowDialog from '@/components/game/FlowDialog.vue'
import { EDITOR_CONSOLES, type LayoutEditor } from './layoutEditors'

defineProps<{ editor: LayoutEditor }>()
const emit = defineEmits<{ choose: [ConsoleId]; cancel: [] }>()

const { t } = useI18n()
</script>

<template>
  <FlowDialog
    :title="t(`settingsUi.layoutEditor.${editor}.title`)"
    escape-closes
    @back="emit('cancel')"
  >
    <p class="m-0">{{ t(`settingsUi.layoutEditor.${editor}.text`) }}</p>
    <div class="flex flex-col gap-2">
      <Button
        v-for="(c, i) in EDITOR_CONSOLES[editor]"
        :key="c"
        :variant="ButtonVariant.Menu"
        :size="ButtonSize.MenuRow"
        :icon="Gamepad2"
        :data-nav-initial="i === 0 ? '' : undefined"
        @click="emit('choose', c)"
      >
        {{ t(`console.${c}`) }}
      </Button>
    </div>
    <template #actions>
      <Button :size="ButtonSize.Medium" @click="emit('cancel')">
        {{ t('settingsUi.layoutEditor.cancel') }}
      </Button>
    </template>
  </FlowDialog>
</template>
