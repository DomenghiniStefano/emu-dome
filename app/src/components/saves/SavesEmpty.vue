<script setup lang="ts">
// E03, nothing saved yet: what saves itself and how (`text`: after `saves.autoq`, with the quick
// save shortcut), «Apri la libreria», «Importa salvataggi…» and the formats an import reads.
import { useI18n } from 'vue-i18n'
import { Save } from '@lucide/vue'
import { Button, ButtonSize } from '@/components/ui/button'

defineProps<{ text: string }>()
const emit = defineEmits<{ openLibrary: []; import: [] }>()

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-1 flex-col items-center justify-center gap-12 p-32 text-center">
    <span
      class="grid size-[56px] place-items-center rounded-16 border border-line bg-surface-raised text-ink-muted"
      aria-hidden="true"
    >
      <Save :size="24" :stroke-width="1.75" />
    </span>
    <h1 class="m-0 text-title">{{ t('saves.empty.title') }}</h1>
    <p class="m-0 max-w-[460px] text-ink-muted">{{ text }}</p>
    <div class="mt-4 flex flex-wrap justify-center gap-8">
      <Button :size="ButtonSize.Medium" data-nav-initial @click="emit('openLibrary')">
        {{ t('saves.empty.openLibrary') }}
      </Button>
      <Button :size="ButtonSize.Medium" @click="emit('import')">
        {{ t('saves.empty.import') }}
      </Button>
    </div>
    <p class="m-0 text-caption text-ink-faint">{{ t('saves.empty.formats') }}</p>
  </section>
</template>
