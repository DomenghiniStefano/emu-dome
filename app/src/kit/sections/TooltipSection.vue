<script setup lang="ts">
// Tooltip (Trello #104): the name of an icon alone. A Button with `label` shows it by itself, on a
// rest of the mouse and on the controller's or the keyboard's focus; «Mostra» pins one open on a
// button here so its look can be read without hovering, in every theme.
import { computed, ref } from 'vue'
import { Pin, Play, Share, Trash2 } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Tooltip } from '@/components/ui/tooltip'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'

const pinned = ref(false)
const target = ref<InstanceType<typeof Button> | null>(null)
const targetEl = computed(() => (target.value?.$el as HTMLElement | undefined) ?? null)
</script>

<template>
  <KitSection title="Tooltip">
    <KitState label="On a Button with label: hover, or move the focus with the arrows / Tab">
      <div class="flex flex-wrap items-center gap-8">
        <Button :size="ButtonSize.IconRow" :icon="Play" icon-filled label="Gioca questa" />
        <Button
          :variant="ButtonVariant.GhostRegular"
          :size="ButtonSize.IconRow"
          :icon="Pin"
          label="Mostra questa in libreria"
        />
        <Button :size="ButtonSize.ToolIcon" :icon="Share" label="Esporta…" />
      </div>
    </KitState>
    <KitState label="A caller's title is shown in place of the label (aria-disabled, the reason)">
      <Button
        :size="ButtonSize.IconRow"
        :icon="Trash2"
        label="Togli"
        aria-disabled="true"
        title="Disponibile più avanti"
      />
    </KitState>
    <KitState label="Pinned open (the look, without hovering)">
      <div class="flex items-center gap-8 pt-24">
        <Button ref="target" :size="ButtonSize.IconRow" :icon="Play" icon-filled label="Riprendi" />
        <Button :size="ButtonSize.Small" @click="pinned = !pinned">Mostra</Button>
        <Tooltip
          v-if="pinned && targetEl"
          :anchor="targetEl"
          text="Riprendi"
          @close="pinned = false"
        />
      </div>
    </KitState>
  </KitSection>
</template>
