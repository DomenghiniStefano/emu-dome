<script setup lang="ts">
// A setting's name and description, with its marks: the accent dot and «Modificato» when it is
// changed at this level, «Solo su questo computer» for a per-device value; the slot is a line under
// the description (the row's «Ripristina» / «Come in …»). A locked row fades
// this text and greys the name (the control beside it keeps its colour).
import { useI18n } from 'vue-i18n'
import type { SettingDevice } from '@/emu'
import { Badge } from '@/design-system/components/badge'

defineProps<{
  name: string
  desc: string
  changedHere: boolean
  /** The device a per-device value belongs to, when it is one. */
  onlyOn: SettingDevice | null
  locked: boolean
}>()

const { t } = useI18n()
</script>

<template>
  <div
    class="flex min-w-0 flex-1 basis-setting-text flex-col"
    :class="{ 'opacity-locked': locked }"
  >
    <div class="flex flex-wrap items-center gap-8">
      <span v-if="changedHere" class="size-6 rounded-full bg-accent-text" aria-hidden="true"></span>
      <span class="font-medium" :class="{ 'text-ink-muted': locked }">{{ name }}</span>
      <span v-if="changedHere" class="text-caption text-accent-text">
        {{ t('settingsUi.changed') }}
        <span class="sr-only">{{ t('settingsUi.changedSr') }}</span>
      </span>
      <Badge v-if="onlyOn">{{ t(`settingsUi.onlyHere.${onlyOn}`) }}</Badge>
    </div>
    <div class="text-caption text-ink-muted">{{ desc }}</div>
    <div v-if="$slots.default" class="flex flex-wrap items-center gap-8 text-caption">
      <slot />
    </div>
  </div>
</template>
