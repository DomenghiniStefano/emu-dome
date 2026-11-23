<script setup lang="ts">
// CoverChoices (cover/CoverChoices.vue): L12 «Scegli la copertina», its first step, in the Wide
// dialog CoverPicker opens. It takes the controller's focus when it opens, so it sits behind a
// button here: open it to see it.
import { ref } from 'vue'
import CoverChoices from '@/components/cover/CoverChoices.vue'
import type { CoverChoice } from '@/components/cover/coverChoice'
import { Button } from '@/design-system/components/button'
import { Dialog, DialogContent, DialogSize, DialogTrigger } from '@/design-system/components/dialog'
import KitIssue from '@/design-system/kit/KitIssue.vue'
import KitSection from '@/design-system/kit/KitSection.vue'
import KitState from '@/design-system/kit/KitState.vue'
import { KitGroup } from '@/design-system/kit/review'
import { GBA, N3DS_USER, SHOTS } from './mock'

const openIcon = ref(false)
const openNone = ref(false)
const iconChoice = ref<CoverChoice>('user')
const noneChoice = ref<CoverChoice>('generated')
</script>

<template>
  <KitSection title="Cover choices" :group="KitGroup.Library">
    <KitState label="Four choices: the ROM has an icon, 2 screenshots, a user image">
      <Dialog v-model:open="openIcon">
        <DialogTrigger as-child><Button>Mountain Ranger 3D</Button></DialogTrigger>
        <DialogContent :size="DialogSize.Wide" aria-label="Cover" :aria-describedby="undefined">
          <CoverChoices
            v-model="iconChoice"
            :game="N3DS_USER"
            :shots="SHOTS"
            :busy="false"
            :error="null"
            @cancel="openIcon = false"
            @next="openIcon = false"
          />
        </DialogContent>
      </Dialog>
    </KitState>
    <KitState label="Three choices: no icon, no screenshot yet, an error">
      <Dialog v-model:open="openNone">
        <DialogTrigger as-child><Button>Pocket Quest Advance</Button></DialogTrigger>
        <DialogContent :size="DialogSize.Wide" aria-label="Cover" :aria-describedby="undefined">
          <CoverChoices
            v-model="noneChoice"
            :game="GBA"
            :shots="[]"
            :busy="false"
            error="L’immagine non si può aprire: scegline un’altra."
            @cancel="openNone = false"
            @next="openNone = false"
          />
        </DialogContent>
      </Dialog>
    </KitState>
    <template #review>
      <KitIssue title="Con tre scelte le schede diventano grandi e vuote">
        Le scelte stanno in <code>grid-cols-3</code> o <code>grid-cols-4</code> nella stessa
        larghezza del dialogo: senza l’icona della ROM ogni scheda si allarga di un terzo e il
        dialogo cresce in altezza. Senza screenshot, due schede su tre mostrano una grande anteprima
        vuota (un riquadro grigio e uno tratteggiato con un «+» piccolo). Apri il secondo esempio
        sopra.
        <template #solution>
          colonne della stessa larghezza in entrambi i casi (sempre quattro colonne con le tre
          schede centrate, oppure il dialogo più stretto con tre), e nell’anteprima vuota di «Da uno
          screenshot» un’icona Lucide (per esempio <code>Camera</code>) come il «+» di «Immagine
          tua». Da decidere col proprietario.
        </template>
      </KitIssue>
      <KitIssue title="«Icona del gioco» mostra un quadrato colorato invece dell’icona">
        L’anteprima dell’icona c’è solo se il gioco la usa già come copertina: per un gioco con la
        copertina generata o un’immagine sua (il primo esempio sopra, un 3DS con l’icona nella ROM)
        la scheda mostra un quadrato del colore del gioco. Chi sceglie non vede cosa otterrà.
        <template #solution>
          serve dal backend l’indirizzo dell’icona anche quando non è la copertina (un campo
          <code>iconUrl</code> in <code>LibraryGame</code>, servito da <code>emu://</code>), poi
          CoverChoices usa quello. Lavoro di Rust e del contratto, non solo d’interfaccia.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
