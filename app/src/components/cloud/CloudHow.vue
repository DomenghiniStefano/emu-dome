<script setup lang="ts">
// The tab «Come funziona» (Trello #31): where the files go, when they move, what happens when a
// save changed on two devices, what never syncs, and the game files.
import type { Component } from 'vue'
import { useI18n } from 'vue-i18n'
import { FolderLock, Gamepad2, MonitorSmartphone, RefreshCw, Split } from '@lucide/vue'
import { Card } from '@/components/ui/card'

const { t } = useI18n()

const POINTS: { key: string; icon: Component }[] = [
  { key: 'folder', icon: FolderLock },
  { key: 'when', icon: RefreshCw },
  { key: 'conflict', icon: Split },
  { key: 'device', icon: MonitorSmartphone },
  { key: 'games', icon: Gamepad2 },
]
</script>

<template>
  <Card>
    <ul class="m-0 flex list-none flex-col gap-16 p-0">
      <li v-for="p in POINTS" :key="p.key" class="flex gap-12">
        <component
          :is="p.icon"
          class="mt-px flex-none text-accent-text"
          :size="18"
          :stroke-width="1.75"
          aria-hidden="true"
        />
        <span>{{ t(`cloud.how.${p.key}`) }}</span>
      </li>
    </ul>
  </Card>
</template>
