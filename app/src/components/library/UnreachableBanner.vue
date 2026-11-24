<script setup lang="ts">
// E01: a library folder is not reachable; its games stay listed, faded. «Scegli un'altra
// cartella» replaces it. A picked Android folder whose read permission Android took back (#138:
// the app uninstalled with its data kept) says so instead, and «Scegli di nuovo la cartella»
// picks it again (the same folder keeps its place and its games). In a narrow window the button
// goes under the text, on the right.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderOpen, FolderX } from '@lucide/vue'
import { Button } from '@/design-system/components/button'
import LibraryBanner from './LibraryBanner.vue'
import type { UnreachableFolder } from './libraryView'

const props = defineProps<{ folder: UnreachableFolder }>()
const emit = defineEmits<{ replace: [] }>()

const { t } = useI18n()

const permissionLost = computed(() => props.folder.access === 'permissionLost')
const name = computed(() => props.folder.name ?? t('library.folderUnnamed'))
const text = computed(() =>
  permissionLost.value
    ? t('library.permissionLost.text', { path: name.value })
    : t(
        'library.unreachable.text',
        { path: name.value, count: props.folder.count },
        props.folder.count,
      ),
)
</script>

<template>
  <LibraryBanner warning>
    <span class="flex min-w-0 grow basis-menu-min items-start gap-8">
      <FolderX class="flex-none text-warning" :size="18" :stroke-width="1.75" aria-hidden="true" />
      <span class="min-w-0 wrap-anywhere">{{ text }}</span>
    </span>
    <Button class="ml-auto" :icon="FolderOpen" @click="emit('replace')">
      {{ t(permissionLost ? 'library.permissionLost.choose' : 'library.unreachable.choose') }}
    </Button>
  </LibraryBanner>
</template>
