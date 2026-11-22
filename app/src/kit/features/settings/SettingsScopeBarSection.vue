<script setup lang="ts">
// The scope bar at the top of a settings page: the levels as cards (Tutti i giochi, the console's
// games with its menu, Solo questo gioco), «Mostra solo le impostazioni modificate», and the
// sentence of what a change does at the chosen level.
import { ref } from 'vue'
import SettingsScopeBar from '@/components/settings/SettingsScopeBar.vue'
import type { EditLevel } from '@/components/settings/settingsPage'
import type { ConsoleId } from '@/emu'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import { GAMES, levelName } from './mock'

const CONSOLES = [
  { id: 'gb', text: 'Game Boy' },
  { id: 'gbc', text: 'Game Boy Color' },
  { id: 'gba', text: 'Game Boy Advance' },
  { id: 'nds', text: 'Nintendo DS' },
]
const NOTE = 'Le voci in grigio valgono per tutti i giochi: le cambi da «Tutti i giochi».'

const two = ref<EditLevel>('global')
const twoConsole = ref<ConsoleId>('gba')
const three = ref<EditLevel>('game')
const onlyA = ref(false)
const onlyB = ref(true)
const focus = {
  scope: { level: 'game', console: 'nds', game: GAMES.long.id },
  title: GAMES.long.title,
} as const
</script>

<template>
  <KitSection title="Settings scope bar" :group="KitGroup.Settings">
    <KitState
      class="w-full"
      label="Two levels (Tutti i giochi / a console's games), the console's menu"
    >
      <div class="w-full">
        <SettingsScopeBar
          v-model:only-changed="onlyA"
          :levels="['global', 'console']"
          :effective="two"
          :context-console="twoConsole"
          :console-options="CONSOLES"
          :game-focus="null"
          :level-name="levelName"
          locked-note=""
          @level="l => (two = l)"
          @pick-console="id => (twoConsole = id as ConsoleId)"
        />
      </div>
    </KitState>
    <KitState
      class="w-full"
      label="Three levels, opened for a game (long title), only changed on, locked note"
    >
      <div class="w-full">
        <SettingsScopeBar
          v-model:only-changed="onlyB"
          :levels="['global', 'console', 'game']"
          :effective="three"
          context-console="nds"
          :console-options="CONSOLES"
          :game-focus="focus"
          :level-name="levelName"
          :locked-note="NOTE"
          @level="l => (three = l)"
        />
      </div>
    </KitState>

    <template #review>
      <KitIssue title="Il titolo del gioco ripetuto nella barra fissa" :status="IssueStatus.Open">
        Aperta per un gioco, la barra mostra il titolo due volte: sotto «Solo questo gioco» e nella
        frase «Cambi solo …». Con un titolo lungo la frase, unita alla nota sulle voci in grigio,
        arriva a due righe; la barra resta fissa in alto mentre la pagina scorre, quindi toglie
        quello spazio a ogni schermata di impostazioni.
        <template #solution>
          `settingsUi.scope.gameLine` senza il titolo («Cambi solo questo gioco. …»), che resta
          nella scheda; la nota delle voci in grigio su una riga sua più piccola (`text-caption`) in
          SettingsScopeBar.vue. Testi da rivedere con te.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
