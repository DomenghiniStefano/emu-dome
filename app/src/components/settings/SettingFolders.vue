<script setup lang="ts">
// The game folders (`folders`, `library.folders`): changed through the library, which rescans
// and updates the setting.
import { useI18n } from 'vue-i18n'
import { Folder } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { emu } from '../../emu'
import { errorText } from '../../i18n/labels'

defineProps<{ folders: string[]; addText: string; disabled: boolean }>()
const emit = defineEmits<{ error: [string] }>()
const { t } = useI18n()

async function remove(path: string) {
  try {
    await emu.library.removeFolder(path)
  } catch (e) {
    emit('error', errorText(e))
  }
}
async function add() {
  try {
    await emu.library.chooseFolder()
  } catch (e) {
    emit('error', errorText(e))
  }
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- 36 high: the control height awaiting Cowork (spec Decision 5) -->
    <div
      v-for="path in folders"
      :key="path"
      class="flex h-[36px] items-center gap-10 rounded-6 bg-surface-raised px-10"
    >
      <Folder :size="16" :stroke-width="1.75" aria-hidden="true" />
      <span class="min-w-0 flex-1 truncate">{{ path }}</span>
      <Button
        :variant="ButtonVariant.RowGhost"
        :size="ButtonSize.Small"
        :disabled="disabled"
        @click="remove(path)"
      >
        {{ t('settings.library.folders.value.remove') }}
      </Button>
    </div>
    <Button :variant="ButtonVariant.RowSoft" class="self-start" :disabled="disabled" @click="add">
      {{ addText }}
    </Button>
  </div>
</template>
