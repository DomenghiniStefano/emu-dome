<script setup lang="ts">
// E03, nothing saved yet: what saves itself and how (`text`: after `saves.autoq`, with the quick
// save shortcut), «Apri la libreria», «Importa salvataggi…» and the formats an import reads.
import { useI18n } from 'vue-i18n'
import { Import, Library, Save } from '@lucide/vue'
import { Button, ButtonSize } from '@/components/ui/button'
import { EmptyState } from '@/components/ui/empty-state'

defineProps<{ text: string }>()
const emit = defineEmits<{ openLibrary: []; import: [] }>()

const { t } = useI18n()
</script>

<template>
  <EmptyState :icon="Save" :title="t('saves.empty.title')" title-as="h1" :text="text">
    <Button :size="ButtonSize.Medium" :icon="Library" data-nav-initial @click="emit('openLibrary')">
      {{ t('saves.empty.openLibrary') }}
    </Button>
    <Button :size="ButtonSize.Medium" :icon="Import" @click="emit('import')">
      {{ t('saves.empty.import') }}
    </Button>
    <template #footer>
      <p class="m-0 text-caption text-ink-faint">{{ t('saves.empty.formats') }}</p>
    </template>
  </EmptyState>
</template>
