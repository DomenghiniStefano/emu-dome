<script setup lang="ts">
// One side of a save conflict (V06): the last screen, the device («Più recente» on the newer
// one), when it was saved, the play time when known, the size, and «Tieni questa». The newer
// side's «Tieni questa» takes the focus first.
import { useI18n } from 'vue-i18n'
import { Check } from '@lucide/vue'
import { playTimeLabel, saveWhenLabel, sizeLabel } from '@/i18n/labels'
import { Button } from '@/components/ui/button'
import { Card, CardPad } from '@/components/ui/card'
import type { ConflictVersion } from './conflict'
import SaveThumb from './SaveThumb.vue'
import { Badge } from '@/components/ui/badge'

defineProps<{ version: ConflictVersion; newest: boolean }>()
const emit = defineEmits<{ keep: [] }>()

const { t } = useI18n()
</script>

<template>
  <Card :pad="CardPad.Snug" class="gap-8 text-ink">
    <SaveThumb :src="version.thumbUrl" class="aspect-3/2 rounded-control" />
    <div class="flex items-center gap-8 text-heading">
      <span>{{ version.device }}</span>
      <Badge v-if="newest">{{ t('saves.conflict.newest') }}</Badge>
    </div>
    <dl
      class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-8 gap-y-2 text-caption text-ink-muted tabular-nums"
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
    <!-- at the bottom: both sides keep it on one line when one has a line less -->
    <Button
      class="mt-auto"
      :icon="Check"
      :data-nav-initial="newest ? '' : undefined"
      @click="emit('keep')"
    >
      {{ t('saves.conflict.keepThis') }}
    </Button>
  </Card>
</template>
