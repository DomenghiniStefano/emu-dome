<script setup lang="ts">
// G10: the controller shortcuts, read-only (they are edited in Controls, C03).
import type { ComboRow, Pos } from '@/emu'
import { Card, CardVariant } from '@/design-system/components/card'
import { comboText, hotkeyName } from '@/lib/controls'
import MenuList from './MenuList.vue'
import { FocusMark } from './view'

defineProps<{ combos: ComboRow[]; modifier: Pos }>()

const initial = { [FocusMark.SubInitial]: '' }
</script>

<template>
  <MenuList>
    <Card
      v-for="(r, i) in combos"
      :key="r.hotkey"
      v-bind="i === 0 ? initial : {}"
      :variant="CardVariant.ListRow"
      class="gap-8"
      tabindex="0"
    >
      <span class="block min-w-0 flex-1 font-medium">{{ hotkeyName(r.hotkey) }}</span>
      <span class="text-ink-muted">{{ comboText('generic', modifier, r.pos) }}</span>
    </Card>
  </MenuList>
</template>
