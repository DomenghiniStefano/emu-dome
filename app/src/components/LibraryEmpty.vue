<script setup lang="ts">
// Library with no games (design L02, Stati board): choose a folder (the backend adds and scans
// it), drag games or folders onto the window (App.vue), or open a single file the Phase 0 way.
import { useI18n } from 'vue-i18n'
import { Library } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{ error: string | null }>()
const emit = defineEmits<{ chooseFolder: []; openFile: [] }>()

const { t } = useI18n()
</script>

<template>
  <div class="grid flex-1 place-items-center">
    <!-- The box's 480px width is its own; the title is `title` without its tracking (Cowork). -->
    <section
      class="flex max-w-empty-wide flex-col items-center gap-14 rounded-16 border border-dashed border-line-strong p-48 text-center"
    >
      <Library class="text-accent-text" :size="44" :stroke-width="1.5" aria-hidden="true" />
      <h1 class="m-0 text-title tracking-none">{{ t('library.emptyTitle') }}</h1>
      <p class="m-0 text-ink-muted">{{ t('library.emptyLead') }}</p>
      <div class="flex gap-8">
        <Button
          :variant="ButtonVariant.Primary"
          :size="ButtonSize.Medium"
          data-nav-initial
          @click="emit('chooseFolder')"
        >
          {{ t('library.chooseFolder') }}
        </Button>
        <Button :size="ButtonSize.Medium" @click="emit('openFile')">
          {{ t('library.openFile') }}
        </Button>
      </div>
      <p v-if="error" class="m-0 text-danger" role="alert">{{ error }}</p>
    </section>
  </div>
</template>
