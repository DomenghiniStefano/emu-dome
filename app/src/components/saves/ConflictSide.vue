<script setup lang="ts">
// One side of a save conflict (V06): the last screen, the device («Più recente» on the newer
// one), when it was saved, the play time when known, the size, and «Tieni questa». The newer
// side's «Tieni questa» takes the focus first.
import { useI18n } from 'vue-i18n'
import { playTimeLabel, saveWhenLabel, sizeLabel } from '../../i18n/labels'
import { Button } from '@/components/ui/button'
import type { ConflictVersion } from './conflict'
import SaveBadge from './SaveBadge.vue'
import SaveThumb from './SaveThumb.vue'

defineProps<{ version: ConflictVersion; newest: boolean }>()
const emit = defineEmits<{ keep: [] }>()

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-col gap-10 rounded-10 border border-line bg-surface p-12 text-ink">
    <SaveThumb :src="version.thumbUrl" class="aspect-3/2 rounded-6" />
    <div class="flex items-center gap-8 text-heading">
      <span>{{ version.device }}</span>
      <SaveBadge v-if="newest">{{ t('saves.conflict.newest') }}</SaveBadge>
    </div>
    <dl
      class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-12 gap-y-4 text-caption text-ink-muted tabular-nums"
    >
      <dt>{{ t('saves.conflict.saved') }}</dt>
      <dd class="m-0 text-ink">{{ saveWhenLabel(version.savedAt) }}</dd>
      <template v-if="version.playTimeSecs !== null">
        <dt>{{ t('saves.conflict.playTime') }}</dt>
        <dd class="m-0 text-ink">{{ playTimeLabel(version.playTimeSecs, true) }}</dd>
      </template>
      <dt>{{ t('saves.conflict.size') }}</dt>
      <dd class="m-0 text-ink">{{ sizeLabel(version.size) }}</dd>
    </dl>
    <Button :data-nav-initial="newest ? '' : undefined" @click="emit('keep')">
      {{ t('saves.conflict.keepThis') }}
    </Button>
  </div>
</template>
