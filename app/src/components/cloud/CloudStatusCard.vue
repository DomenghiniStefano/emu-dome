<script setup lang="ts">
// The status card of the Cloud section (Trello #31, mockup option A): the state pill and what it
// means (a run going on, a failure E07 / E08 / E09 in the existing texts, the games with two
// versions with a link to choose each, the games waiting for Drive, all synced), when the last
// sync went through, the Drive space, and «Sincronizza ora».
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CloudStatus, GameId } from '@/emu'
import { Badge, BadgeSize, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { errorText, saveWhenLabel } from '@/i18n/labels'
import CloudSpace from './CloudSpace.vue'
import { statusPill, type StatusPill } from './cloud'

const props = defineProps<{
  status: CloudStatus
  /** A game's title by its id (the library's, else Drive's list), for the links. */
  titleOf: (id: GameId) => string
}>()
const emit = defineEmits<{ syncNow: []; choose: [GameId] }>()
const { t } = useI18n()

const pill = computed(() => statusPill(props.status))

const BADGE: Record<StatusPill['kind'], BadgeVariant> = {
  syncing: BadgeVariant.Neutral,
  error: BadgeVariant.Warning,
  conflicts: BadgeVariant.Warning,
  pending: BadgeVariant.Neutral,
  synced: BadgeVariant.Success,
  never: BadgeVariant.Quiet,
}

const pillText = computed(() => {
  const p = pill.value
  return 'n' in p ? t(`cloud.pill.${p.kind}`, { n: p.n }) : t(`cloud.pill.${p.kind}`)
})
const headline = computed(() => {
  const p = pill.value
  if (p.kind === 'error') return errorText(p.error)
  if (p.kind === 'conflicts' || p.kind === 'pending')
    return t(`cloud.status.${p.kind}`, { n: p.n }, p.n)
  return t(`cloud.status.${p.kind}`)
})
const lastLine = computed(() => {
  const at = props.status.lastSync
  if (at === null) return null
  const when = saveWhenLabel(at)
  return pill.value.kind === 'conflicts'
    ? t('cloud.status.rest', { when })
    : t('cloud.status.last', { when })
})
</script>

<template>
  <Card class="flex-row flex-wrap items-center gap-x-16 gap-y-8">
    <div class="flex min-w-0 flex-1 basis-tile-min flex-col gap-4">
      <div class="flex flex-wrap items-center gap-8">
        <Badge :variant="BADGE[pill.kind]" :size="BadgeSize.Pill">{{ pillText }}</Badge>
        <span class="min-w-0 text-label-lead">{{ headline }}</span>
      </div>
      <span v-if="lastLine" class="text-caption text-ink-muted">{{ lastLine }}</span>
      <div v-if="status.conflicts.length" class="mt-4 flex flex-wrap gap-x-16 gap-y-4">
        <Button
          v-for="id in status.conflicts"
          :key="id"
          :variant="ButtonVariant.Link"
          :size="ButtonSize.Text"
          @click="emit('choose', id)"
        >
          {{ t('cloud.status.choose', { title: titleOf(id) }) }}
        </Button>
      </div>
    </div>
    <CloudSpace class="w-tile-min max-w-full" :status="status" />
    <Button
      :size="ButtonSize.Tall"
      :disabled="status.syncing"
      data-nav-initial
      @click="emit('syncNow')"
    >
      {{ t('cloud.status.syncNow') }}
    </Button>
  </Card>
</template>
