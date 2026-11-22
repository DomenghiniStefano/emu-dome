<script setup lang="ts">
// One setting a game has a value of its own for (L06): its name, its page and what every game of
// the console gets instead, the game's value and «Ripristina». Phone in portrait: the game's
// value under the name, the console's under it.
import { useI18n } from 'vue-i18n'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import type { OverrideRow } from './gameOverrides'

defineProps<{ row: OverrideRow; consoleName: string; phone: boolean }>()
const emit = defineEmits<{ reset: [] }>()

const { t } = useI18n()
</script>

<template>
  <div class="flex items-center gap-10 px-12 py-8 touch:px-14">
    <span v-if="!phone" class="size-6 flex-none rounded-full bg-accent-text" aria-hidden="true" />
    <div class="flex min-w-0 flex-1 flex-col gap-2">
      <span class="font-medium">{{ row.name }}</span>
      <span v-if="phone" class="text-body text-accent-text">{{ row.value }}</span>
      <span v-if="row.base !== null" class="text-caption text-ink-muted">
        {{
          phone
            ? t('details.settings.baseShort', { console: consoleName, value: row.base })
            : t('details.settings.base', { page: row.where, console: consoleName, value: row.base })
        }}
      </span>
    </div>
    <span v-if="!phone" class="font-semibold text-accent-text">{{ row.value }}</span>
    <!-- The row's own type, regular weight (larger on the phone), with the touch height there. -->
    <Button
      :variant="ButtonVariant.GhostRegular"
      :size="ButtonSize.Small"
      class="flex-none touch:min-h-touch"
      @click="emit('reset')"
    >
      {{ t('settingsUi.reset') }}
    </Button>
  </div>
</template>
