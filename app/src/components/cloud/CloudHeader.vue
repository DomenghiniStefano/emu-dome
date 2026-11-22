<script setup lang="ts">
// The Cloud section's head once signed in (Trello #31, mockup option A): «Cloud», and on the
// right the Google account with «Esci».
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { LogOut } from '@lucide/vue'
import type { CloudAccount } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Heading } from '@/components/ui/heading'

const props = defineProps<{ account: CloudAccount | null }>()
const emit = defineEmits<{ signOut: [] }>()
const { t } = useI18n()

const name = computed(
  () => props.account?.name ?? props.account?.email ?? t('cloud.header.accountUnknown'),
)
</script>

<template>
  <header class="flex flex-wrap items-center justify-between gap-x-12 gap-y-4">
    <Heading as="h1">{{ t('shell.cloud') }}</Heading>
    <div class="flex max-w-full items-center gap-8 text-caption text-ink-muted">
      <span class="min-w-0 wrap-anywhere">{{ t('cloud.header.account', { name }) }}</span>
      <Button
        :variant="ButtonVariant.Link"
        :size="ButtonSize.Text"
        :icon="LogOut"
        @click="emit('signOut')"
      >
        {{ t('cloud.header.signOut') }}
      </Button>
    </div>
  </header>
</template>
