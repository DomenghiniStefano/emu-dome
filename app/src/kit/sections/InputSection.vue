<script setup lang="ts">
import { ref } from 'vue'
import { Input } from '@/components/ui/input'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const empty = ref('')
const filled = ref('Pocket Quest')
</script>

<template>
  <KitSection title="Input">
    <KitState label="Empty, with a placeholder">
      <Input v-model="empty" class="w-field" placeholder="Pocket Quest Advance" />
    </KitState>
    <KitState label="Filled">
      <Input v-model="filled" class="w-field" />
    </KitState>
    <KitState label="Disabled">
      <Input v-model="filled" class="w-field" disabled />
    </KitState>
  </KitSection>
</template>
