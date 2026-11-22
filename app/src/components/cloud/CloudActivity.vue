<script setup lang="ts">
// The tab «Attività recenti» (Trello #31): what the sync did since the app started (the backend
// keeps the last 50), newest first; a failure says why in the error's own text.
import type { Component } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  Check,
  CircleAlert,
  Settings2,
  Split,
  Trash2,
} from '@lucide/vue'
import type { CloudActivity, CloudActivityKind } from '@/emu'
import { Card, CardVariant } from '@/components/ui/card'
import RowColumns from '@/components/settings/RowColumns.vue'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import { errorText, saveWhenLabel } from '@/i18n/labels'
import { activityLine } from './cloud'
import { gameTitle } from './cloudState'
import { useCloudActivity } from './useCloudLists'

const { t } = useI18n()
const { rows } = useCloudActivity()

const ICON: Record<CloudActivityKind, Component> = {
  pulled: ArrowDownToLine,
  downloaded: ArrowDownToLine,
  pushed: ArrowUpFromLine,
  uploaded: ArrowUpFromLine,
  settings: Settings2,
  conflict: Split,
  resolved: Check,
  removed: Trash2,
  failed: CircleAlert,
}

function text(a: CloudActivity) {
  const line = activityLine(a)
  const title = line.game ? gameTitle(line.game) : t('cloud.activity.someGame')
  return t(line.key, { n: line.n, title }, line.n)
}
</script>

<template>
  <template v-if="rows">
    <Card v-if="rows.length" :variant="CardVariant.List">
      <RowColumns>
        <div
          v-for="(a, i) in rows"
          :key="`${a.at}-${i}`"
          class="flex items-center gap-8 px-12 py-6 not-first:border-t not-first:border-line"
        >
          <component
            :is="ICON[a.kind]"
            class="flex-none"
            :class="a.kind === 'failed' ? 'text-danger' : 'text-ink-muted'"
            :size="18"
            :stroke-width="1.75"
            aria-hidden="true"
          />
          <div class="flex min-w-0 flex-1 flex-col gap-2">
            <span class="wrap-anywhere">{{ text(a) }}</span>
            <span v-if="a.error" class="text-caption text-danger">{{ errorText(a.error) }}</span>
          </div>
          <span class="flex-none text-caption text-ink-muted tabular-nums">{{
            saveWhenLabel(a.at)
          }}</span>
        </div>
      </RowColumns>
    </Card>
    <EmptyState
      v-else
      as="div"
      :variant="EmptyStateVariant.Card"
      :text="t('cloud.activity.empty')"
    />
  </template>
</template>
