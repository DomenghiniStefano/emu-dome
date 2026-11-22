<script setup lang="ts">
// The tab «I tuoi dispositivi» (Trello #31): every device that published its library list on
// Drive, this one first, with its game count and when it last published.
import { useI18n } from 'vue-i18n'
import { MonitorSmartphone } from '@lucide/vue'
import type { CloudDevice } from '@/emu'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Card, CardVariant } from '@/components/ui/card'
import RowColumns from '@/components/settings/RowColumns.vue'
import { EmptyState, EmptyStateVariant } from '@/components/ui/empty-state'
import { saveWhenLabel } from '@/i18n/labels'
import { useCloudDevices } from './useCloudLists'

const { t } = useI18n()
const { rows } = useCloudDevices()

const meta = (d: CloudDevice) =>
  t('cloud.devices.meta', {
    games: t('cloud.devices.games', { n: d.games }, d.games),
    updated:
      d.updated === 0
        ? t('cloud.devices.never')
        : t('cloud.devices.updated', { when: saveWhenLabel(d.updated) }),
  })
</script>

<template>
  <template v-if="rows">
    <Card v-if="rows.length" :variant="CardVariant.List">
      <RowColumns>
        <div
          v-for="d in rows"
          :key="d.id"
          class="flex items-center gap-8 px-12 py-6 not-first:border-t not-first:border-line"
        >
          <MonitorSmartphone
            class="flex-none text-ink-muted"
            :size="20"
            :stroke-width="1.75"
            aria-hidden="true"
          />
          <div class="flex min-w-0 flex-1 flex-col gap-2">
            <span class="font-medium wrap-anywhere">{{ d.name }}</span>
            <span class="text-caption text-ink-muted">{{ meta(d) }}</span>
          </div>
          <Badge v-if="d.thisDevice" :variant="BadgeVariant.Neutral" :size="BadgeSize.Pill">
            {{ t('cloud.devices.this') }}
          </Badge>
        </div>
      </RowColumns>
    </Card>
    <EmptyState
      v-else
      as="div"
      :variant="EmptyStateVariant.Card"
      :text="t('cloud.devices.empty')"
    />
  </template>
</template>
