<script setup lang="ts">
// The game folders (`folders`, `library.folders`): changed through the library, which rescans
// and updates the setting.
import { useI18n } from 'vue-i18n'
import { Folder, FolderPlus, X } from '@lucide/vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { emu } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { folderLabel, library } from '@/lib/library'

defineProps<{ folders: string[]; addText: string; disabled: boolean }>()
const { t } = useI18n()
const failWith = useErrorSink()

async function remove(path: string) {
  try {
    await emu.library.removeFolder(path)
  } catch (e) {
    failWith(e)
  }
}
async function add() {
  try {
    await emu.library.chooseFolder()
  } catch (e) {
    failWith(e)
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <div
      v-for="path in folders"
      :key="path"
      class="flex min-h-control items-center gap-8 rounded-control bg-surface-raised pr-2 pl-8"
    >
      <Folder :size="16" :stroke-width="1.75" aria-hidden="true" />
      <span class="min-w-0 flex-1 truncate">{{
        folderLabel(library.folders, path, t('library.folderUnnamed'))
      }}</span>
      <!-- Repeated on every folder: the icon alone (#104), the touch minimum on the phone. -->
      <Button
        :variant="ButtonVariant.GhostRegular"
        :fade="ButtonFade.None"
        :size="ButtonSize.IconRow"
        :icon="X"
        :label="t('settings.library.folders.value.remove')"
        :disabled="disabled"
        @click="remove(path)"
      />
    </div>
    <Button
      :variant="ButtonVariant.Soft"
      :fade="ButtonFade.None"
      class="self-start"
      :icon="FolderPlus"
      :disabled="disabled"
      @click="add"
    >
      {{ addText }}
    </Button>
  </div>
</template>
