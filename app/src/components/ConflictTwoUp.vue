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
import { playTimeLabel, saveWhenLabel, sizeLabel } from '../i18n/labels'
import FlowDialog from './FlowDialog.vue'

/** One side of the conflict. */
export interface ConflictVersion {
  /** «Questo computer», «Galaxy A71»: the device's own name, from the sync step. */
  device: string
  /** Unix ms. */
  savedAt: number
  playTimeSecs: number | null
  size: number
  /** The last screen, when the version has a quick save with a thumbnail. */
  thumbUrl: string | null
}

const props = defineProps<{ title: string; versions: [ConflictVersion, ConflictVersion] }>()
const emit = defineEmits<{ keep: [index: 0 | 1]; keepBoth: []; later: [] }>()

const { t } = useI18n()

const newest = computed<0 | 1>(() =>
  props.versions[1].savedAt > props.versions[0].savedAt ? 1 : 0,
)
const sides = computed(() => props.versions.map((v, i) => ({ v, i: i as 0 | 1 })))
</script>

<template>
  <FlowDialog :title="t('saves.conflict.title')" @back="emit('later')">
    <p class="p">{{ t('saves.conflict.text', { title }) }}</p>
    <div class="two">
      <div v-for="{ v, i } in sides" :key="i" class="side">
        <span class="shot" aria-hidden="true">
          <img v-if="v.thumbUrl" :src="v.thumbUrl" alt="" />
        </span>
        <div class="device">
          <span>{{ v.device }}</span>
          <span v-if="i === newest" class="badge">{{ t('saves.conflict.newest') }}</span>
        </div>
        <dl class="facts num">
          <dt>{{ t('saves.conflict.saved') }}</dt>
          <dd>{{ saveWhenLabel(v.savedAt) }}</dd>
          <template v-if="v.playTimeSecs !== null">
            <dt>{{ t('saves.conflict.playTime') }}</dt>
            <dd>{{ playTimeLabel(v.playTimeSecs, true) }}</dd>
          </template>
          <dt>{{ t('saves.conflict.size') }}</dt>
          <dd>{{ sizeLabel(v.size) }}</dd>
        </dl>
        <button
          class="btn"
          :data-nav-initial="i === newest ? '' : undefined"
          @click="emit('keep', i)"
        >
          {{ t('saves.conflict.keepThis') }}
        </button>
      </div>
    </div>
    <template #actions>
      <button class="btn" @click="emit('later')">{{ t('saves.conflict.later') }}</button>
      <button class="btn" @click="emit('keepBoth')">{{ t('saves.conflict.keepBoth') }}</button>
    </template>
  </FlowDialog>
</template>

<style scoped>
.p {
  margin: 0;
}
.two {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  margin-top: 6px;
}
.side {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--ink);
}
.shot {
  display: block;
  aspect-ratio: 3 / 2;
  border-radius: var(--radius-6);
  background: #000;
  overflow: hidden;
}
.shot img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  image-rendering: pixelated;
}
.device {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
.badge {
  font-size: 11px;
  line-height: 16px;
  font-weight: 500;
  padding: 0 6px;
  border-radius: 4px;
  background: var(--surface-raised);
  color: var(--ink-muted);
}
.facts {
  margin: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 4px 12px;
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.facts dd {
  margin: 0;
  color: var(--ink);
}
</style>
