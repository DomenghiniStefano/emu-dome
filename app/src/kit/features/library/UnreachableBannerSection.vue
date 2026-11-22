<script setup lang="ts">
// UnreachableBanner (library/UnreachableBanner.vue): E01, a library folder that is not reachable.
import UnreachableBanner from '@/components/library/UnreachableBanner.vue'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import LibraryBannerProposal from '@/kit/proposals/LibraryBannerProposal.vue'
import { IssueStatus, KitGroup } from '@/kit/review'

const noop = () => {}
const NAS = '//nas-salotto/condivisa/Emulazione/Nintendo DS/Collezione completa (Europa)'
</script>

<template>
  <KitSection title="Unreachable folder banner" :group="KitGroup.Library">
    <KitState label="Short path, one game">
      <div class="w-page-md max-w-full">
        <UnreachableBanner :folder="{ path: 'E:/Giochi', count: 1 }" @replace="noop" />
      </div>
    </KitState>
    <KitState label="Long network path, 312 games">
      <div class="w-page-md max-w-full">
        <UnreachableBanner :folder="{ path: NAS, count: 312 }" @replace="noop" />
      </div>
    </KitState>
    <KitState label="Narrow (tile-min)">
      <div class="w-tile-min">
        <UnreachableBanner :folder="{ path: 'E:/Giochi/GBA', count: 4 }" @replace="noop" />
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="In una finestra stretta il testo diventa una colonna di lettere"
        :status="IssueStatus.Proposed"
      >
        La riga non va mai a capo: il pulsante «Scegli un’altra cartella» tiene la sua larghezza e
        il testo, con <code>wrap-anywhere</code>, si spezza lettera per lettera in una colonna larga
        tre caratteri. Succede sotto i 300 px circa, cioè sul telefono in verticale e nella finestra
        desktop più stretta.
        <template #current>
          <div class="w-tile-min">
            <UnreachableBanner :folder="{ path: 'E:/Giochi/GBA', count: 4 }" @replace="noop" />
          </div>
        </template>
        <template #proposed>
          <div class="w-tile-min">
            <LibraryBannerProposal kind="unreachable" path="E:/Giochi/GBA" :count="4" />
          </div>
          <div class="w-dialog max-w-full">
            <LibraryBannerProposal kind="unreachable" :path="NAS" :count="312" />
          </div>
        </template>
        <template #solution>
          in <code>library/UnreachableBanner.vue</code> <code>flex-wrap</code> sulla riga, icona e
          testo in un gruppo con una larghezza minima (un token di ruolo nuovo, per esempio
          <code>banner-text-min</code>; la proposta usa per ora <code>basis-menu-min</code>, 220) e
          il pulsante <code>ml-auto</code>: quando non c’è posto il pulsante scende sotto il testo,
          allineato a destra. Vale per tutti gli avvisi se si accetta la proposta in «Scan progress
          and summary».
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
