<script setup lang="ts">
// E01: a library folder is not reachable; its games stay listed, faded. «Scegli un'altra
// cartella» replaces it. In a narrow window the button goes under the text, on the right.
import { useI18n } from 'vue-i18n'
import { FolderOpen, FolderX } from '@lucide/vue'
import { Button } from '@/design-system/components/button'
import LibraryBanner from './LibraryBanner.vue'
import type { UnreachableFolder } from './libraryView'

defineProps<{ folder: UnreachableFolder }>()
const emit = defineEmits<{ replace: [] }>()

const { t } = useI18n()
</script>

<template>
  <LibraryBanner warning>
    <span class="flex min-w-0 grow basis-menu-min items-start gap-8">
      <FolderX class="flex-none text-warning" :size="18" :stroke-width="1.75" aria-hidden="true" />
      <span class="min-w-0 wrap-anywhere">{{
        t(
          'library.unreachable.text',
          { path: folder.name ?? t('library.folderUnnamed'), count: folder.count },
          folder.count,
        )
      }}</span>
    </span>
    <Button class="ml-auto" :icon="FolderOpen" @click="emit('replace')">
      {{ t('library.unreachable.choose') }}
    </Button>
  </LibraryBanner>
</template>
