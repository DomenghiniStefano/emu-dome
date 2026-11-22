<script setup lang="ts">
// GameFileCard (details/GameFileCard.vue): L10 «File del gioco», with the DetailsCard and
// DetailsAction every card of the details' tabs is made of.
import DetailsAction from '@/components/details/DetailsAction.vue'
import DetailsCard from '@/components/details/DetailsCard.vue'
import GameFileCard from '@/components/details/GameFileCard.vue'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import LibraryGameFileCardProposal from '@/kit/proposals/LibraryGameFileCardProposal.vue'
import { IssueStatus, KitGroup } from '@/kit/review'
import { GB_LONG, GBA, GBC_MISSING, N3DS_USER } from './mock'
</script>

<template>
  <KitSection title="Game file card" :group="KitGroup.Library">
    <KitState label="GBA on the computer: SHA-1">
      <div class="w-dialog max-w-full"><GameFileCard :game="GBA" device="desktop" /></div>
    </KitState>
    <KitState label="3DS on the phone: title ID, 2 GB">
      <div class="w-dialog max-w-full"><GameFileCard :game="N3DS_USER" device="phone" /></div>
    </KitState>
    <KitState label="GB with no header facts">
      <div class="w-dialog max-w-full"><GameFileCard :game="GB_LONG" device="desktop" /></div>
    </KitState>
    <KitState label="File not reachable: nothing says so">
      <div class="w-dialog max-w-full"><GameFileCard :game="GBC_MISSING" device="desktop" /></div>
    </KitState>
    <KitState label="Details card with an action (DetailsAction)">
      <div class="w-dialog max-w-full">
        <DetailsCard class="gap-10">
          <Heading as="h2" :variant="HeadingVariant.Overline">Salvataggio del gioco</Heading>
          <span class="tabular-nums">Ultima modifica: oggi, 21:14</span>
          <div class="flex flex-wrap gap-8">
            <DetailsAction>Mostra i backup</DetailsAction>
            <DetailsAction>Importa…</DetailsAction>
            <DetailsAction>Esporta…</DetailsAction>
          </div>
        </DetailsCard>
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="File non raggiungibile: la scheda dice «Su questo computer»"
        :status="IssueStatus.Proposed"
      >
        Per un gioco la cui cartella non si raggiunge, «File del gioco» scrive comunque «Su questo
        computer» e il percorso, come se il file ci fosse. È proprio la scheda dove l’utente va a
        cercare dov’è finito il file.
        <template #current>
          <div class="w-dialog max-w-full">
            <GameFileCard :game="GBC_MISSING" device="desktop" />
          </div>
        </template>
        <template #proposed>
          <div class="w-dialog max-w-full">
            <LibraryGameFileCardProposal :game="GBC_MISSING" device="desktop" />
          </div>
        </template>
        <template #solution>
          in <code>details/GameFileCard.vue</code>, con <code>!game.available</code>, al posto di
          <code>deviceLine</code> la riga warning con l’icona e
          <code>errors.libraryGameUnavailable</code> (lo stesso testo della testata dei dettagli).
        </template>
      </KitIssue>
      <KitIssue title="Il percorso va a capo a metà parola" :status="IssueStatus.Proposed">
        Il percorso usa <code>break-all</code>: va a capo dove finisce la riga anche in mezzo a una
        parola che potrebbe scendere intera («For / gotten Cartridges»). Il resto della libreria usa
        <code>wrap-anywhere</code>, che prova prima spazi e separatori.
        <template #current>
          <div class="w-dialog max-w-full">
            <GameFileCard :game="GB_LONG" device="desktop" />
          </div>
        </template>
        <template #proposed>
          <div class="w-dialog max-w-full">
            <LibraryGameFileCardProposal :game="GB_LONG" device="desktop" />
          </div>
        </template>
        <template #solution>
          in <code>details/GameFileCard.vue</code> <code>wrap-anywhere</code> al posto di
          <code>break-all</code> sulla riga del percorso; l’impronta in esadecimale tiene
          <code>break-all</code> (non ha spazi).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
