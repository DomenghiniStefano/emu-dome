<script setup lang="ts">
// One quick-save slot (board «Salvataggi» V02; #59 item 3, the one slot component): a 4:3
// thumbnail, the name, the date, «Gioca da qui» and «…». Saves (V02) uses it, and so will Game
// details › Salvataggi (L05). An empty slot shows no buttons. `outdated`: the state was made by
// an older version of the emulator, «Potrebbe non caricarsi» under the date.
//
// The parent decides what «…» offers (V03): the Automatico slot has no Rinomina, Blocca, Elimina.
import { useI18n } from 'vue-i18n'
import { Ellipsis, TriangleAlert } from '@lucide/vue'

withDefaults(
  defineProps<{
    name: string
    /** The date, already as text; null when unknown. */
    when?: string | null
    thumbUrl?: string | null
    empty?: boolean
    outdated?: boolean
  }>(),
  { when: null, thumbUrl: null, empty: false, outdated: false },
)
const emit = defineEmits<{ play: []; more: [Event] }>()

const { t } = useI18n()
</script>

<template>
  <div class="slot" :class="{ empty }">
    <span class="thumb" aria-hidden="true">
      <img v-if="thumbUrl && !empty" :src="thumbUrl" alt="" />
    </span>
    <span class="name">{{ name }}</span>
    <span v-if="when" class="when num">{{ when }}</span>
    <span v-if="outdated && !empty" class="old">
      <TriangleAlert :size="12" :stroke-width="1.75" aria-hidden="true" />
      {{ t('saves.quick.mightNotLoad') }}
    </span>
    <span v-if="!empty" class="actions">
      <button class="play-here" @click="emit('play')">{{ t('saves.quick.load') }}</button>
      <button
        class="more"
        :aria-label="t('saves.slotMore')"
        :title="t('saves.slotMore')"
        @click="emit('more', $event)"
      >
        <Ellipsis :size="16" :stroke-width="1.75" aria-hidden="true" />
      </button>
    </span>
  </div>
</template>

<style scoped>
.slot {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  border-radius: var(--radius-10);
  background: var(--surface);
  border: 1px solid var(--line);
  text-align: left;
  min-width: 0;
}
.thumb {
  display: block;
  aspect-ratio: 4 / 3;
  border-radius: 4px;
  background: #000;
  overflow: hidden;
}
.slot.empty .thumb {
  background: var(--surface-raised);
}
.thumb img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  image-rendering: pixelated;
}
.name {
  font-weight: 600;
}
.when {
  font-size: 12px;
  line-height: 16px;
  color: var(--ink-muted);
}
.old {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  line-height: 16px;
  color: var(--warning);
}
.actions {
  display: flex;
  gap: 6px;
  margin-top: 2px;
}
.actions button {
  height: 28px;
  padding: 0 10px;
  border-radius: var(--radius-6);
  font-weight: 500;
  font-size: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.play-here {
  flex: 1;
  min-width: 0;
  border: 0;
  background: var(--accent-soft);
  color: var(--accent-text);
}
.more {
  flex: 0 0 auto;
  border: 1px solid var(--line);
  background: var(--surface-raised);
}
.actions button:hover {
  background: var(--surface-hover);
}
</style>
