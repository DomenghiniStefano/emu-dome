<script setup lang="ts">
// A setting's name and description, with its marks: the accent dot and «Modificato» when it is
// changed at this level, «Solo su questo computer» for a per-device value. A locked row fades
// this text and greys the name (the control beside it keeps its colour).
import { useI18n } from 'vue-i18n'
import type { SettingDevice } from '../../emu'

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
    class="flex min-w-0 flex-1 basis-setting-text flex-col gap-2"
    :class="{ 'opacity-locked': locked }"
  >
    <div class="flex flex-wrap items-center gap-8">
      <span v-if="changedHere" class="size-6 rounded-full bg-accent-text" aria-hidden="true"></span>
      <span class="font-medium" :class="{ 'text-ink-muted': locked }">{{ name }}</span>
      <span v-if="changedHere" class="text-caption text-accent-text">
        {{ t('settingsUi.changed') }}
        <span class="sr-only">{{ t('settingsUi.changedSr') }}</span>
      </span>
      <!-- 11 / 16: off the type scale (spec Decision 5, Cowork) -->
      <span
        v-if="onlyOn"
        class="rounded-4 bg-surface-raised px-6 text-[11px] leading-[16px] text-ink-muted"
      >
        {{ t(`settingsUi.onlyHere.${onlyOn}`) }}
      </span>
    </div>
    <div class="text-caption text-ink-muted">{{ desc }}</div>
  </div>
</template>
