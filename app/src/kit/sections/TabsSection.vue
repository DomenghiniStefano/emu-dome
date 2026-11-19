<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsContent, TabsList, TabsSize, TabsTrigger } from '@/components/ui/tabs'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const tab = ref('overview')
const large = ref('saves')
const pill = ref('saves')
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
  </KitSection>
</template>
