<script setup lang="ts">
// What the game flow shows over the game or the shell (boards «Stati» and «Errori»):
// G01 starting (with G03, the saves coming down from Google Drive), G02 cannot start, G06 the
// quick-menu hint, G14 closing, the resume question, E04 the in-game save cannot be read, E11 the
// emulator stopped, E12 slower than normal.
// State in gameFlow.ts; App.vue reacts to the emitted choices.
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CloudPull from '@/components/cloud/CloudPull.vue'
import { progressOf, transferShare } from '@/components/cloud/cloud'
import { cloud } from '@/components/cloud/cloudState'
import { flow } from './gameFlow'
import CrashDialog from './CrashDialog.vue'
import GameScene from './GameScene.vue'
import QuickMenuHint from './QuickMenuHint.vue'
import ResumeDialog from './ResumeDialog.vue'
import SaveBrokenDialog from './SaveBrokenDialog.vue'
import SlowDialog from './SlowDialog.vue'
import StartFailedDialog from './StartFailedDialog.vue'
import type { StateLoadBlock } from '@/emu'

defineProps<{
  /** The open game's `stateLoadBlock`: the resume question shows «Riprendi» disabled with it. */
  loadBlock: StateLoadBlock | null
}>()
const emit = defineEmits<{
  startErrorClose: []
  resume: [boolean]
  crashCopy: []
  crashLibrary: []
  crashResume: []
  slowClose: []
  brokenUse: []
  brokenPlay: []
  brokenClose: []
}>()

const { t } = useI18n()
const pull = computed(() => progressOf(cloud.progress, 'pulling', null))
</script>

<template>
  <GameScene
    v-if="flow.launching"
    :title="flow.launching.title"
    :colour="flow.launching.colour"
    :value="transferShare(pull)"
  >
    <!-- G03 (#31): the newest saves coming down from Google Drive first. -->
    <CloudPull v-if="pull" :progress="pull" />
    <div v-else class="text-ink-muted">{{ t('flow.starting') }}</div>
  </GameScene>

  <GameScene v-if="flow.closing" :title="t('flow.closing', { title: flow.closing.title })">
    <ul class="m-0 list-none p-0 text-center text-ink-muted">
      <li v-if="flow.closing.autosave">{{ t('flow.stepAuto') }}</li>
      <li>{{ t('flow.stepInGame') }}</li>
    </ul>
  </GameScene>

  <QuickMenuHint :shown="flow.hint" />

  <StartFailedDialog
    v-if="flow.startError"
    :title="flow.startError.title"
    :reason="flow.startError.reason"
    :file="flow.startError.file"
    :path="flow.startError.path"
    @close="emit('startErrorClose')"
  />

  <SaveBrokenDialog
    v-if="flow.saveBroken"
    :title="flow.saveBroken.title"
    :backup-at="flow.saveBroken.backupAt"
    @use="emit('brokenUse')"
    @play="emit('brokenPlay')"
    @close="emit('brokenClose')"
  />

  <ResumeDialog
    v-if="flow.resumeAsk"
    :load-block="loadBlock"
    @answer="resume => emit('resume', resume)"
  />

  <CrashDialog
    v-if="flow.crashed"
    :console-id="flow.crashed.console"
    :saved-at="flow.crashed.at"
    @copy="emit('crashCopy')"
    @library="emit('crashLibrary')"
    @resume="emit('crashResume')"
  />

  <SlowDialog
    v-if="flow.slow"
    :fps="flow.slow.fps"
    :target="flow.slow.target"
    @close="emit('slowClose')"
  />
</template>
