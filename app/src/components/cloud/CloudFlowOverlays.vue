<script setup lang="ts">
// What the sync shows over the app (Trello #31): G05 (a game open on another device), G04 (two
// versions of a save: only outside the game and while no game is starting, since its answer
// writes the game's saves; it waits in the queue meanwhile), and G14's strip while a closed
// game's saves go up to Drive. State in cloudState.ts; App.vue places it.
import { computed } from 'vue'
import { flow } from '@/components/game/gameFlow'
import CloudConflictDialog from './CloudConflictDialog.vue'
import CloudInUseDialog from './CloudInUseDialog.vue'
import CloudPushStrip from './CloudPushStrip.vue'
import { answerInUse, cloud } from './cloudState'

const props = defineProps<{ inGame: boolean }>()

const conflict = computed(() =>
  props.inGame || flow.launching ? null : (cloud.conflicts[0] ?? null),
)
</script>

<template>
  <CloudInUseDialog v-if="cloud.inUse" :ask="cloud.inUse" @answer="answerInUse" />
  <!-- Keyed by the game: the next question opens afresh with its newer version chosen. -->
  <CloudConflictDialog v-else-if="conflict" :key="conflict.id" :conflict="conflict" />
  <CloudPushStrip v-if="!inGame" />
</template>
