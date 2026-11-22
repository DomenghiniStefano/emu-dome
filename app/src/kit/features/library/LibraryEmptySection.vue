<script setup lang="ts">
// LibraryEmpty (library/LibraryEmpty.vue): L02, the library with no games.
import LibraryEmpty from '@/components/library/LibraryEmpty.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { KitGroup } from '@/kit/review'

const noop = () => {}
const ERROR = 'La cartella D:/Giochi non si può leggere. Controlla i permessi e riprova.'
</script>

<template>
  <KitSection title="Empty library" :group="KitGroup.Library">
    <KitState label="No games">
      <div class="flex w-page-md max-w-full">
        <LibraryEmpty :error="null" @choose-folder="noop" @open-file="noop" />
      </div>
    </KitState>
    <KitState label="The folder could not be added">
      <div class="flex w-page-md max-w-full">
        <LibraryEmpty :error="ERROR" @choose-folder="noop" @open-file="noop" />
      </div>
    </KitState>
  </KitSection>
</template>
