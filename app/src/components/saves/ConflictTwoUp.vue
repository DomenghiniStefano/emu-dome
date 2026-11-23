<script setup lang="ts">
// Two versions of the same save, side by side (design system component ConflictTwoUp; board
// «Salvataggi» V06, and G04 at game start). Each side: device and time, play time, the last
// screen if there is one, size; the newer one carries «Più recente», never the accent: the
// choice stays the user's. «Tieni questa» under each side, «Tienile entrambe» under both (the
// other becomes a dated backup); closing = «Decidi dopo, apro la versione di questo dispositivo».
//
// Not shown anywhere yet: a local conflict does not exist (one device writes its own saves).
// The Google Drive sync step mounts it with the two versions it found.
// Controller / keyboard: its own nav scope through FlowDialog, B = «Decidi dopo».
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Copy } from '@lucide/vue'
import { Button, ButtonSize } from '@/design-system/components/button'
import FlowDialog from '@/components/game/FlowDialog.vue'
import ConflictSide from './ConflictSide.vue'
import type { ConflictVersion } from './conflict'
import { newerIndex } from './conflict'

export type { ConflictVersion } from './conflict'

const props = defineProps<{ title: string; versions: [ConflictVersion, ConflictVersion] }>()
const emit = defineEmits<{ keep: [index: 0 | 1]; keepBoth: []; later: [] }>()

const { t } = useI18n()

const newest = computed(() => newerIndex(props.versions))
const sides = computed(() => props.versions.map((version, i) => ({ version, i: i as 0 | 1 })))
</script>

<template>
  <FlowDialog :title="t('saves.conflict.title')" escape-closes @back="emit('later')">
    <p class="m-0">{{ t('saves.conflict.text', { title }) }}</p>
    <div class="mt-4 grid grid-cols-2 gap-12">
      <ConflictSide
        v-for="{ version, i } in sides"
        :key="i"
        :version="version"
        :newest="i === newest"
        @keep="emit('keep', i)"
      />
    </div>
    <template #actions>
      <!-- The way out stays words (#104, like «Non ora»), the choice beside it has its icon. -->
      <Button :size="ButtonSize.Medium" @click="emit('later')">
        {{ t('saves.conflict.later') }}
      </Button>
      <Button :size="ButtonSize.Medium" :icon="Copy" @click="emit('keepBoth')">
        {{ t('saves.conflict.keepBoth') }}
      </Button>
    </template>
  </FlowDialog>
</template>
