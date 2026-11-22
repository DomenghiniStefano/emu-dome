<script setup lang="ts">
// The Cloud section's head once signed in (Trello #31, mockup option A): «Cloud», and on the
// right the Google account with «Esci».
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
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
  <header class="flex flex-wrap items-center justify-between gap-x-16 gap-y-8">
    <Heading as="h1">{{ t('shell.cloud') }}</Heading>
    <div class="flex min-w-0 items-center gap-12 text-caption text-ink-muted">
      <span class="min-w-0 wrap-anywhere">{{ t('cloud.header.account', { name }) }}</span>
      <Button :variant="ButtonVariant.Link" :size="ButtonSize.Text" @click="emit('signOut')">
        {{ t('cloud.header.signOut') }}
      </Button>
    </div>
  </header>
</template>
