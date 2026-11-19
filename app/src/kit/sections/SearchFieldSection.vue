<script setup lang="ts">
import { ref } from 'vue'
import { SearchField } from '@/components/ui/search-field'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const empty = ref('')
const filled = ref('Pocket Quest')
</script>

<template>
  <KitSection title="SearchField">
    <KitState label="Default · empty">
      <SearchField
        v-model="empty"
        class="w-field"
        label="Search for a game"
        placeholder="Search for a game"
      />
    </KitState>
    <KitState label="Default · with a query">
      <SearchField
        v-model="filled"
        class="w-field"
        label="Search for a game"
        placeholder="Search for a game"
      />
    </KitState>
  </KitSection>
</template>
