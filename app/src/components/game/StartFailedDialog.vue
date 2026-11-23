<script setup lang="ts">
// G02: the game could not start, and why. When the failure is about a file or a folder (Trello
// #128) its name leads, then the reason, then where it is on a muted line: at most two lines,
// broken anywhere (an Android URI or a deep path has no spaces), the whole of it in the tooltip.
import { useI18n } from 'vue-i18n'
import { Check } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { DialogTone } from '@/design-system/components/dialog'
import FlowDialog from './FlowDialog.vue'

defineProps<{ title: string; reason: string; file: string | null; path: string | null }>()
const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
</script>

<template>
  <FlowDialog
    :title="t('flow.startFailed', { title })"
    :tone="DialogTone.Danger"
    @back="emit('close')"
  >
    <p v-if="file" class="m-0 font-semibold wrap-anywhere text-ink">{{ file }}</p>
    <p class="m-0">{{ reason }}</p>
    <p v-if="path" class="m-0 line-clamp-2 text-caption break-all text-ink-muted" :title="path">
      {{ path }}
    </p>
    <template #actions>
      <Button
        :variant="ButtonVariant.Primary"
        :size="ButtonSize.Medium"
        :icon="Check"
        data-nav-initial
        @click="emit('close')"
      >
        {{ t('flow.gotIt') }}
      </Button>
    </template>
  </FlowDialog>
</template>
