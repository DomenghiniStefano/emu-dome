<script setup lang="ts">
// The game flow's blocking dialogs on FlowDialog (glass over the scrim, its own nav scope, B is
// the safe exit): the resume question, E12 slow, E04 the save that cannot be read. They are open
// while mounted, so each opens from its button here and closes with any answer.
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import ResumeDialog from '@/components/game/ResumeDialog.vue'
import SaveBrokenDialog from '@/components/game/SaveBrokenDialog.vue'
import SlowDialog from '@/components/game/SlowDialog.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import { FIXED } from './mock'

type Shown = 'resume' | 'resumeBlocked' | 'slow' | 'broken' | 'brokenNoBackup' | null
const shown = ref<Shown>(null)
const close = () => (shown.value = null)
</script>

<template>
  <KitSection title="Game flow dialogs" :group="KitGroup.Game">
    <KitState label="Resume: Riprendi is the main action">
      <Button @click="shown = 'resume'">Resume question</Button>
    </KitState>
    <KitState label="Resume, the core cannot load: Riprendi off, the reason">
      <Button @click="shown = 'resumeBlocked'">Resume, load blocked</Button>
    </KitState>
    <KitState label="E12 slow (warning title)">
      <Button @click="shown = 'slow'">Slow</Button>
    </KitState>
    <KitState label="E04 save unreadable, with a backup / without">
      <div class="flex gap-8">
        <Button @click="shown = 'broken'">With backup</Button>
        <Button @click="shown = 'brokenNoBackup'">No backup</Button>
      </div>
    </KitState>

    <ResumeDialog v-if="shown === 'resume'" :load-block="null" @answer="close" />
    <ResumeDialog v-if="shown === 'resumeBlocked'" load-block="vulkanNoTitleId" @answer="close" />
    <SlowDialog v-if="shown === 'slow'" :fps="41.6" :target="59.7" @close="close" />
    <SaveBrokenDialog
      v-if="shown === 'broken'"
      title="Pocket Quest Advance"
      :backup-at="FIXED - 26 * 3_600_000"
      @use="close"
      @play="close"
      @close="close"
    />
    <SaveBrokenDialog
      v-if="shown === 'brokenNoBackup'"
      title="Pocket Quest Advance"
      :backup-at="null"
      @use="close"
      @play="close"
      @close="close"
    />

    <template #review>
      <KitIssue
        title="E12 consiglia un’azione ma non ha il pulsante per farla"
        :status="IssueStatus.Open"
      >
        Il testo dice «Prova la preimpostazione «Prestazioni massime» nelle Impostazioni», ma
        l’unico pulsante è «Non ora», che fa pensare a un «Ora» che non c’è. Per seguire il
        consiglio bisogna chiudere il gioco, aprire le Impostazioni e cercare la preimpostazione da
        soli.
        <template #solution>
          da decidere col proprietario: un pulsante principale «Usa Prestazioni massime» (quando le
          preimpostazioni avranno un backend: oggi nessuna card azione o preimpostazione è eseguita)
          accanto a «Non ora»; fino ad allora il pulsante si chiama «Ho capito» o «Chiudi» e il
          testo dice dove trovare la preimpostazione.
        </template>
      </KitIssue>
      <KitIssue
        title="E03 (il gioco si è fermato): tre pulsanti su due righe"
        :status="IssueStatus.Open"
      >
        Nel dialogo del crash, con il salvataggio automatico, i pulsanti sono tre («Copia i dettagli
        per una segnalazione», «Torna alla libreria», «Riprendi dal salvataggio automatico»): nella
        larghezza del dialogo non stanno su una riga, e l’azione principale finisce da sola sulla
        seconda riga, a destra, sotto le altre due. Si vede nello scenario shell-flow-crash a 1024 ×
        640.
        <template #solution>
          da decidere nella primitiva Dialog (DialogContent / DialogFooter, non dal dialogo): un
          dialogo con tre azioni un po’ più largo, oppure le azioni secondarie a sinistra e la
          principale a destra sulla stessa riga, oppure «Copia i dettagli» come link nel testo.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
