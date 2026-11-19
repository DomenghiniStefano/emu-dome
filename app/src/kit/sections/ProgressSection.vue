<script setup lang="ts">
import { Progress } from '@/components/ui/progress'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
</script>

<template>
  <KitSection title="Progress">
    <KitState label="Indeterminate (no total known: the band sweeps)">
      <Progress class="w-scan" label="Scanning" />
    </KitState>
    <KitState label="Determinate: 40%, done">
      <div class="flex w-scan flex-col gap-8">
        <Progress :value="40" label="Importing" />
        <Progress :value="100" label="Done" />
      </div>
    </KitState>
  </KitSection>
</template>
