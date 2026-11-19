<script setup lang="ts">
// E01: a library folder is not reachable; its games stay listed, faded. «Scegli un'altra
// cartella» replaces it.
import { useI18n } from 'vue-i18n'
import { FolderX } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import type { UnreachableFolder } from './libraryView'

defineProps<{ folder: UnreachableFolder }>()
const emit = defineEmits<{ replace: [] }>()

const { t } = useI18n()
</script>

<template>
  <div
    class="flex items-center gap-12 rounded-10 border border-warning-edge bg-warning-tint px-14 py-10"
    role="status"
  >
    <FolderX class="flex-none text-warning" :size="18" :stroke-width="1.75" aria-hidden="true" />
    <span class="min-w-0 flex-1 wrap-anywhere">{{
      t('library.unreachable.text', { path: folder.path, count: folder.count }, folder.count)
    }}</span>
    <Button @click="emit('replace')">{{ t('library.unreachable.choose') }}</Button>
  </div>
</template>
