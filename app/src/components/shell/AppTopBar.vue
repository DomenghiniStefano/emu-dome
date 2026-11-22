<script setup lang="ts">
// The shell's top bar (design L01): thin glass with the small mark and the wordmark (#101), and
// the search everywhere in the middle (X01, #32): a field with its Ctrl K on the desktop, a
// magnifier on the phone. Either opens the search (`search`). On the right, the notifications'
// bell (X03).
import { useI18n } from 'vue-i18n'
import { Search } from '@lucide/vue'
import markSmall from '@/assets/brand/emudome-mark-small.svg'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd } from '@/components/ui/kbd'
import NotificationBell from '@/components/notifications/NotificationBell.vue'
import { useDeviceLayout } from '@/composables/useDeviceLayout'

const emit = defineEmits<{ search: [] }>()
const { t } = useI18n()
const layout = useDeviceLayout()
</script>

<template>
  <header
    class="flex shrink-0 basis-titlebar items-center gap-8 border-b border-line bg-glass-thin px-12 backdrop-glass"
  >
    <!-- Decoration: the wordmark beside it says the name. -->
    <img :src="markSmall" alt="" aria-hidden="true" class="block h-mark-small w-auto" />
    <!-- The wordmark's 14px and -0.025em are off the type scale (Cowork, spec Decision 5). -->
    <div class="text-wordmark">EmuDome</div>
    <div class="flex-1"></div>
    <Button
      v-if="layout.phone.value"
      :variant="ButtonVariant.BarIcon"
      :size="ButtonSize.Icon"
      :aria-label="t('search.field')"
      @click="emit('search')"
    >
      <Search :size="20" :stroke-width="1.75" aria-hidden="true" />
    </Button>
    <Button
      v-else
      :variant="ButtonVariant.Field"
      :size="ButtonSize.Compact"
      class="w-topbar-search max-w-full min-w-0"
      @click="emit('search')"
    >
      <Search class="flex-none" :size="16" :stroke-width="1.75" aria-hidden="true" />
      <span class="min-w-0 flex-1 truncate text-left">{{ t('search.field') }}</span>
      <Kbd>{{ t('search.key.ctrl') }}</Kbd>
      <Kbd>{{ t('search.key.k') }}</Kbd>
    </Button>
    <div class="flex-1"></div>
    <NotificationBell />
  </header>
</template>
