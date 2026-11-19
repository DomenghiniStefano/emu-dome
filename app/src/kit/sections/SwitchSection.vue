<script setup lang="ts">
import { ref } from 'vue'
import { Switch } from '@/components/ui/switch'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const on = ref(true)
const off = ref(false)
</script>

<template>
  <KitSection title="Switch">
    <KitState label="On">
      <Switch v-model="on" aria-label="On" />
    </KitState>
    <KitState label="Off">
      <Switch v-model="off" aria-label="Off" />
    </KitState>
    <KitState label="On · disabled">
      <Switch :model-value="true" disabled aria-label="On, disabled" />
    </KitState>
    <KitState label="Off · disabled">
      <Switch :model-value="false" disabled aria-label="Off, disabled" />
    </KitState>
  </KitSection>
</template>
