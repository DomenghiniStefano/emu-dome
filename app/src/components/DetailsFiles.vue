<script setup lang="ts">
// Game details › File e informazioni (L10, board «Dettagli» desktop): two cards, «Come appare»
// (GameLooksCard) and «File del gioco» (GameFileCard).
// Not built yet (no backend): the board's Google Drive line and «Rimuovi dalla libreria…».
// Phone in portrait: the same cards, one per row.
//
// Controller / keyboard: the shell's nav scope; L12 opens its own.
import type { LibraryGame, SettingDevice } from '../emu'
import GameFileCard from './details/GameFileCard.vue'
import GameLooksCard from './details/GameLooksCard.vue'

defineProps<{ game: LibraryGame; phone: boolean; device?: SettingDevice | null }>()
const emit = defineEmits<{
  /** «Cambia…»: L12. */
  cover: []
  error: [string]
}>()
</script>

<template>
  <div class="grid items-start gap-16" :class="phone ? 'grid-cols-1' : 'grid-cols-2'">
    <GameLooksCard
      :game="game"
      :phone="phone"
      @cover="emit('cover')"
      @error="text => emit('error', text)"
    />
    <GameFileCard :game="game" :phone="phone" :device="device" />
  </div>
</template>
