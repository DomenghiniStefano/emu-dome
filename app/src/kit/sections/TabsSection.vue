<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsContent, TabsList, TabsSize, TabsTrigger } from '@/components/ui/tabs'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

const tab = ref('overview')
const large = ref('saves')
const pill = ref('saves')
const now = ref('overview')
const proposed = ref('overview')
</script>

<template>
  <KitSection title="Tabs">
    <KitState label="Default (40)">
      <Tabs v-model="tab">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="remap">Remap</TabsTrigger>
          <TabsTrigger value="sticks" disabled>Sticks</TabsTrigger>
        </TabsList>
        <TabsContent value="overview" class="pt-12 text-ink-muted">Overview panel</TabsContent>
        <TabsContent value="remap" class="pt-12 text-ink-muted">Remap panel</TabsContent>
      </Tabs>
    </KitState>
    <KitState label="Large (44)">
      <Tabs v-model="large" :size="TabsSize.Large">
        <TabsList>
          <TabsTrigger value="saves">Saves</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
          <TabsTrigger value="files">Files</TabsTrigger>
        </TabsList>
      </Tabs>
    </KitState>
    <KitState label="Large on the phone: pills (36)">
      <Tabs v-model="pill" :size="TabsSize.Large" data-density="touch">
        <TabsList class="gap-8 border-b-0">
          <TabsTrigger value="saves">Saves</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
          <TabsTrigger value="files">Files</TabsTrigger>
        </TabsList>
      </Tabs>
    </KitState>
    <template #review>
      <KitIssue
        title="Una tab disabilitata è uguale a una non scelta"
        :status="IssueStatus.Proposed"
      >
        TabsTrigger disabilitata cambia solo il cursore: «Sticks» qui sopra ha lo stesso grigio di
        «Remap». Col controller il passo la salta senza dire perché, col mouse sembra una tab che
        non risponde.
        <template #current>
          <Tabs v-model="now">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="remap">Remap</TabsTrigger>
              <TabsTrigger value="sticks" disabled>Sticks</TabsTrigger>
            </TabsList>
          </Tabs>
        </template>
        <template #proposed>
          <Tabs v-model="proposed">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="remap">Remap</TabsTrigger>
              <TabsTrigger value="sticks" disabled class="disabled:opacity-disabled">
                Sticks
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </template>
        <template #solution>
          disabled:opacity-disabled nella base di tabsTriggerVariants (tabs/variants.ts), come
          Button.
        </template>
      </KitIssue>
      <KitIssue title="Col focus la tab attiva perde la sua riga">
        La riga sotto la tab attiva è un box-shadow (shadow-tab). L'anello di focus (styles.css,
        :root[data-nav] :focus) imposta un altro box-shadow fuori dai layer e lo sostituisce: mentre
        la tab attiva ha il focus non si vede più quale sia la scelta. Per vederlo: premi una
        freccia e porta il focus su «Overview» qui sopra.
        <template #solution>
          La riga della tab attiva diventa un bordo inferiore o un pseudo-elemento
          (tabs/variants.ts), oppure l'anello di focus rinuncia al box-shadow (lo stesso problema
          tocca Rail e Swatch, vedi Button).
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
