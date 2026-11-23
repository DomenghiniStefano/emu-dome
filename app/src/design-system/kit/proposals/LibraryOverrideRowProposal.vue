<script setup lang="ts">
// Kit proposal (library group, «Override row»): the game's value in the ink colour, semibold, with
// the accent kept only on the «changed» dot (accent only on the main action). Desktop layout only.
// Only shown on the Kit page.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import type { OverrideRow } from '@/components/details/gameOverrides'

defineProps<{ row: OverrideRow; consoleName: string }>()

const { t } = useI18n()
</script>

<template>
  <div class="flex items-center gap-10 px-12 py-8">
    <span class="size-6 flex-none rounded-full bg-accent-text" aria-hidden="true" />
    <div class="flex min-w-0 flex-1 flex-col gap-2">
      <span class="font-medium">{{ row.name }}</span>
      <span v-if="row.base !== null" class="text-caption text-ink-muted">
        {{ t('details.settings.base', { page: row.where, console: consoleName, value: row.base }) }}
      </span>
    </div>
    <span class="font-semibold text-ink">{{ row.value }}</span>
    <Button :variant="ButtonVariant.GhostRegular" :size="ButtonSize.Small" class="flex-none">
      {{ t('settingsUi.reset') }}
    </Button>
  </div>
</template>
