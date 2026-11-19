<script setup lang="ts">
// Library with no games (design L02, Stati board): choose a folder (the backend adds and scans
// it), drag games or folders onto the window (App.vue), or open a single file the Phase 0 way.
import { useI18n } from 'vue-i18n'
import { Library } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'

defineProps<{ error: string | null }>()
const emit = defineEmits<{ chooseFolder: []; openFile: [] }>()

const { t } = useI18n()
</script>

<template>
  <div class="grid flex-1 place-items-center">
    <EmptyState
      :variant="EmptyStateVariant.Framed"
      :icon="Library"
      :title="t('library.emptyTitle')"
      title-as="h1"
      :text="t('library.emptyLead')"
    >
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
      <template #footer>
        <p v-if="error" class="m-0 text-danger" role="alert">{{ error }}</p>
      </template>
    </EmptyState>
  </div>
</template>
