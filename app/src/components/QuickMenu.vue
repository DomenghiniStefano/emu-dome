<script setup lang="ts">
import { ref } from 'vue'
import { I18nT, useI18n } from 'vue-i18n'
import type { GameInfo, Speed, VideoMode } from '../emu'
import { speedLabel } from '../i18n/labels'
import { useNavScope } from '../nav'

const { t } = useI18n()

const props = defineProps<{
  game: GameInfo
  speed: Speed
  videoMode: VideoMode
  nativeAvailable: boolean
}>()
const emit = defineEmits<{
  videoMode: [VideoMode]
  resume: []
  speed: [Speed]
  save: [number]
  load: [number]
  reset: []
  close: []
}>()

// Controller and keyboard: B / Esc resume, the focus starts on "Resume".
const root = ref<HTMLElement | null>(null)
useNavScope(root, { back: () => emit('resume'), autofocus: true })

const speeds: Speed[] = [
  { kind: 'factor', value: 0.5 },
  { kind: 'factor', value: 1 },
  { kind: 'factor', value: 1.5 },
  { kind: 'factor', value: 2 },
  { kind: 'factor', value: 3 },
  { kind: 'factor', value: 4 },
  { kind: 'max' },
]

const speedKey = (s: Speed) => (s.kind === 'max' ? 'max' : String(s.value))

const isSelected = (s: Speed) =>
  s.kind === props.speed.kind &&
  (s.kind === 'max' || (props.speed.kind === 'factor' && s.value === props.speed.value))
</script>

<template>
  <aside ref="root" class="menu glass-thick" :aria-label="t('menu.label')">
    <header>
      <div>
        <div class="title">{{ game.title }}</div>
        <div class="muted">{{ t('menu.status', { console: game.console, core: game.core }) }}</div>
      </div>
    </header>

    <section>
      <div class="overline">{{ t('menu.speed') }}</div>
      <div class="segmented" role="radiogroup" :aria-label="t('menu.speedLabel')">
        <button
          v-for="s in speeds"
          :key="speedKey(s)"
          role="radio"
          :aria-checked="isSelected(s)"
          :class="{ on: isSelected(s) }"
          class="num"
          @click="emit('speed', s)"
        >
          {{ speedLabel(s) }}
        </button>
      </div>
      <I18nT keypath="menu.turboHint" tag="div" class="muted small">
        <template #key><kbd>Tab</kbd></template>
      </I18nT>
    </section>

    <section>
      <div class="overline">{{ t('menu.saves') }}</div>
      <div v-for="slot in [1, 2, 3]" :key="slot" class="slot">
        <span class="slot-name">{{ t('menu.slot', { slot }) }}</span>
        <button class="btn" @click="emit('save', slot)">{{ t('menu.save') }}</button>
        <button class="btn" @click="emit('load', slot)">{{ t('menu.load') }}</button>
      </div>
      <I18nT keypath="menu.savesHint" tag="div" class="muted small">
        <template #save><kbd>F1</kbd></template>
        <template #load><kbd>F4</kbd></template>
      </I18nT>
    </section>

    <section>
      <div class="overline">{{ t('menu.video') }}</div>
      <div class="segmented two" role="radiogroup" :aria-label="t('menu.videoLabel')">
        <button
          role="radio"
          :aria-checked="videoMode === 'native'"
          :class="{ on: videoMode === 'native' }"
          :disabled="!nativeAvailable"
          @click="emit('videoMode', 'native')"
        >
          {{ t('menu.native') }}
        </button>
        <button
          role="radio"
          :aria-checked="videoMode === 'webview'"
          :class="{ on: videoMode === 'webview' }"
          @click="emit('videoMode', 'webview')"
        >
          {{ t('menu.webview') }}
        </button>
      </div>
      <div class="muted small">
        {{ nativeAvailable ? t('menu.videoCompare') : t('menu.videoNoNative') }}
      </div>
    </section>

    <section class="actions">
      <button class="btn" @click="emit('reset')">{{ t('menu.reset') }}</button>
      <button class="btn danger" @click="emit('close')">{{ t('menu.close') }}</button>
    </section>

    <footer>
      <button class="btn btn-primary resume" data-nav-initial @click="emit('resume')">
        {{ t('menu.resume') }} <span class="esc">Esc</span>
      </button>
    </footer>
  </aside>
</template>

<style scoped>
.menu {
  position: fixed;
  top: 16px;
  right: 16px;
  bottom: 16px;
  width: min(420px, calc(100% - 32px));
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: enter 250ms var(--ease-out);
}
@keyframes enter {
  from {
    opacity: 0;
    transform: translateX(16px);
  }
}
@media (prefers-reduced-motion: reduce) {
  .menu {
    animation: none;
  }
}
header {
  padding: 20px 20px 8px;
}
.title {
  font-size: 15px;
  line-height: 20px;
  font-weight: 600;
}
.muted {
  color: var(--ink-muted);
}
.small {
  font-size: 12px;
  line-height: 16px;
}
section {
  padding: 12px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.segmented {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 2px;
  padding: 2px;
  border-radius: var(--radius-6);
  background: var(--surface-raised);
  border: 1px solid var(--line);
}
.segmented.two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}
.segmented button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.segmented button {
  height: 28px;
  border: 0;
  border-radius: 4px;
  background: none;
  color: var(--ink-muted);
  font-weight: 500;
}
.segmented button.on {
  background: var(--accent-soft);
  color: var(--accent-text);
  font-weight: 600;
}
.slot {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--radius-10);
  background: var(--surface);
}
.slot-name {
  flex: 1;
  font-weight: 500;
}
.actions {
  flex-direction: row;
  flex-wrap: wrap;
  margin-top: auto;
}
.danger {
  color: var(--danger);
}
footer {
  padding: 16px 20px 20px;
  border-top: 1px solid var(--line);
}
.resume {
  width: 100%;
}
.esc {
  font-weight: 400;
  opacity: 0.8;
}
</style>
