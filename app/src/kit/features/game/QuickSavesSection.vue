<script setup lang="ts">
// The quick menu's quick saves (G07, G08): the slot tiles (QuickSlotTile) with Carica and Salva
// under them, over a paused game in the menu's glass panel.
import { ref } from 'vue'
import { RotateCcw, Save } from '@lucide/vue'
import { AUTO_SLOT } from '@/emu'
import QuickSaves from '@/components/quick-menu/QuickSaves.vue'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import GameStage from './GameStage.vue'
import { FIXED, SLOTS, thumb } from './mock'

const chosen = ref(2)
const EVERY_SLOT = [
  ...SLOTS,
  ...[5, 6, 7].map(slot => ({ slot, savedAt: null, thumbUrl: null })),
  { slot: 8, savedAt: FIXED - 400 * 3_600_000, thumbUrl: thumb('indigo', 'sienna') },
]
const isEmpty = (slot: number) => !SLOTS.find(s => s.slot === slot)?.savedAt
</script>

<template>
  <KitSection title="Quick saves" :group="KitGroup.Game">
    <KitState label="Main screen, a slot chosen (click the tiles), its save just made">
      <GameStage panel>
        <QuickSaves
          :slots="SLOTS"
          :chosen="chosen"
          :chosen-empty="isEmpty(chosen)"
          :auto-chosen="chosen === AUTO_SLOT"
          :saved-slot="chosen === 2 ? 2 : null"
          :load-block="null"
          @pick="s => (chosen = s)"
        />
      </GameStage>
    </KitState>
    <KitState label="Automatico chosen: Salva replaced by a note">
      <GameStage panel>
        <QuickSaves
          :slots="SLOTS"
          :chosen="AUTO_SLOT"
          :chosen-empty="false"
          auto-chosen
          :saved-slot="null"
          :load-block="null"
        />
      </GameStage>
    </KitState>
    <KitState label="An empty slot chosen, and a core that cannot load (Carica off, reason)">
      <GameStage panel>
        <QuickSaves
          :slots="SLOTS"
          :chosen="3"
          chosen-empty
          :auto-chosen="false"
          :saved-slot="null"
          load-block="vulkanNoTitleId"
        />
      </GameStage>
    </KitState>
    <KitState label="G08: every slot, the backups row">
      <GameStage panel>
        <QuickSaves
          :slots="EVERY_SLOT"
          :chosen="8"
          :chosen-empty="false"
          :auto-chosen="false"
          :saved-slot="null"
          :load-block="null"
          all
        />
      </GameStage>
    </KitState>

    <template #review>
      <KitIssue
        title="La nota dello slot automatico sembra un pulsante vuoto"
        :status="IssueStatus.Proposed"
      >
        Con «Automatico» scelto, al posto di «Salva» c’è un riquadro tratteggiato alto quanto un
        pulsante con dentro una frase. Il bordo tratteggiato è lo stesso degli slot vuoti sopra,
        quindi si legge come «qui manca qualcosa»; e la coppia Carica / Salva perde la sua forma.
        Altrove (Carica bloccato, terzo stato) la regola è «disabilitato con il motivo sotto».
        <template #current>
          <div class="grid w-panel max-w-full grid-cols-[1fr_1fr] gap-8">
            <Button :variant="ButtonVariant.Soft" :size="ButtonSize.Menu" class="min-w-0">
              <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
              Carica dallo slot automatico
            </Button>
            <span
              class="flex items-center justify-center self-stretch rounded-control border border-dashed border-line-strong px-8 text-center text-caption-inline text-ink-muted"
            >
              Lo slot automatico si aggiorna da solo
            </span>
          </div>
        </template>
        <template #proposed>
          <div class="flex w-panel max-w-full flex-col gap-block-gap">
            <div class="grid grid-cols-[1fr_1fr] gap-8">
              <Button :variant="ButtonVariant.Soft" :size="ButtonSize.Menu" class="min-w-0">
                <RotateCcw :size="18" :stroke-width="1.75" aria-hidden="true" />
                Carica dallo slot automatico
              </Button>
              <Button
                :variant="ButtonVariant.Soft"
                :size="ButtonSize.Menu"
                :fade="ButtonFade.Dim"
                class="min-w-0"
                disabled
              >
                <Save :size="18" :stroke-width="1.75" aria-hidden="true" />
                Salva
              </Button>
            </div>
            <span class="text-caption text-ink-muted">Lo slot automatico si aggiorna da solo.</span>
          </div>
        </template>
        <template #solution>
          in QuickSaves.vue, con <code>autoChosen</code> mostrare il pulsante Salva disabilitato
          (<code>ButtonFade.Dim</code>, come Carica) e la frase sotto in
          <code>text-caption text-ink-muted</code>, nello stesso posto della riga di
          <code>loadBlock</code>. Il riquadro tratteggiato sparisce.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
