<script setup lang="ts">
// What the game flow shows over the game or the shell (boards «Stati» and «Errori»):
// G01 starting, G02 cannot start, G06 the quick-menu hint, G14 closing, the resume question,
// E11 the emulator stopped, E12 slower than normal. State in gameFlow.ts; App.vue reacts to the
// emitted choices.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { flow } from '../gameFlow'
import FlowDialog from './FlowDialog.vue'

const emit = defineEmits<{
  startErrorClose: []
  resume: [boolean]
  crashCopy: []
  crashLibrary: []
  crashResume: []
  slowClose: []
}>()

const { t, locale } = useI18n()

/** «2 minuti fa» from a Unix ms, in the interface language. */
const savedWhen = computed(() => {
  const at = flow.crashed?.at
  if (!at) return ''
  const rtf = new Intl.RelativeTimeFormat(locale.value, { numeric: 'auto' })
  const minutes = Math.round((at - Date.now()) / 60000)
  return Math.abs(minutes) < 60
    ? rtf.format(minutes, 'minute')
    : rtf.format(Math.round(minutes / 60), 'hour')
})
</script>

<template>
  <!-- G01: the game is starting -->
  <div
    v-if="flow.launching"
    class="launch"
    :style="{ '--cover': flow.launching.colour ?? 'var(--surface)' }"
    role="status"
  >
    <div class="launch-title">{{ flow.launching.title }}</div>
    <div class="launch-line">{{ t('flow.starting') }}</div>
    <div class="bar" aria-hidden="true"><span></span></div>
  </div>

  <!-- G14: closing -->
  <div v-if="flow.closing" class="launch closing" role="status">
    <div class="launch-title">{{ t('flow.closing', { title: flow.closing.title }) }}</div>
    <ul class="steps">
      <li v-if="flow.closing.autosave">{{ t('flow.stepAuto') }}</li>
      <li>{{ t('flow.stepInGame') }}</li>
    </ul>
    <div class="bar" aria-hidden="true"><span></span></div>
  </div>

  <!-- G06: the quick menu hint, the first few games -->
  <Transition name="fade">
    <div v-if="flow.hint" class="hint glass-thick" role="status">
      <kbd>{{ t('flow.hintEsc') }}</kbd>
      <span>{{ t('flow.hintOr') }}</span>
      <kbd>{{ t('flow.hintHome') }}</kbd>
      <span>{{ t('flow.hintText') }}</span>
    </div>
  </Transition>

  <!-- G02: cannot start -->
  <FlowDialog
    v-if="flow.startError"
    :title="t('flow.startFailed', { title: flow.startError.title })"
    tone="danger"
    @back="emit('startErrorClose')"
  >
    <p class="p">{{ flow.startError.reason }}</p>
    <template #actions>
      <button class="btn btn-primary" data-nav-initial @click="emit('startErrorClose')">
        {{ t('flow.gotIt') }}
      </button>
    </template>
  </FlowDialog>

  <!-- saves.resume = ask -->
  <FlowDialog v-if="flow.resumeAsk" :title="t('flow.resumeTitle')" @back="emit('resume', false)">
    <template #actions>
      <button class="btn" @click="emit('resume', false)">{{ t('flow.restart') }}</button>
      <button class="btn btn-primary" data-nav-initial @click="emit('resume', true)">
        {{ t('flow.resume') }}
      </button>
    </template>
  </FlowDialog>

  <!-- E11: the emulator stopped -->
  <FlowDialog
    v-if="flow.crashed"
    :title="t('flow.crashTitle')"
    tone="danger"
    @back="emit('crashLibrary')"
  >
    <p class="p">
      {{ t('flow.crashText', { console: t(`console.${flow.crashed.console}`) }) }}
      {{ flow.crashed.at ? t('flow.crashSaved', { when: savedWhen }) : t('flow.crashNoSave') }}
    </p>
    <template #actions>
      <button class="btn" @click="emit('crashCopy')">{{ t('flow.copyDetails') }}</button>
      <button
        class="btn"
        :data-nav-initial="flow.crashed.at ? undefined : ''"
        @click="emit('crashLibrary')"
      >
        {{ t('flow.backToLibrary') }}
      </button>
      <button
        v-if="flow.crashed.at"
        class="btn btn-primary"
        data-nav-initial
        @click="emit('crashResume')"
      >
        {{ t('flow.resumeAuto') }}
      </button>
    </template>
  </FlowDialog>

  <!-- E12: slower than normal -->
  <FlowDialog
    v-if="flow.slow"
    :title="
      t('flow.slowTitle', { fps: Math.round(flow.slow.fps), target: Math.round(flow.slow.target) })
    "
    tone="warning"
    @back="emit('slowClose')"
  >
    <p class="p">{{ t('flow.slowText') }}</p>
    <template #actions>
      <button class="btn" data-nav-initial @click="emit('slowClose')">
        {{ t('flow.notNow') }}
      </button>
    </template>
  </FlowDialog>
</template>

<style scoped>
.launch {
  position: fixed;
  inset: 0;
  z-index: 35;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  background: radial-gradient(60% 50% at 50% 30%, var(--cover) 0%, transparent 75%), var(--bg);
}
.closing {
  --cover: var(--surface);
}
.launch-title {
  font-size: 32px;
  line-height: 38px;
  font-weight: 600;
  letter-spacing: -0.02em;
  text-align: center;
  padding: 0 16px;
}
.launch-line,
.steps {
  color: var(--ink-muted);
}
.steps {
  margin: 0;
  padding: 0;
  list-style: none;
  text-align: center;
}
.bar {
  width: 200px;
  height: 4px;
  border-radius: 999px;
  overflow: hidden;
  background: var(--surface-raised);
}
.bar span {
  display: block;
  width: 40%;
  height: 100%;
  border-radius: 999px;
  background: var(--accent-text);
  animation: slide 1.2s ease-in-out infinite;
}
@keyframes slide {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(250%);
  }
}
.hint {
  position: fixed;
  z-index: 30;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: var(--radius-10);
  color: var(--ink-muted);
  pointer-events: none;
}
kbd {
  font: inherit;
  font-size: 12px;
  padding: 1px 6px;
  border: 1px solid var(--line-strong);
  border-radius: 4px;
  color: var(--ink);
}
.fade-leave-active {
  transition: opacity 600ms;
}
.fade-leave-to {
  opacity: 0;
}
.p {
  margin: 0;
}
@media (prefers-reduced-motion: reduce) {
  .bar span {
    animation: none;
    width: 100%;
  }
}
</style>
